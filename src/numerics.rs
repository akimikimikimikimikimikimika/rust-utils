#[cfg(feature="numerics")]
extern crate num;
#[cfg(feature="numerics")]
pub use num::{
	one,zero,
	Float,Integer,Signed,Unsigned
};

#[cfg(feature="numerics")]
/// プリミティブな関数をドット演算子を使わない一般的な表記で使えるようにする
mod primitive_funcs {
	use super::*;

	pub fn abs<S:Signed>(x:S) -> S {
		return x.abs();
	}

	pub fn exp<F:Float>(x:F) -> F {
		return x.exp();
	}

	pub fn log<F:Float>(x:F) -> F {
		return x.ln();
	}

	pub fn sin<F:Float>(x:F) -> F {
		return x.sin();
	}

	pub fn cos<F:Float>(x:F) -> F {
		return x.cos();
	}

	pub fn tan<F:Float>(x:F) -> F {
		return x.tan();
	}

	pub fn sinh(x:f64) -> f64 {
		return x.sinh();
	}

	pub fn cosh<F:Float>(x:F) -> F {
		return x.cosh();
	}

	pub fn tanh<F:Float>(x:F) -> F {
		return x.tanh();
	}

	pub fn atan2<F:Float>(y:F,x:F) -> F {
		return y.atan2(x);
	}

}
#[cfg(feature="numerics")]
pub use primitive_funcs::*;



#[cfg(feature="numerics")]
/// `hypot` 関数を多数の要素でも使えるようにする
mod hypot {
	use super::*;

	pub fn hypot<F:Float>(x:F,y:F) -> F {
		return y.hypot(x);
	}

	pub trait HypotFn<T> {
		fn hypot(&self) -> T;
	}
	impl<T:Float,const N:usize> HypotFn<T> for [T;N] {
		fn hypot(&self) -> T {
			match N {
				0 => T::zero(),
				1 => self[0],
				2 => self[0].hypot(self[1]),
				_ => [self[0],self[1..].hypot()].hypot()
			}
		}
	}
	impl<T:Float> HypotFn<T> for [T] {
		fn hypot(&self) -> T {
			match self.len() {
				0 => T::zero(),
				1 => self[0],
				2 => self[0].hypot(self[1]),
				_ => [self[0],self[1..].hypot()].hypot()
			}
		}
	}
	impl<T:Float> HypotFn<T> for (T,T) {
		fn hypot(&self) -> T {
			self.0.hypot(self.1)
		}
	}
	impl<T:Float> HypotFn<T> for (T,T,T) {
		fn hypot(&self) -> T {
			self.0.hypot(self.1).hypot(self.2)
		}
	}

}
#[cfg(feature="numerics")]
pub use hypot::*;



#[cfg(feature="numerics")]
/// 標準の複合代入演算子と同様の機能を幾つかの演算に適用する
mod operate_and_assign {
	use super::*;

	/// ブール値の and/or 演算子の複合代入版
	pub trait AndOrAssign {
		fn and_assign(&mut self,rhs:Self);
		fn or_assign(&mut self,rhs:Self);
	}
	impl AndOrAssign for bool {
		fn and_assign(&mut self,rhs:Self) {
			*self = (*self) && rhs
		}
		fn or_assign(&mut self,rhs:Self) {
			*self = (*self) || rhs
		}
	}

	// 以下では最大/最小の複合代入演算子を定義しているが、 `Ord` と `Float` であえて別のトレイトにしている。そうしないとコンフリクトが発生するから。

	/// 最大/最小にも複合代入演算子を用意する
	pub trait MinMaxAssignForOrd {
		/// もう一方の値と比較し、小さい方を代入する
		fn min_assign(&mut self,rhs:Self);
		/// もう一方の値と比較し、大きい方を代入する
		fn max_assign(&mut self,rhs:Self);
	}
	impl<T:Ord+Copy> MinMaxAssignForOrd for T {
		fn min_assign(&mut self,rhs:Self) {
			*self = (*self).min(rhs);
		}
		fn max_assign(&mut self,rhs:Self) {
			*self = (*self).max(rhs);
		}
	}

	/// 最大/最小にも複合代入演算子を用意する
	pub trait MinMaxAssignForFloat {
		/// もう一方の値と比較し、小さい方を代入する
		fn min_assign(&mut self,rhs:Self);
		/// もう一方の値と比較し、大きい方を代入する
		fn max_assign(&mut self,rhs:Self);
	}
	impl<T:Float> MinMaxAssignForFloat for T {
		fn min_assign(&mut self,rhs:Self) {
			*self = (*self).min(rhs);
		}
		fn max_assign(&mut self,rhs:Self) {
			*self = (*self).max(rhs);
		}
	}

}
#[cfg(feature="numerics")]
pub use operate_and_assign::*;



/// `Ord` に従う型の最大/最小を多数の要素でも使えるようにする
mod order_min_max {

	/// 配列の中から最大/最小を決定する
	pub trait MinMaxFns<T> {
		/// 配列の中から最小の値を計算します
		fn minimum(&self) -> T;
		/// 配列の中から最大の値を計算します
		fn maximum(&self) -> T;
	}

	impl<T:Ord+Copy> MinMaxFns<T> for [T] {
		fn minimum(&self) -> T {
			match self.len() {
				0 => { panic!("minimizing empty slice is not allowed"); }
				1 => self[0],
				2 => self[0].min(self[1]),
				_ => self[0].min(self[1..].minimum())
			}
		}
		fn maximum(&self) -> T {
			match self.len() {
				0 => { panic!("maximizing empty slice is not allowed"); }
				1 => self[0],
				2 => self[0].max(self[1]),
				_ => self[0].max(self[1..].maximum())
			}
		}
	}

	impl<T:Ord+Copy,const N:usize> MinMaxFns<T> for [T;N] {
		fn minimum(&self) -> T {
			match N {
				0 => { panic!("minimizing empty slice is not allowed"); }
				1 => self[0],
				2 => self[0].min(self[1]),
				_ => self[0].min(self[1..].minimum())
			}
		}
		fn maximum(&self) -> T {
			match self.len() {
				0 => { panic!("maximizing empty slice is not allowed"); }
				1 => self[0],
				2 => self[0].max(self[1]),
				_ => self[0].max(self[1..].maximum())
			}
		}
	}

	impl<T:Ord+Copy> MinMaxFns<T> for Vec<T> {
		fn minimum(&self) -> T {
			self.as_slice().minimum()
		}
		fn maximum(&self) -> T {
			self.as_slice().maximum()
		}
	}

}
pub use order_min_max::*;



#[cfg(feature="numerics")]
/// `Float` に従う型の最大/最小を多数の要素でも使えるようにする。同時に NaN の伝播則をカスタマイズできるようにする。
mod float_min_max {
	use super::*;

	#[derive(Clone,Copy,PartialEq,Eq)]
	/// 浮動小数型の演算の規則を定義します
	pub enum NaNRule {
		/// 2つの値のうち一方に NaN が含まれている場合は NaN が返されます
		Propagate,
		/// 2つの値のうち一方が NaN の場合は他方の値を返し、両方が NaN の場合のみ NaN を返します
		Ignore,
	}

	type N = NaNRule;

	pub trait MinMaxFloat<T> {
		/// 複数の浮動小数の中から最小の値を与えます。 `rule` には NaN の取り扱い方を指定します。
		fn minimum(self,rule:N) -> T;
		/// 複数の浮動小数の中から最大の値を与えます。 `rule` には NaN の取り扱い方を指定します。
		fn maximum(self,rule:N) -> T;
	}

	impl<T:Float> MinMaxFloat<T> for (T,T) {
		fn minimum(self,rule:N) -> T {
			match (self.0.is_nan(),self.1.is_nan(),rule) {
				(false,false,_) => self.0.min(self.1),
				(false,true,N::Ignore) => self.0,
				(true,false,N::Ignore) => self.1,
				(true,_,N::Propagate)|(_,true,N::Propagate)|(true,true,N::Ignore) => T::nan()
			}
		}
		fn maximum(self,rule:N) -> T {
			match (self.0.is_nan(),self.1.is_nan(),rule) {
				(false,false,_) => self.0.max(self.1),
				(false,true,N::Ignore) => self.0,
				(true,false,N::Ignore) => self.1,
				(true,_,N::Propagate)|(_,true,N::Propagate)|(true,true,N::Ignore) => T::nan()
			}
		}
	}

}
#[cfg(feature="numerics")]
pub use float_min_max::*;



#[cfg(feature="numerics")]
/// `Float` 型を幾つかの丸め方のルールに従って丸められるようにする。
mod float_rounding {

	#[derive(Debug,Clone,Copy,PartialEq,Eq)]
	/// 浮動小数の丸め方を指定します
	pub enum FloatRoundingRule {
		/// 現在の値より小さくて最も近い整数に丸めます (`floor` と同じ)
		Down,
		/// 現在の値より大きくて最も近い整数に丸めます (`ceil` と同じ)
		Up,
		/// 現在の値から0に近づく方向で最も近い整数に丸めます (`trunc` と同じ)
		TowardZero,
		/// 現在の値から0から離れる方向で最も近い整数に丸めます
		TowardInfinity,
		/// 現在の値から最も近い整数に丸め、隣接する整数が同程度に近い場合は小さい方を選びます
		ToNearestOrDown,
		/// 現在の値から最も近い整数に丸め、隣接する整数が同程度に近い場合は大きい方を選びます
		ToNearestOrUp,
		/// 現在の値から最も近い整数に丸め、隣接する整数が同程度に近い場合は0に近い方を選びます
		ToNearestOrTowardZero,
		/// 現在の値から最も近い整数に丸め、隣接する整数が同程度に近い場合は0に遠い方を選びます (`round` と同じ)
		ToNearestOrTowardInfinity,
		/// 現在の値から最も近い整数に丸め、隣接する整数が同程度に近い場合は偶数になる方を選びます
		ToNearestOrEven,
		/// 現在の値から最も近い整数に丸め、隣接する整数が同程度に近い場合は奇数になる方を選びます
		ToNearestOrOdd,
	}
	type R = FloatRoundingRule;

	pub trait Rounding {
		/// 指定した丸め方で浮動小数を丸めます
		fn rounding(&self,rule:R) -> Self;
		/// 指定した丸め方で浮動小数を丸めます。丸める桁数も指定できます。
		fn rounding_with_precision(&self,rule:R,precision:i32) -> Self;
	}

	trait RoundingInternal {
		fn toward_zero(&self) -> Self;
		fn toward_infinity(&self) -> Self;
		fn to_nearest_or_down(&self) -> Self;
		fn to_nearest_or_up(&self) -> Self;
		fn to_nearest_or_toward_zero(&self) -> Self;
		fn to_nearest_or_toward_infinity(&self) -> Self;
		fn to_nearest_or_even(&self) -> Self;
		fn to_nearest_or_odd(&self) -> Self;
	}
	use std::num::FpCategory as C;
	impl RoundingInternal for f64 {

		#[inline]
		fn toward_zero(&self) -> Self {
			match (self.is_sign_positive(),self.is_sign_negative()) {
				(true,false) => self.floor(),
				(false,true) => self.ceil(),
				_ => *self
			}
		}

		#[inline]
		fn toward_infinity(&self) -> Self {
			match (self.is_sign_positive(),self.is_sign_negative()) {
				(true,false) => self.ceil(),
				(false,true) => self.floor(),
				_ => *self
			}
		}

		#[inline]
		fn to_nearest_or_down(&self) -> Self {
			if !matches!(self.classify(),C::Normal|C::Subnormal) { return *self; }
			match (self.is_sign_positive(),(*self)%1.0) {
				(true,r) if r > 0.5 => self.ceil(),
				(true,_) => self.floor(),
				(false,r) if r > -0.5 => self.ceil(),
				(false,_) => self.floor()
			}
		}

		#[inline]
		fn to_nearest_or_up(&self) -> Self {
			if !matches!(self.classify(),C::Normal|C::Subnormal) { return *self; }
			match (self.is_sign_positive(),(*self)%1.0) {
				(true,r) if r < 0.5 => self.floor(),
				(true,_) => self.ceil(),
				(false,r) if r < -0.5 => self.floor(),
				(false,_) => self.ceil()
			}
		}

		#[inline]
		fn to_nearest_or_toward_zero(&self) -> Self {
			if !matches!(self.classify(),C::Normal|C::Subnormal) { return *self; }
			match (self.is_sign_positive(),(*self)%1.0) {
				(true,r) if r > 0.5 => self.ceil(),
				(true,_) => self.floor(),
				(false,r) if r < -0.5 => self.floor(),
				(false,_) => self.ceil()
			}
		}

		#[inline]
		fn to_nearest_or_toward_infinity(&self) -> Self {
			if !matches!(self.classify(),C::Normal|C::Subnormal) { return *self; }
			match (self.is_sign_positive(),(*self)%1.0) {
				(true,r) if r < 0.5 => self.floor(),
				(true,_) => self.ceil(),
				(false,r) if r > -0.5 => self.ceil(),
				(false,_) => self.floor()
			}
		}

		#[inline]
		fn to_nearest_or_even(&self) -> Self {
			if !matches!(self.classify(),C::Normal|C::Subnormal) { return *self; }
			match (self.is_sign_positive(),(*self)%2.0) {
				(true,r) if r <= 0.5 => self.floor(),
				(true,r) if r <= 1.0 => self.ceil(),
				(true,r) if r <  1.5 => self.floor(),
				(true,_) => self.ceil(),
				(false,r) if r >= -0.5 => self.ceil(),
				(false,r) if r >= -1.0 => self.floor(),
				(false,r) if r >  -1.5 => self.ceil(),
				(false,_) => self.floor(),
			}
		}

		#[inline]
		fn to_nearest_or_odd(&self) -> Self {
			if !matches!(self.classify(),C::Normal|C::Subnormal) { return *self; }
			match (self.is_sign_positive(),(*self)%2.0) {
				(true,r) if r <  0.5 => self.floor(),
				(true,r) if r <= 1.0 => self.ceil(),
				(true,r) if r <= 1.5 => self.floor(),
				(true,_) => self.ceil(),
				(false,r) if r >  -0.5 => self.ceil(),
				(false,r) if r >= -1.0 => self.floor(),
				(false,r) if r >= -1.5 => self.ceil(),
				(false,_) => self.floor(),
			}
		}

	}

	impl Rounding for f64 {

		fn rounding(&self,rule:R) -> Self {
			match rule {
				R::Down => self.floor(),
				R::Up => self.ceil(),
				R::TowardZero => self.toward_zero(),
				R::TowardInfinity => self.toward_infinity(),
				R::ToNearestOrDown => self.to_nearest_or_down(),
				R::ToNearestOrUp => self.to_nearest_or_up(),
				R::ToNearestOrTowardZero => self.to_nearest_or_toward_zero(),
				R::ToNearestOrTowardInfinity => self.to_nearest_or_toward_infinity(),
				R::ToNearestOrEven => self.to_nearest_or_even(),
				R::ToNearestOrOdd => self.to_nearest_or_odd(),
			}
		}

		fn rounding_with_precision(&self,rule:R,precision:i32) -> Self {
			( (*self) * 10_f64.powi(precision) ).rounding(rule) / 10_f64.powi(precision)
		}

	}

	#[cfg(test)]
	#[test]
	fn test_rounding() {
		macro_rules! test_items {
			( Input: $($iv:literal)+ $( $case:ident: $($rc:literal)+ )+ ) => {
				( [$($iv),+], [ $( (R::$case,[$($rc),+]) ),+ ] )
			};
		}

		// See refs: https://en.wikipedia.org/wiki/Rounding#Comparison_of_approaches_for_rounding_to_an_integer
		let (input,expected) = test_items! {
			Input:
				-1.8 -1.5 -1.2 -1.0 -0.8 -0.5 -0.2 -0.0 0.0 0.2 0.5 0.8 1.0 1.2 1.5 1.8
			Down:
				-2.0 -2.0 -2.0 -1.0 -1.0 -1.0 -1.0 -0.0 0.0 0.0 0.0 0.0 1.0 1.0 1.0 1.0
			Up:
				-1.0 -1.0 -1.0 -1.0 -0.0 -0.0 -0.0 -0.0 0.0 1.0 1.0 1.0 1.0 2.0 2.0 2.0
			TowardZero:
				-1.0 -1.0 -1.0 -1.0 -0.0 -0.0 -0.0 -0.0 0.0 0.0 0.0 0.0 1.0 1.0 1.0 1.0
			TowardInfinity:
				-2.0 -2.0 -2.0 -1.0 -1.0 -1.0 -1.0 -0.0 0.0 1.0 1.0 1.0 1.0 2.0 2.0 2.0
			ToNearestOrDown:
				-2.0 -2.0 -1.0 -1.0 -1.0 -1.0 -0.0 -0.0 0.0 0.0 0.0 1.0 1.0 1.0 1.0 2.0
			ToNearestOrUp:
				-2.0 -1.0 -1.0 -1.0 -1.0 -0.0 -0.0 -0.0 0.0 0.0 1.0 1.0 1.0 1.0 2.0 2.0
			ToNearestOrTowardZero:
				-2.0 -1.0 -1.0 -1.0 -1.0 -0.0 -0.0 -0.0 0.0 0.0 0.0 1.0 1.0 1.0 1.0 2.0
			ToNearestOrTowardInfinity:
				-2.0 -2.0 -1.0 -1.0 -1.0 -1.0 -0.0 -0.0 0.0 0.0 1.0 1.0 1.0 1.0 2.0 2.0
			ToNearestOrEven:
				-2.0 -2.0 -1.0 -1.0 -1.0 -0.0 -0.0 -0.0 0.0 0.0 0.0 1.0 1.0 1.0 2.0 2.0
			ToNearestOrOdd:
				-2.0 -1.0 -1.0 -1.0 -1.0 -1.0 -0.0 -0.0 0.0 0.0 1.0 1.0 1.0 1.0 1.0 2.0
		};

		let mut failed:Vec<String> = vec![];
		for (r,exp) in expected {
			for (i,e) in std::iter::zip(input.iter(),exp.into_iter()) {
				let c = i.rounding(r);
				let mut b = c==e;
				// 0.0 と -0.0 を区別する
				if b && e==0.0 { b = c.is_sign_negative()==e.is_sign_negative(); }
				if !b {
					failed.push(format!(
						"({:+}).rounding(R::{:?}) = {:+} != {:+}",
						i,r,c,e
					));
				}
			}
		}
		if failed.len()>0 {
			let src = format!(
				"以下のテストに失敗しました\n{}",
				failed.join("\n")
			);
			panic!("{}",src);
		}

	}

}
#[cfg(feature="numerics")]
pub use float_rounding::*;



#[cfg(feature="numerics")]
/// ある型に対して取りうる最大/最小の値を得る。
mod maximum_minimum {
	/// 指定した型で取りうる最大の値を返します
	pub fn maximum_value<T>() -> T where T: num::Bounded {
		T::max_value()
	}
	/// 指定した型で取りうる最小の値を返します
	pub fn minimum_value<T>() -> T where T: num::Bounded {
		T::min_value()
	}
}
#[cfg(feature="numerics")]
pub use maximum_minimum::*;
