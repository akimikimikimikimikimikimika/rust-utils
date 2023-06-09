use super::*;
type C<T> = Complex<T>;
type R<T> = num::rational::Ratio<T>;

/// 指数関数や対数関数を定義するモジュール
mod exp_log {
	use super::*;

	/// `Float` と `Complex` に対して `log` に対応するトレイト
	pub trait Logarithm<B>: Sized {
		fn log_impl(self,base:B) -> Self;
	}
	/// `Float` と `Complex` に対して `ln` に対応するトレイト
	pub trait NaturalLogarithm: Sized {
		fn ln_impl(self) -> Self;
	}
	/// `Float` と `Complex` に対して `exp` に対応するトレイト
	pub trait Exponential: Sized {
		fn exp_impl(self) -> Self;
	}

	macro_rules! log_impl {
		( $t:ty, $b:ty ) => {
			impl Logarithm<$b> for $t {
				#[inline]
				fn log_impl(self,base:$b) -> $t { self.log(base) }
			}
			impl NaturalLogarithm for $t {
				#[inline]
				fn ln_impl(self) -> $t { self.ln() }
			}
			impl Exponential for $t {
				#[inline]
				fn exp_impl(self) -> $t { self.exp() }
			}
		};
	}
	log_impl!( f64, f64 );
	log_impl!( f32, f32 );
	log_impl!( C<f64>, f64 );
	log_impl!( C<f32>, f32 );

	#[inline]
	pub fn log<T,B>(x:T,base:B) -> T where T: Logarithm<B> {
		x.log_impl(base)
	}
	#[inline]
	pub fn ln<T>(x:T) -> T where T: NaturalLogarithm {
		x.ln_impl()
	}
	#[inline]
	pub fn exp<T>(x:T) -> T where T: Exponential {
		x.exp_impl()
	}

	macro_rules! functions {
		( $($name:ident)+ ) => { $(
			#[inline]
			pub fn $name<F:Float>(x:F) -> F {
				x.$name()
			}
		)+ };
	}
	functions!( log2 log10 ln_1p exp2 exp_m1 );

}
pub use exp_log::{log,ln,log2,log10,ln_1p,exp,exp2,exp_m1};

/// 2乗根、3乗根、n乗根を定義するモジュール
pub(crate) mod root {
	use super::*;

	/// `Float` と `Complex` に対して `sqrt`, `cbrt` に対応するトレイト
	pub trait Root: Sized {
		fn sqrt_impl(self) -> Self;
		fn cbrt_impl(self) -> Self;
	}

	/// 1のn乗根の値を計算するトレイト
	pub trait Unit: Sized {
		fn unit(n:usize) -> Vec<Self>;
	}

	/// `Float` と `Complex` に対して `sqrt_all`, `cbrt_all` に対応するトレイト
	pub trait RootAll<C>: Root {
		fn sqrt_all(self) -> [C;2];
		fn cbrt_all(self) -> [C;3];
		fn root_all<const N:usize>(self) -> [C;N];
	}

	macro_rules! impl_root {
		( $($f:ident)+ ) => { $(

			impl Root for $f {
				#[inline]
				fn sqrt_impl(self) -> Self { self.sqrt() }
				#[inline]
				fn cbrt_impl(self) -> Self { self.cbrt() }
			}

			impl Root for C<$f> {
				#[inline]
				fn sqrt_impl(self) -> Self { self.sqrt() }
				#[inline]
				fn cbrt_impl(self) -> Self { self.cbrt() }
			}

			impl Unit for C<$f> {
				fn unit(n:usize) -> Vec<Self> {
					use std::$f::consts::TAU;
					(0..n).map(|i| {
						C::from_polar(1.0, (i as $f)/(n as $f)*TAU )
					})
					.collect::<Vec<_>>()
				}
			}

			impl RootAll<C<$f>> for C<$f> {
				fn sqrt_all(self) -> [C<$f>;2] {
					let p = self.sqrt();
					[p,-p]
				}
				fn cbrt_all(self) -> [C<$f>;3] {
					let p = self.cbrt();
					let t1 = Complex::from_polar(
						1.0,
						120.0.to_radians()
					);
					let t2 = Complex::from_polar(
						1.0,
						240.0.to_radians()
					);
					[p,p*t1,p*t2]
				}
				fn root_all<const N:usize>(self) -> [C<$f>;N] {
					let p = self.powf(1.0/(N as $f));
					Self::unit(N).into_iter()
					.map(|r| p*r )
					.collect::<Vec<_>>()
					.try_into().unwrap()
				}
			}

			impl RootAll<C<$f>> for $f {
				#[inline]
				fn sqrt_all(self) -> [C<$f>;2] {
					C {re:self,im:0.0}.sqrt_all()
				}
				#[inline]
				fn cbrt_all(self) -> [C<$f>;3] {
					C {re:self,im:0.0}.cbrt_all()
				}
				#[inline]
				fn root_all<const N:usize>(self) -> [C<$f>;N] {
					C {re:self,im:0.0}.root_all::<N>()
				}
			}

		)+ };
	}
	impl_root!( f64 f32 );

	#[inline]
	/// 平方根を計算します。 `Float` と `Complex` に対応します。
	/// * 入力値が複素数の場合は主値を返します。
	/// * 実数で負の値を与えると `NaN` を返します。
	pub fn sqrt<T: Root>(x:T) -> T { x.sqrt_impl() }
	#[inline]
	/// 立方根を計算します。 `Float` と `Complex` に対応します。
	/// * 入力値が複素数の場合は主値を返します。
	pub fn cbrt<T: Root>(x:T) -> T { x.cbrt_impl() }
	#[inline]
	/// 平方根を計算します。 `Float` と `Complex` に対応します。
	/// * 全ての根を計算します。
	/// * 入力値が実数であっても、対応する複素数型を返します。
	pub fn sqrt_all<INPUT,ROOT>(x:INPUT) -> [ROOT;2] where INPUT: RootAll<ROOT> { x.sqrt_all() }
	#[inline]
	/// 立方根を計算します。 `Float` と `Complex` に対応します。
	/// * 全ての根を計算します。
	/// * 入力値が実数であっても、対応する複素数型を返します。
	pub fn cbrt_all<INPUT,ROOT>(x:INPUT) -> [ROOT;3] where INPUT: RootAll<ROOT> { x.cbrt_all() }
	#[inline]
	/// n乗根を計算します。 `Float` と `Complex` に対応します。
	/// * 全ての根を計算します。
	/// * 入力値が実数であっても、対応する複素数型を返します。
	pub fn root_all<INPUT,ROOT,const N:usize>(x:INPUT) -> [ROOT;N] where INPUT: RootAll<ROOT> { x.root_all::<N>() }

}
pub use root::{sqrt,cbrt,sqrt_all,cbrt_all,root_all};

/// 三角関数に対する関数定義をまとめて行うマクロ
macro_rules! trig {
	( func($($f:ident)+) types($($t:ty),+) ) => {
		/// 三角関数を定義するモジュール
		pub(crate) mod trigonometric {
			use super::*;

			/// `Float` と `Complex` に対して諸々の三角関数に対応するトレイト
			pub trait Trigonometric: Sized {
				$( fn $f(self) -> Self; )+
			}

			trig! {
				name(Trigonometric)
				func($($f)+) types($($t),+)
			}

			$(
				#[inline]
				pub fn $f<T: Trigonometric>(x:T) -> T {
					x.$f()
				}
			)+
		}
		pub use trigonometric::{$($f),+};
	};
	( name($n:ident) func($($f:ident)+) types($t0:ty $(,$t:ty)+) ) => {
		trig!{ name($n) func($($f)+) types($t0) }
		trig!{ name($n) func($($f)+) types($($t),+) }
	};
	( name($n:ident) func($($f:ident)+) types($t:ty) ) => {
		impl $n for $t { $(
			#[inline]
			fn $f(self) -> Self { self.$f() }
		)+ }
	};
}
trig!{
	func(sin cos tan sinh cosh tanh asin acos atan asinh acosh atanh)
	types(f64,f32,C<f64>,C<f32>)
}

/// 浮動小数型のみに対応した関数の定義をまとめて行うマクロ
macro_rules! misc {
	( $( $name:ident ( $arg0:ident $(,$args:ident)* ) as $tr:ident )+ ) => {
		/// 浮動小数型のみに対応した細々とした関数を定義するモジュール
		pub(crate) mod float_misc {
			use super::*;

			$(
				#[doc=concat!("`Float` に対して `",stringify!($name),"` を定義するトレイト")]
				pub trait $tr: Float + Sized {
					fn call(self $(,$args:Self)* ) -> Self;
				}
				impl $tr for f64 {
					#[inline]
					fn call(self $(,$args:Self)* ) -> Self {
						self.$name($($args),*)
					}
				}
				impl $tr for f32 {
					#[inline]
					fn call(self $(,$args:Self)* ) -> Self {
						self.$name($($args),*)
					}
				}
				#[inline]
				pub fn $name<T: $tr>($arg0:T $(,$args:T)*) -> T {
					$arg0.call($($args),*)
				}
			)+
		}
		pub use float_misc::{$($name),+};
	};
}
misc! {
	atan2(y,x) as Atan2
	hypot(y,x) as Hypot
	mul_add(a,b,c) as MulAdd
}

/// `clamp` 関数を拡張した形で実装するモジュール
mod clamp {
	use super::*;
	use std::cmp::Ordering;

	/// `Float` と `Complex` に clamp を実装するトレイト
	pub trait Clamp: Sized {
		fn clamp_impl(self,val1:Self,val2:Self) -> Self;
	}

	impl Clamp for f64 {
		fn clamp_impl(self,val1:Self,val2:Self) -> Self {
			if self.is_nan() { return self; }
			match Self::partial_cmp(&val1,&val2) {
				Some(Ordering::Less) => self.clamp(val1,val2),
				Some(Ordering::Greater) => self.clamp(val2,val1),
				Some(Ordering::Equal) => val1,
				None => self
			}
		}
	}
	impl Clamp for f32 {
		fn clamp_impl(self,val1:Self,val2:Self) -> Self {
			if self.is_nan() { return self; }
			match Self::partial_cmp(&val1,&val2) {
				Some(Ordering::Less) => self.clamp(val1,val2),
				Some(Ordering::Greater) => self.clamp(val2,val1),
				Some(Ordering::Equal) => val1,
				None => self
			}
		}
	}
	impl Clamp for C<f64> {
		fn clamp_impl(self,val1:Self,val2:Self) -> Self {
			C {
				re: clamp(self.re,val1.re,val2.re),
				im: clamp(self.im,val1.im,val2.im)
			}
		}
	}
	impl Clamp for C<f32> {
		fn clamp_impl(self,val1:Self,val2:Self) -> Self {
			C {
				re: clamp(self.re,val1.re,val2.re),
				im: clamp(self.im,val1.im,val2.im)
			}
		}
	}

	#[inline]
	pub fn clamp<T: Clamp>(x:T,val1:T,val2:T) -> T {
		x.clamp_impl(val1,val2)
	}

}
pub use clamp::clamp;

/// `power` 関数を定義するモジュール
mod power {
	use super::*;
	use std::{
		ops::Neg,
		num::{NonZeroI32,NonZeroU32}
	};
	use num::pow::pow as pow_usize;

	/// 関数 `power` の引数として受け入れ可能な値の型を定義するトレイト
	pub trait SupportsPowerOf<P> {
		type Result;
		fn power_impl(self,pow:P) -> Self::Result;
	}

	/// `SupportsPowerOf<P,R>` の実装をまとめて行うマクロ
	macro_rules! pow_impl {
		// 型を分類して受け取る
		(recurse
			int: $($i:ident)+,
			bigint: $($b:ident)+,
			int_pow: $($ip:ident)+, // i32 に変換できる i* 型
			uint_as_int_pow: $($uip:ident)+, // i32 に変換できる u* 型
			uint_pow: $($up:ident)+, // u32 に変換できる u* 型
			float: $($f:ident)+
		) => {
			// 分類された実装に渡していく
			pow_impl!{impl(int) base: $($i)+ $($b)+, pow: $($up)+ }
			pow_impl!{impl(float_int) base: $($f)+, pow: $($ip)+ $($uip)+ }
			pow_impl!{impl(float) base: $($f)+ }
			pow_impl!{impl(complex) int_pow: $($ip)+, uint_pow: $($up)+, float: $($f)+ }
			pow_impl!{impl(ratio) base: $($i)+ $($b)+, pow: $($ip)+ $($uip)+ }
			pow_impl!{impl(usize_pow) base: $($i),+, $($b),+, $($f),+, $(R<$i>),+, $(R<$b>),+, $(C<$f>),+ }
			pow_impl!{impl(ref_usize_pow) base: $($i),+ $($f),+, $(R<$i>),+, $(C<$f>),+ }
		};

		(impl(int)
			base: $($b:ident)+,
			pow: $p:ident $($pr:ident)*
		) => {
			$( pow_impl!{each_ref ($b,$p) pow as u32 } )+
			pow_impl!{impl(int) base: $($b)+, pow: $($pr)* }
		};
		(impl(float_int)
			base: $($b:ident)+,
			pow: $p:ident $($pr:ident)*
		) => {
			$( pow_impl!{each_ref ($b,$p) powi as i32 } )+
			pow_impl!{impl(float_int) base: $($b)+, pow: $($pr)* }
		};
		(impl(float) base: $($b:ident)+ ) => {
			$( pow_impl!{each_ref ($b,$b) powf } )+
		};
		(impl(complex)
			int_pow: $($ip:ident)+,
			uint_pow: $($up:ident)+,
			float: $($f:ident)+
		) => {
			$( pow_impl!{each_ref
				(C<T>,$ip)<T,> powi as i32
				where T: Clone + Num + Neg<Output=T>
			} )+
			$( pow_impl!{each_ref
				(C<T>,$up)<T,> powu as u32
				where T: Clone + Num
			} )+
			$(
				pow_impl!{each_ref (C<$f>,$f) powf }
				pow_impl!{each_ref (C<$f>,C<$f>) powc }
			)+
		};
		(impl(ratio)
			base: $($b:ident)+,
			pow: $p:ident $($pr:ident)*
		) => {
			$( pow_impl!{each_ref (R<$b>,$p) pow as i32 } )+
			pow_impl!{impl(ratio) base: $($b)+, pow: $($pr)* }
		};
		(impl(usize_pow) base: $($b:ty),+ ) => { $(
			pow_impl!{each
				(self:$b,pow:usize) -> $b { pow_usize(self,pow) }
			}
			pow_impl!{each
				(self:$b,pow:&'p usize)<'p,> -> $b { pow_usize(self,*pow) }
			}
		)+ };
		(impl(ref_usize_pow) base: $($b:ty),+ ) => { $(
			pow_impl!{each
				(self:&'b $b,pow:usize)<'b,> -> $b { pow_usize(*self,pow) }
			}
			pow_impl!{each
				(self:&'b $b,pow:&'p usize)<'b,'p,> -> $b { pow_usize(*self,*pow) }
			}
		)+ };
		(impl($im:ident) $($others:tt)+ ) => {};

		(each_ref
			($b:ty,$p:ty) $cmd:ident
		) => {
			pow_impl!{each
				(self:$b,pow:$p) -> $b { self.$cmd(pow) }
			}
			pow_impl!{each
				(self:&'b $b,pow:$p)<'b,> -> $b { self.$cmd(pow) }
			}
			pow_impl!{each
				(self:$b,pow:&'p $p)<'p,> -> $b { self.$cmd(*pow) }
			}
			pow_impl!{each
				(self:&'b $b,pow:&'p $p)<'b,'p,> -> $b { self.$cmd(*pow) }
			}
		};
		(each_ref
			($b:ty,$p:ty)
			$( <$($gl:lifetime,)* $($gt:ident,)*> )?
			$cmd:ident as $d:ty
			$( where $($w:tt)+ )?
		) => {
			pow_impl!{each
				(self:$b,pow:$p)$(<$($gl,)* $($gt,)*>)? -> $b { self.$cmd( <$d as From<$p>>::from(pow) ) }
				$( where $($w)+ )?
			}
			pow_impl!{each
				(self:&'b $b,pow:$p)<'b,$($($gl,)* $($gt,)*)?> -> $b { self.$cmd( <$d as From<$p>>::from(pow) ) }
				$( where $($w)+ )?
			}
			pow_impl!{each
				(self:$b,pow:&'p $p)<'p,$($($gl,)* $($gt,)*)?> -> $b { self.$cmd( <$d as From<$p>>::from(*pow) ) }
				$( where $($w)+ )?
			}
			pow_impl!{each
				(self:&'b $b,pow:&'p $p)<'b,'p,$($($gl,)* $($gt,)*)?> -> $b { self.$cmd( <$d as From<$p>>::from(*pow) ) }
				$( where $($w)+ )?
			}
		};
		(each
			($s:ident:$bt:ty,$p:ident:$pt:ty)
			$( <$($gl:lifetime,)* $($gt:ident,)*> )?
			-> $rt:ty
			{ $($body:tt)+ }
			$( where $($w:tt)+ )?
		) => {
			impl $(<$($gl,)* $($gt,)*>)? SupportsPowerOf<$pt> for $bt $( where $($w)+ )?
			{
				type Result = $rt;
				#[inline]
				fn power_impl($s,$p:$pt) -> Self::Result { $($body)+ }
			}
		}
	}

	pow_impl! { recurse
		int: i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize,
		bigint: BigInt BigUint,
		int_pow: i8 i16 i32 NonZeroI32,
		uint_as_int_pow: u8 u16,
		uint_pow: u8 u16 u32 NonZeroU32,
		float: f32 f64
	}

	#[inline]
	pub fn power<B,P>(base:B,pow:P) -> <B as SupportsPowerOf<P>>::Result
	where B: SupportsPowerOf<P> {
		//! ## `power`
		//! 冪乗を計算します。 `.pow()`, `.powf()`, `.powi()` など多様な冪乗の関数を一元化し、型に合わせた関数を呼び出すように実装されています。
		//! ### 対応する型
		//! ```rust
		//! power(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|BigInt|BigUint,u8|u16|u32|usize)
		//! power(f32|f64,i8|i16|i32|u8|u16|usize)
		//! power(f32,f32) power(f64,f64)
		//! power(Complex<f32>,f32|Complex<f32>)
		//! power(Complex<f64>,f64|Complex<f64>)
		//! power(Complex<f32|f64>,usize)
		//! power(Complex<T>,u8|u16|u32)
		//! power(Complex<T>,i8|i16|i32|u8|u16)
		//! ```

		base.power_impl(pow)
	}

}
pub use power::power;
