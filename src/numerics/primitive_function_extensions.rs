use super::*;

/// `hypot` の拡張
mod hypot_extension {
	use super::*;

	compose_struct! {
		pub trait Iter<T> = IntoIterator<Item=T>;
	}

	/// `hypot` 関数を多数の要素でも使えるようにするトレイト
	pub trait HypotForArray<T> {
		/// 多数個の要素に対して平方和のルートを計算する
		fn hypot(self) -> T;
	}
	impl<T:Float, I:Iter<T>> HypotForArray<T> for I {
		fn hypot(self) -> T {
			self.into_iter()
			.reduce( |a,v| a.hypot(v) )
			.unwrap_or(T::zero())
		}
	}

	/// `hypot` 関数を多数の要素でも使えるようにするトレイト
	pub trait HypotForTuple<T> {
		/// 多数個の要素に対して平方和のルートを計算する
		fn hypot(self) -> T;
	}

	/// * タプル `(T,T,...)` の各要素に対して、 hypot を計算するトレイト `HypotForTuple` の実装をまとめて行うマクロ
	/// * `impl_hypot!(indices: 1 2 ... (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! impl_hypot {
		(indices: $($i:tt)+ ) => {
			impl_hypot! {@each T | $($i),+ }
		};
		(@each $t:ident $($tx:ident $x:tt),* | $y0:tt $(,$y:tt)* ) => {
			impl_hypot! {@each $t $($tx $x),* | }
			impl_hypot! {@each $t $($tx $x,)* $t $y0 | $($y),* }
		};
		(@each $t:ident $($tx:ident $x:tt),* | ) => {
			impl<$t:Float> HypotForTuple<$t> for ($t,$($tx),*) {
				#[inline]
				fn hypot(self) -> $t {
					self.0 $( .hypot(self.$x) )*
				}
			}
		};
	}
	pub(crate) use impl_hypot;

}
pub use hypot_extension::*;



/// `mul_add` の拡張
mod mul_add_extension {
	use super::*;
	use primitive_functions::mul_add;
	use primitive_functions::float_misc::MulAdd;

	/// `mul_add` を複数個の要素に拡張するトレイト
	pub trait MulAddExtension<T> {
		/// ## `mul_add`
		/// * 複数個の値のペアの積をとり、それらの和をとる。
		/// * `mul_add` を使ってより正確な値を得ることができる。
		/// * `([(a1,b1),(a2,b2),...],c).mul_add()` という表記により `a1*b1+a2*b2+...+c` を得る。
		fn mul_add(self) -> T;
	}

	impl<T,I> MulAddExtension<T> for (I,T)
	where T: Float + MulAdd, I: IntoIterator<Item=(T,T)>
	{
		fn mul_add(self) -> T {
			self.0.into_iter()
			.fold(
				self.1,
				|a,(x,y)| mul_add(x,y,a)
			)
		}
	}

}
pub use mul_add_extension::*;



/// 多項式の計算を効率よく行う `eval_poly` を定義するモジュール
mod evaluate_polynomials {
	use super::*;
	use primitive_functions::float_misc::MulAdd;
	type C<T> = Complex<T>;

	// 型に合わせた実装部

	/// 実数に対する Horner の方法による実装
	fn eval_poly_real_real<'c,F>(x:F,coeffs:&'c [F]) -> F
	where F: Float + MulAdd
	{
		use primitive_functions::mul_add;

		let mut iter = coeffs.iter().rev();
		// 最高次の値を取り出す
		let mut val = match iter.next() {
			Some(v) => *v,
			None => { return F::zero(); }
		};
		// 残りの次数について計算して val に足し合わせる
		for c in iter {
			val = mul_add(val,x,*c);
		}
		val
	}
	/// 係数が複素数の場合も同様
	fn eval_poly_real_complex<'c,F>(x:F,coeffs:&'c [C<F>]) -> C<F>
	where F: Float + MulAdd
	{
		use primitive_functions::mul_add;

		let mut iter = coeffs.iter().rev();
		// 最高次の値を取り出す
		let mut val = match iter.next() {
			Some(v) => *v,
			None => { return C::zero(); }
		};
		// 残りの次数について計算して val に足し合わせる
		for c in iter {
			val.re = mul_add(val.re,x,c.re);
			val.im = mul_add(val.im,x,c.im);
		}
		val
	}
	/// 複素数に対する Goertzel の方法による実装
	fn eval_poly_complex_complex<'c,F>(z:C<F>,coeffs:&'c [C<F>]) -> C<F>
	where F: Float + MulAdd, f32: Into<F>
	{
		use primitive_functions::mul_add;

		// 入力した変数 z = x+iy に対して p = 2x, q = - (x²+y²) を計算する
		let C { re: x,im: y } = z;
		let p = x * 2.0.into();
		let q = - mul_add( x, x, y*y );

		// 作業変数 a, b を用意する。初期値は a が最高次, b がその次の次数の係数である。0次の場合 (coeffs の要素数が1の場合) と、係数がない場合 (coeffs の要素数が0の場合) は早期にリターンする。
		let mut iter = coeffs.iter().rev();
		let mut a = match iter.next() {
			Some(c) => *c,
			None => { return C::zero(); }
		};
		let mut b = match iter.next() {
			Some(c) => *c,
			None => { return a; }
		};

		// 残りの次数について漸化的に a = pa+b, b = qa + c を変化させる
		for c in iter {
			(a.re,a.im,b.re,b.im) = (
				mul_add(p,a.re,b.re),
				mul_add(p,a.im,b.im),
				mul_add(q,a.re,c.re),
				mul_add(q,a.im,c.im)
			);
		}

		// 最後に za + b を計算してから返す
		C {
			re: ([(x,a.re),(-y,a.im)],b.re).mul_add(),
			im: ([(y,a.re),( x,a.im)],b.im).mul_add()
		}
	}
	/// 係数が実数の場合も同様
	fn eval_poly_complex_real<'c,F>(z:C<F>,coeffs:&'c [F]) -> C<F>
	where F: Float + MulAdd, f32: Into<F>
	{
		use primitive_functions::mul_add;

		// 入力した変数 z = x+iy に対して p = 2x, q = - (x²+y²) を計算する
		let C { re: x,im: y } = z;
		let p = x * 2.0.into();
		let q = - mul_add( x, x, y*y );

		// 作業変数 a, b を用意する。初期値は a が最高次, b がその次の次数の係数である。0次の場合 (coeffs の要素数が1の場合) と、係数がない場合 (coeffs の要素数が0の場合) は早期にリターンする。
		let mut iter = coeffs.iter().rev();
		let mut a = match iter.next() {
			Some(c) => C { re: *c, im: F::zero() },
			None => { return C::zero(); }
		};
		let mut b = match iter.next() {
			Some(c) => C { re: *c, im: F::zero() },
			None => { return a; }
		};

		// 残りの次数について漸化的に a = pa+b, b = qa + c を変化させる
		for c in iter {
			(a.re,a.im,b.re,b.im) = (
				mul_add(p,a.re,b.re),
				mul_add(p,a.im,b.im),
				mul_add(q,a.re,*c),
				mul_add(q,a.im,F::zero())
			);
		}

		// 最後に za + b を計算してから返す
		C {
			re: ([(x,a.re),(-y,a.im)],b.re).mul_add(),
			im: ([(y,a.re),( x,a.im)],b.im).mul_add()
		}
	}

	/// `eval_poly` で受け入れられる型を抽象化したトレイト
	pub trait EvaluatePolynomials<X,R> {
		fn eval_poly(&self,x:X) -> R;
	}

	// 実装とリンク
	macro_rules! impl_eval_poly {
		( $( ($x:ty,$c:ty) -> $r:ty => $f:ident )+ ) => { $(
			impl EvaluatePolynomials<$x,$r> for [$c] {
				#[inline]
				fn eval_poly(&self,x:$x) -> $r {
					$f(x,self)
				}
			}
		)+ };
	}
	impl_eval_poly! {
		(  f64 ,  f64 ) ->   f64  => eval_poly_real_real
		(  f32 ,  f32 ) ->   f32  => eval_poly_real_real
		(  f64 ,C<f64>) -> C<f64> => eval_poly_real_complex
		(  f32 ,C<f32>) -> C<f32> => eval_poly_real_complex
		(C<f64>,  f64 ) -> C<f64> => eval_poly_complex_real
		(C<f32>,  f32 ) -> C<f32> => eval_poly_complex_real
		(C<f64>,C<f64>) -> C<f64> => eval_poly_complex_complex
		(C<f32>,C<f32>) -> C<f32> => eval_poly_complex_complex
	}

	// 外部からアクセスできるインターフェース

	#[inline]
	/// 多項式 `f(x) = c₀ + c₁x + c₂x² + ...` の値を計算します
	/// * `x` ... 変数の値
	/// * `coeffs` ... 係数 (`coeffs[n]` が n 次の項の係数)
	pub fn eval_poly<'c,X,C,R>(x:X,coeffs:&'c [C]) -> R
	where [C]: EvaluatePolynomials<X,R>
	{ <[C]>::eval_poly(coeffs,x) }

}
pub use evaluate_polynomials::eval_poly;



/// 方程式を計算するモジュール
mod equation {
	use super::*;
	use primitive_functions::*;

	type C<F> = Complex<F>;
	type C64 = C<f64>;
	type C32 = C<f32>;

	fn solve3(p:C64,q:C64,r:C64) -> [C64;3] {
		let p3:C64 = p / 3.0;
		let a:C64 = power(p3,3) - (p*q) / 6.0 + r / 2.0;
		let b:C64 = power(p3,2) - q / 3.0;
		let bs:C64 = sqrt(b);

		let c = a / power(bs,3);
		let t0 = acos(c) / 3.0;
		(0..3).map(|n| {
			use std::f64::consts::PI;
			let d = ((2*n+1) as f64)/3.0 * PI;
			let t = t0 + d;
			cos(t) * (2.0*bs) - (p/3.0)
		}).collect::<Vec<_>>().try_into().unwrap()
	}

	fn solve4(a:C64,b:C64,c:C64,d:C64) -> [C64;4] {
		let p:C64 = - 3.0 / 8.0 * power(a,2) + b;
		let q:C64 = power(a,2) / 8.0 - a * b / 2.0 + c;
		let r:C64 = - 3.0 * power(a/4.0,4) + b * power(a/4.0,2) - a * c / 4.0 + d;

		let [xs1,xs2,xs3] = solve3(2.0*p,p*p-4.0*r,-q*q);
		let mut x1 = sqrt(xs1) / 2.0;
		let mut x2 = sqrt(xs2) / 2.0;
		let mut x3 = sqrt(xs3) / 2.0;

		let sign = (x1*x2*x3/q).re.signum();
		x1 *= sign;
		x2 *= sign;
		x3 *= sign;

		[
			(-x1) + (-x2) + (-x3),
			(-x1) + ( x2) + ( x3),
			( x1) + (-x2) + ( x3),
			( x1) + ( x2) + (-x3)
		]
	}

}
