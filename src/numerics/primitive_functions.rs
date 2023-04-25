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
