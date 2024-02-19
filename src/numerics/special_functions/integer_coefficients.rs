//! ## `integer_coefficients`
//! このモジュールでは整数の係数に用いられる数列を定義しています

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