use super::*;

/// `Float`, `Complex` の双方に対してプリミティブな関数をまとめて定義するマクロ
macro_rules! func_def {
	( $(
		$func_impl:ident($($extra_args:ident),*)
		=> $func_as:ident($arg0:ident $(,$args:ident)*)
	)+ ) => {
		mod func_generic {
			use super::*;

			pub trait FloatOrComplex { $(
				fn $func_impl(&self $(,$extra_args:Self)*) -> Self;
			)+ }

			func_def!{@impl FloatOrComplex for [f64,f32,Complex<f64>,Complex<f32>] { $(
				#[inline]
				fn $func_impl(&self $(,$extra_args:Self)*) -> Self {
					self.$func_as($($extra_args),*)
				}
			)+ } }

		}
		use func_generic::FloatOrComplex;

		$(
			#[inline]
			pub fn $func_as<T: FloatOrComplex>( $arg0:T $(,$args:T)* ) -> T {
				$arg0.$func_impl($($args),*)
			}
		)+
	};
	(@impl $f_or_c:ident for [$t0:ty $(,$t:ty)*] { $($body:tt)+ }) => {
		impl $f_or_c for $t0 { $($body)+ }
		func_def!{@impl $f_or_c for [$($t),*] { $($body)+ }}
	};
	(@impl $f_or_c:ident for [] {$($body:tt)+}) => {};
}

func_def! {
	sqrt_impl()       => sqrt(x)
	cbrt_impl()       => cbrt(x)
	exp_impl()        => exp(x)
	ln_impl()         => ln(x)
	sin_impl()        => sin(x)
	cos_impl()        => cos(x)
	tan_impl()        => tan(x)
	sinh_impl()       => sinh(x)
	cosh_impl()       => cosh(x)
	tanh_impl()       => tanh(x)
	asin_impl()       => asin(x)
	acos_impl()       => acos(x)
	atan_impl()       => atan(x)
	asinh_impl()      => asinh(x)
	acosh_impl()      => acosh(x)
	atanh_impl()      => atanh(x)
}

#[inline]
pub fn log2<F:Float>(x:F) -> F {
	x.log2()
}

#[inline]
pub fn log10<F:Float>(x:F) -> F {
	x.log10()
}

#[inline]
pub fn atan2<F:Float>(y:F,x:F) -> F {
	y.atan2(x)
}

#[inline]
pub fn hypot<F:Float>(y:F,x:F) -> F {
	y.hypot(x)
}

#[inline]
pub fn abs_sub<F:Float>(x1:F,x2:F) -> F {
	x1.abs_sub(x2)
}

#[inline]
pub fn mul_add<F:Float>(a:F,b:F,c:F) -> F {
	a.mul_add(b,c)
}

/// `power` 関数を定義するモジュール
mod power {
	use super::*;
	use std::{
		ops::Neg,
		num::{NonZeroI32,NonZeroU32}
	};
	use num::pow::pow as pow_usize;
	type R<T> = num::rational::Ratio<T>;
	type C<T> = Complex<T>;

	/// 関数 `power` の引数として受け入れ可能な値の型を定義しています
	pub trait SupportsPowerOf<P,R> {
		fn power_impl(self,pow:P) -> R;
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
			impl $(<$($gl,)* $($gt,)*>)? SupportsPowerOf<$pt,$rt> for $bt $( where $($w)+ )?
			{
				#[inline]
				fn power_impl($s,$p:$pt) -> $rt { $($body)+ }
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
	/// ## `power`
	/// 冪乗を計算します
	/// ### 対応する型
	/// ```rust
	/// power(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|BigInt|BigUint,u8|u16|u32|usize)
	/// power(f32|f64,i8|i16|i32|u8|u16|usize)
	/// power(f32,f32) power(f64,f64)
	/// power(Complex<f32>,f32|Complex<f32>)
	/// power(Complex<f64>,f64|Complex<f64>)
	/// power(Complex<f32|f64>,usize)
	/// power(Complex<T>,u8|u16|u32)
	/// power(Complex<T>,i8|i16|i32|u8|u16)
	/// ```
	pub fn power<B,P,R>(base:B,pow:P) -> R
	where B: SupportsPowerOf<P,R> {
		base.power_impl(pow)
	}

}
pub use power::power;
