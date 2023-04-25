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

	pub trait FloatOrComplex : Sized {
		fn eval_poly<'a>(self,coeffs:&'a [Self]) -> Self;
	}

	macro_rules! impl_float {
		($($types:ident)+) => { $(
			// 実数に対する Horner の方法による実装
			impl FloatOrComplex for $types {
				fn eval_poly<'a>(self,coeffs:&'a [$types]) -> Self {
					use primitive_functions::*;

					let mut iter = coeffs.iter().rev();
					// 最高次の値を取り出す
					let mut val = match iter.next() {
						Some(v) => *v,
						None => { return 0.0; }
					};
					// 残りの次数について計算して val に足し合わせる
					for c in iter {
						val = mul_add(val,self,*c);
					}
					val
				}
			}
		)+ };
	}
	impl_float!( f64 f32 );

	impl<F> FloatOrComplex for Complex<F> where F: Float, f64: Into<F> {
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

	#[inline]
	/// 多項式 `f(x) = c₀ + c₁x + c₂x² + ...` の値を計算します
	/// * `x` ... 変数の値
	/// * `coeffs` ... 係数 (`coeffs[n]` が n 次の項の係数)
	pub fn eval_poly<'a,T: FloatOrComplex>(x:T,coeffs:&'a [T]) -> T {
		x.eval_poly(coeffs)
	}

}
pub use evaluate_polynomials::eval_poly;
