//! ## `gamma_functions`
//! このモジュールではガンマ関数を定義しています

use super::*;
use numerics::primitive_functions::*;

// ガンマ関数のインターフェースを実装する

/// ガンマ関数を対応する型に対して、ガンマ関数を定義するためのトレイト
/// Cephes と SpecialFunctions.jl の実装を参考に実装
pub trait Gamma {
	/// ガンマ関数
	fn gamma(self) -> Self;
	/// 対数ガンマ関数
	fn log_gamma(self) -> Self;
}

/// 入力した値に対するガンマ関数を計算します。
#[inline]
pub fn gamma<T:Gamma>(x:T) -> T {
	x.gamma()
}

/// 入力した値に対する対数ガンマ関数を計算します。
#[inline]
pub fn log_gamma<T:Gamma>(x:T) -> T {
	x.log_gamma()
}



// 以下ではガンマ関数の内容を実装する

/// ガンマ関数の実装を行うモジュール。
/// ガンマ関数は Stirling 近似により求められるが、実際に Stirling 近似を行うのは `stirling_series` である。
mod gamma_func_impl {
	use super::*;

	/// この値よりも小さい場合は、この値を超えるまで値をシフトさせてから、スターリング近似を適用する
	pub const MIN_STIRLING: u8 = 7;

	/// ガンマ関数の値をチェックせずに計算を進める内部向けトレイト。
	/// `Gamma` トレイトで定義した `gamma` などの関数は特殊な値のチェックを行い、先にその値の場合を処理した上で、一般的な値の場合をこのトレイトの `gamma_unchecked` などの関数で処理する。
	trait GammaUnchecked {
		/// ガンマ関数 (値チェックなし)
		fn gamma_unchecked(self) -> Self;
		/// 対数ガンマ関数 (値チェックなし)
		fn log_gamma_unchecked(self) -> Self;
	}

	/// 複数の浮動小数型に対してトレイト実装をまとめて用意するマクロ
	/// 浮動小数の型を抽象化したコードを用意した上で、型を代入して実装する
	macro_rules! gamma_impl {
		// f_type には浮動小数の型を与える
		( $($f_type:ident)+ ) => { $(

			/// 実数向けのガンマ関数のインターフェース
			impl Gamma for $f_type {

				fn gamma(self) -> Self {

					// 予め決まった値になるものを取り除く
					match self.categorize() {
						Cat::NaN => { return $f_type::nan(); },
						Cat::PositiveInfinity => { return $f_type::infinity(); },
						Cat::NegativeInfinity => { return $f_type::nan(); },
						_ => {}
					}

					// 0以下の整数の場合は NaN になる
					if self.is_integer() && self<0.5 { return $f_type::nan(); }

					// 整数の場合は別途用意する

					// 特殊な値を上でチェックしたので、チェックなしで計算を進める
					self.gamma_unchecked()

				}

				fn log_gamma(self) -> Self {

					// 予め決まった値になるものを取り除く
					match self.categorize() {
						Cat::NaN => { return $f_type::nan(); },
						Cat::PositiveInfinity => { return $f_type::infinity(); },
						Cat::NegativeInfinity => { return $f_type::nan(); },
						_ => {}
					}

					// 0以下の整数の場合は NaN になる
					if self.is_integer() && self<0.5 { return $f_type::nan(); }

					// 特殊な値を上でチェックしたので、チェックなしで計算を進める
					self.log_gamma_unchecked()

				}

			}

			/// 複素数向けのガンマ関数のインターフェース
			impl Gamma for Complex<$f_type> {
				fn gamma(self) -> Self {
					self.log_gamma().exp()
				}
				fn log_gamma(self) -> Self {
					use std::$f_type::consts::PI;
					type C = Complex<$f_type>; // 複素数型の定義

					// 実部と虚部を分解する
					let C { re: x, im: y } = self;

					// 予め決まった値になるものを取り除く
					match (x.categorize(),y.categorize()) {
						(Cat::NaN,_)|(_,Cat::NaN) =>
							return C { re: $f_type::nan(), im: $f_type::nan() },
						(Cat::PositiveInfinity,Cat::Positive) =>
							return C { re: x, im: $f_type::infinity() },
						(Cat::PositiveInfinity,Cat::Negative) =>
							return C { re: x, im: $f_type::neg_infinity() },
						(Cat::NegativeInfinity,Cat::Positive) =>
							return C { re: x, im: $f_type::neg_infinity() },
						(Cat::NegativeInfinity,Cat::Negative) =>
							return C { re: x, im: $f_type::infinity() },
						(Cat::PositiveInfinity|Cat::NegativeInfinity,Cat::ZeroPositive|Cat::ZeroNegative) =>
							return self,
						(Cat::ZeroPositive,Cat::ZeroPositive) =>
							return C { re: $f_type::infinity(), im: -0.0 },
						(Cat::ZeroPositive,Cat::ZeroNegative) =>
							return C { re: $f_type::infinity(), im:  0.0 },
						(Cat::ZeroNegative,Cat::ZeroPositive) =>
							return C { re: $f_type::infinity(), im: -PI  },
						(Cat::ZeroNegative,Cat::ZeroNegative) =>
							return C { re: $f_type::infinity(), im:  PI  },
						(_,Cat::PositiveInfinity|Cat::NegativeInfinity) =>
							return C { re: $f_type::neg_infinity(), im: y },
						_ => {}
					}

					// 特殊な値を上でチェックしたので、チェックなしで計算を進める
					self.log_gamma_unchecked()

				}

			}

			/// 実数向けの値チェック済みのガンマ関数の実装
			impl GammaUnchecked for $f_type {

				fn gamma_unchecked(self) -> Self {

					let mut z = self;

					// 入力が負の値の場合は反射律を使って正の値に直してから計算する
					if self.is_sign_negative() {
						use std::$f_type::consts::PI;
						return PI / (1.0-z).gamma_unchecked() / sin(PI*z);
					}

					// 入力が規定よりも小さい値であれば、大きくしてスターリング近似を適用する
					if z < (MIN_STIRLING as $f_type) {
						// スターリング近似で得た値から割る値を求める
						let mut div:$f_type = 1.0;
						while z < (MIN_STIRLING as $f_type) {
							div *= z;
							z += 1.0;
						}
						return z.gamma_unchecked() / div;
					}

					// スターリング近似により計算
					self.stirling_series().exp()

				}

				fn log_gamma_unchecked(self) -> Self {

					let mut z = self;

					// 入力が負の値の場合は反射律を使って正の値に直してから計算する
					if self.is_sign_negative() {
						use std::$f_type::consts::PI;
						return ln(PI) - (1.0-z).log_gamma_unchecked() - ln(sin(PI*z));
					}

					// 入力が規定よりも小さい値であれば、大きくしてスターリング近似を適用する
					if z < (MIN_STIRLING as $f_type) {
						// スターリング近似で得た値から割る値を求める
						let mut sub:$f_type = 0.0;
						while z < (MIN_STIRLING as $f_type) {
							sub += ln(z);
							z += 1.0;
						}
						return z.log_gamma_unchecked() - sub;
					}

					// スターリング近似により計算
					self.stirling_series()

				}

			}

			/// 複素数向けの値チェック済みのガンマ関数の実装
			impl GammaUnchecked for Complex<$f_type> {

				fn gamma_unchecked(self) -> Self {
					exp(self.log_gamma_unchecked())
				}

				fn log_gamma_unchecked(self) -> Self {
					use std::$f_type::consts::{PI,TAU};
					type C = Complex<$f_type>; // 複素数型の定義

					// 入力が負の値の場合は反射律を使って正の値に直してから計算する
					if self.re < 0.1 {
						// 実部が ln(π) で、虚部は適切なブランチが選ばれるようにする
						let pi_complex = C::new(ln(PI),TAU.copysign(self.im)*(0.5*self.re+0.25));
						// 反射律の適用
						return pi_complex - (1.0-self).log_gamma_unchecked() - ln(sin(PI*self));
					}

					// 入力の実部や虚部が十分大きければそのままスターリング近似を適用する
					if self.re >= (MIN_STIRLING as $f_type) || abs(self.im) >= (MIN_STIRLING as $f_type) {
						return self.stirling_series()
					}

					// 以下では入力の実部の値をシフトさせることにより、スターリング近似で計算できるようにする
					// 合わせて、出力もシフトさせる
					// 対数ガンマ関数なので、適切な位相ブランチを選ぶアルゴリズムを組み込んでいる

					// 入力の z の虚部を正の値にとったもの
					let mut z = C::new(self.re,abs(self.im));
					// 出力シフトの位相を回転させる回数
					let mut turn: u8 = 0;
					// 出力シフトの指数
					let mut shift_exp = z.clone();
					// 早速出力シフトがあったので、入力もシフトする
					z.re += 1.0;
					// 入力をシフトさせ続ける
					while z.re <= (MIN_STIRLING as $f_type) {
						// 掛ける前に shift_exp が負であるかどうか
						let is_neg_before = shift_exp.im.is_sign_negative();
						// 出力をシフトするために掛ける
						shift_exp *= z;
						// 掛けた後に shift_exp が負であるかどうか
						let is_neg_after = shift_exp.im.is_sign_negative();
						// 掛けたことにより虚部の符号が負から正に変わったのであれば、1回転したとみなす
						if !is_neg_after && is_neg_before {
							turn += 1;
						}
						// 入力のシフトを実行
						z.re += 1.0;
					}
					// 出力シフトを得る
					let mut shift = ln(shift_exp);
					// 位相回転が含まれている場合、ここで実行する
					if turn>0 {
						if z.im.is_sign_negative() {
							shift.im = - (turn as $f_type)*TAU - shift.im;
						} else {
							shift.im += (turn as $f_type)*TAU;
						}
					}

					// シフトさせた上で返す
					z.stirling_series() - shift

				}

			}

		)+ };
	}
	gamma_impl!( f32 f64 );

}

/// Stirling 級数により log(Γ(x)) の値を計算するモジュール
mod stirling_series {
	use super::*;

	/// 最大でこの次数まで Stirling 級数の係数を近似させる (オーバーフローしない程度に)
	const MAX_COUNT:usize = 10;

	/// Stirling 級数を計算する関数の実装するトレイト
	pub(super) trait StirlingSeries: Sized {
		/// 実部が1より十分大きな数に対して Stirling 級数を使って log(Γ(x)) を計算
		fn stirling_series(self) -> Self;
	}

	/// StirlingSeries をまとめて実装するマクロ
	macro_rules! ss {

		// 浮動小数の型ごとに、実数版と複素数版をまとめて定義する
		( floats:
			$( (
				$float:tt, // 実数型
				$ln_sqrt_2_pi:ident, // LN_SQRT_2_PI の変数名
				$coeffs_stock:ident, // COEFFS_STOCK の変数名
				$to_float:ident, // to_f64 のような実数型に変換する関数名
				$zero:literal // 0 を表す値
			) )+
		) => { $(

			/// ln(sqrt(2π)) の定数
			static $ln_sqrt_2_pi:Lazy<$float> = Lazy::new(|| {
				use std::$float::consts::TAU;
				TAU.sqrt().ln()
			});

			/// log(Γ(x)) を計算するための Stirling 級数の係数をストック
			static $coeffs_stock:Lazy<[$float;MAX_COUNT]> = Lazy::new(|| {
				// 空の配列を用意
				let mut a = [$zero;MAX_COUNT];
				// 配列のそれぞれの要素に級数の係数を格納
				for n in (1..=MAX_COUNT).rev() {
					// a[n-1]: x^(-2n+1) 次の係数
					// B_{2n} / 2n(2n-1)
					a[n-1] = bernoulli_number(2*n).$to_float().unwrap() / ((2*n*(2*n-1)) as $float);
				}
				// 配列を返す
				a
			});

			// 実数型と複素数型の StirlingSeries の実装を行う
			ss!{ ss_impl:
				$float, $float, abs,
				$ln_sqrt_2_pi, $coeffs_stock
			}
			ss!{ ss_impl:
				Complex<$float>, $float, norm,
				$ln_sqrt_2_pi, $coeffs_stock
			}

		)+ };

		// それぞれ個別の型に対する実装を行う
		( ss_impl:
			$raw_type:ty, // 型
			$real_type:ty, // 実数の型
			$norm:ident, // 規格化の関数名
			$ln_sqrt_2_pi:ident, // LN_SQRT_2_PI の変数名
			$coeffs_stock:ident // COEFFS_STOCK の変数名
		) => {
			impl StirlingSeries for $raw_type {
				fn stirling_series(self) -> Self {
					// 級数以外の部分: (x-1/2)ln(x) - x
					let mut value = (*$ln_sqrt_2_pi) + (self-0.5) * self.ln() - self;

					// 変化が無視できるくらい小さくなるまで足していく
					for n in 1..=MAX_COUNT {
						let diff = $coeffs_stock[n-1] / self.powi((2*n-1) as i32);
						if (diff/value).$norm() < <$real_type>::EPSILON*100.0 { break }
						value += diff;
					}

					value
				}
			}
		};

	}

	// f64, Complex<f64>, f32, Complex<f32> に対してまとめて実装
	ss!{ floats:
		( f64, LN_SQRT_2_PI_F64, COEFFS_STOCK_F64, to_f64, 0_f64 )
		( f32, LN_SQRT_2_PI_F32, COEFFS_STOCK_F32, to_f32, 0_f32 )
	}

	#[cfg(test)]
	#[test]
	/// 係数の計算結果が適切かどうか確認する
	fn coeffs_test() {
		let c = *LN_SQRT_2_PI_F64;
		println!("ln(sqrt(2π)): {c:.e}");
		for (n,value) in (1..=MAX_COUNT).zip(COEFFS_STOCK_F64.iter()) {
			println!("coeff {n:2}: {value:.e}");
		}
	}

}
use stirling_series::*;



// 実装したガンマ関数の値をテストする

#[cfg(test)]
#[test]
/// ガンマ関数が適切に動作するかテストする関数
fn gamma_func_test() {
	let a = [1.0,1.5,2.0,3.0,3.14,5.0,10.0];
	for v in a {
		println!("Γ({}) = {} or {}",v,gamma(v),gamma(Complex::new(v,0.0)));
	}
}