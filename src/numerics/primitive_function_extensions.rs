use super::*;

/// `hypot` の拡張
mod hypot_extension {
	use super::*;

	compose_struct! {
		pub trait Iter<T> = IntoIterator<Item=T>;
	}

	/// `hypot` 関数を多数の要素でも使えるようにするトレイト
	pub trait HypotFn<T> {
		/// 多数個の要素に対して平方和のルートを計算する
		fn hypot(self) -> T;
	}
	impl<T:Float, I:Iter<T>> HypotFn<T> for I {
		fn hypot(self) -> T {
			self.into_iter()
			.reduce( |a,v| a.hypot(v) )
			.unwrap_or(T::zero())
		}
	}

}
pub use hypot_extension::*;



/// `mul_add` の拡張
mod mul_add_extension {
	use super::*;
	use primitive_functions::mul_add;

	/// `mul_add` を複数個の要素に拡張するトレイト
	pub trait MulAddExtension<T> {
		/// ## `mul_add`
		/// 複数個の値のペアの積をとり、それらの和をとる。 `mul_add` を使ってより正確な値を得ることができる。
		/// * `[(a1,b1),(a2,b2),...].mul_add()` : `a1*b1+a2*b2+...` を得る
		/// * `([(a1,b1),(a2,b2),...],c).mul_add()` : `a1*b1+a2*b2+...+c` を得る
		fn mul_add(self) -> T;
	}
	impl<I,T> MulAddExtension<T> for (I,T)
	where I: IntoIterator<Item=(T,T)>, T: Float
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



/// `sqrt`, `cbrt` の拡張
mod sqrt_cbrt_extension {
	use super::*;

	/// 複素数の `sqrt`, `cbrt` を拡張するトレイト
	pub trait SqrtCbrtExtension : Sized {
		/// 全ての2乗根を返す
		fn sqrt_all(&self) -> [Self;2];
		/// 全ての3乗根を返す
		fn cbrt_all(&self) -> [Self;3];
	}

	impl<F> SqrtCbrtExtension for Complex<F> where F: Float, f64: Into<F> {
		fn sqrt_all(&self) -> [Self;2] {
			let p = self.sqrt();
			[p,-p]
		}
		fn cbrt_all(&self) -> [Self;3] {
			let p = self.cbrt();
			let t1 = Complex::from_polar(F::one(), (120.0).into().to_radians());
			let t2 = Complex::from_polar(F::one(), (240.0).into().to_radians());
			[p,p*t1,p*t2]
		}
	}

}
pub use sqrt_cbrt_extension::*;



/// 多項式の計算を効率よく行う `eval_poly` を定義するモジュール
mod evaluate_polynomials {
	use super::*;
	type C<T> = Complex<T>;

	pub trait FloatOrComplex : Sized {
		fn eval_poly<'a>(self,coeffs:&'a [Self]) -> Self;
	}

	impl<F> FloatOrComplex for C<F> where F: Float, f64: Into<F> {
		// 複素数に対する Goertzel の方法による実装
		fn eval_poly<'a>(self,coeffs:&'a [Self]) -> Self {
			use primitive_functions::mul_add;

			// 入力した変数 z = x+iy に対して p = 2x, q = - (x²+y²) を計算する
			let Self { re: x,im: y } = self;
			let p = x * 2.0.into();
			let q = - mul_add( x, x, y*y );

			// 作業変数 a, b を用意する。初期値は a が最高次, b がその次の次数の係数である。0次の場合 (coeffs の要素数が1の場合) と、係数がない場合 (coeffs の要素数が0の場合) は早期にリターンする。
			let mut iter = coeffs.iter().rev();
			let mut a = match iter.next() {
				Some(c) => *c,
				None => { return Self::zero(); }
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
			Self {
				re: ([(x,a.re),(-y,a.im)],b.re).mul_add(),
				im: ([(y,a.re),( x,a.im)],b.im).mul_add()
			}
		}
	}

	// 型に合わせた実装部

	/// 実数に対する Horner の方法による実装
	fn eval_poly_real_real<'c,F>(x:F,coeffs:&'c [F]) -> F where F: Float {
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
	fn eval_poly_real_complex<'c,F>(x:F,coeffs:&'c [C<F>]) -> C<F> where F: Float {
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
	fn eval_poly_complex_complex<'c,F>(z:C<F>,coeffs:&'c [C<F>]) -> C<F> where F: Float, f32: Into<F> {
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
	fn eval_poly_complex_real<'c,F>(z:C<F>,coeffs:&'c [F]) -> C<F> where F: Float, f32: Into<F> {
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

	impl EvaluatePolynomials<f64,f64> for [f64] {
		#[inline]
		fn eval_poly(&self,x:f64) -> f64 {
			eval_poly_real_real(x,self)
		}
	}
	impl EvaluatePolynomials<f32,f32> for [f32] {
		#[inline]
		fn eval_poly(&self,x:f32) -> f32 {
			eval_poly_real_real(x,self)
		}
	}
	impl EvaluatePolynomials<f64,C<f64>> for [C<f64>] {
		#[inline]
		fn eval_poly(&self,x:f64) -> C<f64> {
			eval_poly_real_complex(x,self)
		}
	}
	impl EvaluatePolynomials<f32,C<f32>> for [C<f32>] {
		#[inline]
		fn eval_poly(&self,x:f32) -> C<f32> {
			eval_poly_real_complex(x,self)
		}
	}
	impl EvaluatePolynomials<C<f64>,C<f64>> for [f64] {
		#[inline]
		fn eval_poly(&self,z:C<f64>) -> C<f64> {
			eval_poly_complex_real(z,self)
		}
	}
	impl EvaluatePolynomials<C<f32>,C<f32>> for [f32] {
		#[inline]
		fn eval_poly(&self,z:C<f32>) -> C<f32> {
			eval_poly_complex_real(z,self)
		}
	}
	impl<F> EvaluatePolynomials<C<F>,C<F>> for [C<F>] where F: Float + From<f32> {
		#[inline]
		fn eval_poly(&self,x:C<F>) -> C<F> {
			eval_poly_complex_complex(x,self)
		}
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
