use super::*;
extern crate once_cell;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use FloatCategory as Cat;

mod gamma {
	use super::*;

	pub trait GammaFunc {
		/// ガンマ関数を計算します
		fn gamma(self) -> Self;
	}

	impl GammaFunc for f64 {
		fn gamma(self) -> Self {

			// 予め決まった値になるものを取り除く
			match self.categorize() {
				Cat::NaN => { return f64::nan(); },
				Cat::PositiveInfinity => { return f64::infinity(); },
				Cat::NegativeInfinity => { return f64::nan(); },
				_ => {}
			}

			// 0以下の整数の場合は NaN になる
			if self.is_integer() && self<0.5 { return f64::nan(); }

			// 整数の場合は別途用意する

			// 入力が負の値の場合は反射律を使って正の値に直してから計算する
			if self.is_sign_negative() {
				use std::f64::consts::PI;
				let lg = log_gamma_stirling_series(1.0-self);
				return PI / lg.exp() / sin(PI*self);
			}

			log_gamma_stirling_series(self).exp()

		}
	}

	/// Stirling 級数により値を計算するモジュール
	mod stirling_series {
		use super::*;

		/// ln(sqrt(2π))
		static LN_SQRT_2_PI:Lazy<f64> = Lazy::new(|| {
			use std::f64::consts::TAU;
			TAU.sqrt().ln()
		});

		/// 1より大きな正の実数に対して Stirling 級数を使って log(Γ(x)) を計算
		pub fn calc_series(x:f64) -> f64 {
			let mut value = (*LN_SQRT_2_PI) + (x-0.5) * x.ln() - x;

			// 変化が無視できるくらい小さくなるまで足していく
			for n in 1..=MAX_COUNT {
				let diff = COEFFS_STOCK[n-1] / x.powi((2*n-1) as i32);
				if (diff/value).abs() < f64::EPSILON*100.0 { break }
				value += diff;
			}

			value
		}

		/// 最大でこの次数まで Stirling 級数の係数を近似させる (オーバーフローしない程度に)
		const MAX_COUNT:usize = 10;

		/// log(Γ(x)) を計算するための Stirling 級数の係数をストック
		static COEFFS_STOCK:Lazy<[f64;MAX_COUNT]> = Lazy::new(|| {
			let mut a = [0_f64;MAX_COUNT];
			for n in (1..=MAX_COUNT).rev() {
				a[n-1] = bernoulli_number(2*n).to_f64().unwrap() / ((2*n*(2*n-1)) as f64);
			}
			a
		});

	}
	use stirling_series::calc_series as log_gamma_stirling_series;

	#[cfg(test)]
	#[test]
	fn gamma_func_test() {
		let a = [1.0,1.5,2.0,3.0,3.14,5.0,10.0];
		for v in a {
			println!("Γ({}) = {}",v,v.gamma());
		}
	}

}
pub use gamma::*;

/// 整数に関連する値を計算するモジュール
mod integer_coefficients {
	use super::*;
	use num::{rational::Ratio,BigInt};
	type RU = Ratio<usize>;
	type RI = BigRational;

	/// RI 型の分数を生成
	fn ratio(num:isize,den:isize) -> RI {
		RI::new(
			BigInt::from(num),
			BigInt::from(den)
		)
	}

	/// 2項係数を計算します
	pub fn binomial_coefficient(n:usize,mut k:usize) -> usize {
		// k>n の場合はブロック
		if k>n { panic!("k>n の引数が与えられました"); }
		// (n,k) よりも (n,n-k) の方が計算しやすければ、そちらを利用する
		k.min_assign(n-k);
		let v =
		std::iter::zip(
			((n-k+1)..=n).rev(),
			(1..=k).rev()
		)
		.fold(
			RU::one(),
			|a,(num,den)| a * RU::new(num,den)
		);
		if !v.is_integer() { panic!("2項係数の計算に失敗しました"); }
		v.to_integer()
	}

	/// 多項係数を計算します
	pub fn multinomial_coefficients(params:impl IntoIterator<Item=usize> + Clone) -> usize {
		let sum = params.clone().into_iter().sum();

		let mut iter = params.into_iter()
		.filter( |p| *p>0 ).peekable();
		let first_op = iter.next();
		let first = match (first_op,iter.peek()) {
			(Some(f),Some(_)) => f,
			_ => { return 1; }
		};

		let v =
		std::iter::zip(
			(1..=sum).skip(first),
			iter.map( |n| 1..=n )
			.flatten()
		)
		.fold(
			RU::one(),
			|a,(num,den)| a * RU::new(num,den)
		);
		if !v.is_integer() { panic!("多項係数の計算に失敗しました"); }
		v.to_integer()
	}

	/// 計算したベルヌーイ数のデータをストックする配列
	static BERNOULLI_NUMBER:Lazy<Mutex<Vec<RI>>> = Lazy::new(|| Mutex::new(vec![
		RI::one(),ratio(1,6)
	]));

	/// ベルヌーイ数を計算します
	pub fn bernoulli_number(n:usize) -> RI {

		match n {
			0 => { return RI::one(); },
			1 => { return ratio(-1,2); },
			n if n%2==1 => { return RI::zero(); },
			_ => {}
		}

		let e = n/2;

		let mut list = BERNOULLI_NUMBER.lock().unwrap();

		// ストックになければ値を計算する
		let c = list.len()-1;
		if e>c {
			list.reserve(e-c);
			for n_half in (c+1)..(e+1) {
				let n = n_half*2;
				let b = ratio(-1,(n+1) as isize) *
				list.iter().enumerate()
				.map(|(k_half,b)| {
					let k = k_half*2;
					let c = RI::from_usize(binomial_coefficient(n+1,k)).unwrap();
					c * b.clone()
				})
				.fold(
					ratio(-(binomial_coefficient(n+1,1) as isize),2),
					|a,b| a+b
				);
				list.push(b);
			}
		}

		// 値を取り出す
		list[e].clone()
	}

	#[cfg(test)]
	#[test]
	fn bernoulli_number_test() {
		for n in [64] {
			println!("B_{} = {}",n,bernoulli_number(n));
		}
	}

}
pub use integer_coefficients::*;
