use super::*;

#[cfg(feature="numerics")]
extern crate num;
#[cfg(feature="numerics")]
pub use num::*;



#[cfg(feature="numerics")]
/// プリミティブな関数を `x.sin()` ではなく `sin(x)` のような表記で使えるようにする
pub mod primitive_functions {
	use super::*;

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

	pub fn sinh<F:Float>(x:F) -> F {
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
/// `hypot` 関数を多数の要素でも使えるようにする
mod hypot {
	use super::*;

	compose_struct! {
		pub trait Iter<T> = IntoIterator<Item=T>;
	}

	pub fn hypot<F:Float>(x:F,y:F) -> F {
		return y.hypot(x);
	}

	pub trait HypotFn<T> {
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
#[cfg(feature="numerics")]
pub use hypot::*;



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
	#[cfg(feature="numerics")]
	impl<T:Float> MinMaxAssignForFloat for T {
		fn min_assign(&mut self,rhs:Self) {
			*self = (*self).min(rhs);
		}
		fn max_assign(&mut self,rhs:Self) {
			*self = (*self).max(rhs);
		}
	}

}
// pub use operate_and_assign::*;



/// `Ord` に従う型の最大/最小を多数の要素に対して実行する
mod min_max {
	use super::*;

	compose_struct! {
		pub trait Iter<T> = IntoIterator<Item=T>;
	}

	/// 複数の要素の中から最大/最小を決定する
	pub trait MinMax<T> {
		/// 複数の要素の中から最小の値を計算します
		fn minimum(self) -> T;
		/// 複数の要素の中から最大の値を計算します
		fn maximum(self) -> T;
	}

	impl<T:Ord,I:Iter<T>> MinMax<T> for I {
		fn minimum(self) -> T {
			self.into_iter()
			.reduce( |a,v| a.min(v) )
			.expect("minimizing empty slice is not allowed")
		}
		fn maximum(self) -> T {
			self.into_iter()
			.reduce( |a,v: T| a.max(v) )
			.expect("maximizing empty slice is not allowed")
		}
	}

}
pub use min_max::*;



#[cfg(feature="numerics")]
/// `Float` に従う型の最大/最小を多数の要素でも使えるようにする。同時に NaN の伝播則をカスタマイズできるようにする。
mod float_min_max {
	use super::*;

	compose_struct! {
		pub trait Iter<T> = IntoIterator<Item=T>;
		pub trait ReduceFn<T> = Fn(T,T) -> T;
	}

	pub trait MinMax<T> {
		/// 複数の浮動小数の中から最小の値を与えます。値に NaN が含まれていれば無視されます。全ての値が NaN の場合や値が含まれていない場合は NaN を返します。
		fn minimum(self) -> T;
		/// 複数の浮動小数の中から最大の値を与えます。値に NaN が含まれていれば無視されます。全ての値が NaN の場合や値が含まれていない場合は NaN を返します。
		fn maximum(self) -> T;
		/// 複数の浮動小数の中から最小の値を与えます。値のうちどれか1つでも NaN がある場合や値が含まれていない場合 NaN を返します。
		fn minimum_propagate(self) -> T;
		/// 複数の浮動小数の中から最大の値を与えます。値のうちどれか1つでも NaN がある場合や値が含まれていない場合 NaN を返します。
		fn maximum_propagate(self) -> T;
	}

	impl<T:Float,I:Iter<T>> MinMax<T> for I {
		fn minimum(self) -> T {
			reduce_ignore_nan(self,min)
		}
		fn maximum(self) -> T {
			reduce_ignore_nan(self,max)
		}
		fn minimum_propagate(self) -> T {
			reduce_propagate_nan(self,min)
		}
		fn maximum_propagate(self) -> T {
			reduce_propagate_nan(self,max)
		}
	}

	/// 最小値
	fn min<T:Float>(a:T,v:T) -> T { a.min(v) }
	/// 最大値
	fn max<T:Float>(a:T,v:T) -> T { a.max(v) }

	/// 複数の要素に対する処理を実装。 NaN は無視する。
	fn reduce_ignore_nan<T:Float>(ii:impl Iter<T>,f:impl ReduceFn<T>) -> T {
		ii.into_iter()
		.filter(|v| !v.is_nan())
		.reduce(f)
		.unwrap_or(T::nan())
	}

	/// 複数の要素に対する処理を実装。 NaN は伝播する。
	fn reduce_propagate_nan<T: Float>(ii:impl Iter<T>,f:impl ReduceFn<T>) -> T {
		let mut iter = ii.into_iter();
		let mut m = match iter.next() {
			None => { return T::nan(); },
			Some(v) if v.is_nan() => { return T::nan(); },
			Some(v) => v,
		};
		for v in iter {
			if v.is_nan() { return T::nan(); }
			m = f(m,v);
		}
		m
	}

}
#[cfg(feature="numerics")]
pub use float_min_max::*;



#[cfg(feature="numerics")]
/// `Float` 型を幾つかの丸め方のルールに従って丸められるようにする。
mod float_rounding {
	use super::*;

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

	/// 浮動小数に丸めるメソッドを実装するトレイト
	pub trait Rounding {
		/// 指定した丸め方で浮動小数を丸めます
		fn rounding(&self,rule:R) -> Self;
		/// 指定した丸め方で浮動小数を丸めます。丸める桁数も指定できます。
		fn rounding_with_precision(&self,rule:R,precision:i32) -> Self;
	}

	impl<T:F> Rounding for T {

		fn rounding(&self,rule:R) -> Self {

			let positive = self.is_sign_positive();
			let negative = self.is_sign_negative();

			/// * それぞれの丸め方ごとに floor ceil のルールを決めるマクロ
			/// * 例
			/// ```rust
			/// r! {
			/// 	Down => floor // 丸め方 Down は無条件に floor
			/// 	TowardZero => % 2 { // 丸め方 TowardZero の条件を指定
			/// 		// 2で割った余りによって分岐 ( % n を省略すると余りによる分岐は行いません)
			/// 		+ >= 1.5 => ceil  // 正の値で、余りが 1.5 より大きい場合は ceil 関数を適用
			/// 		-        => floor // 余りによる分岐は省略可能 (負の値は floor になる)
			/// 	}
			/// }
			/// ```
			macro_rules! r {

				// 入力 -> それぞれの丸め方ごとに処理する
				( match { $($any:tt)+ } ) => {
					r!(@arm $($any)+ )
				};

				// 丸め方の1つ: 無条件に floor/ceil が決まる場合
				(@arm
					$( [$($parsed:tt)+] )*
					$strategy:ident => $func:ident
					$($not_yet:tt)*
				) => { r!(@arm
					$( [$($parsed)+] )*
					[ $strategy => self.$func() ]
					$($not_yet)*
				) };
				// 丸め方の1つ: 条件分岐がある場合
				(@arm
					$( [$($parsed:tt)+] )*
					$strategy:ident => $( % $rem:tt )? { $($sub_arms:tt)+ }
					$($not_yet:tt)*
				) => { r!(@arm
					$( [$($parsed)+] )*
					[
						$strategy => r!(@m
							x(
								$( rem($rem) )?
								input( (
									positive, negative,
									$( (*self) % r!(@rem $rem) )?
								) )
							)
							y() z($($sub_arms)+)
						)
					]
					$($not_yet)*
				) };
				// 全ての丸め方のパースが終了したら呼び出す
				(@arm $([ $arm:ident => $($content:tt)+ ])+ ) => {
					match rule {
						$( R::$arm => $($content)+ ),+
					}
				};

				// 剰余の値にマッチ
				(@rem 1) => { Self::val_p10() };
				(@rem 2) => { Self::val_p20() };

				// 以下は1つの丸め方に対して条件分岐がある場合を処理している
				// x: パース済 y: パース中 z: 未パース

				// 1つのアームを全てパースし切った後に、 match のパターンを生成
				(@m
					x( rem($rem:tt) input($($input:tt)+) $($x:tt)* )
					y( sign($sp:tt,$sn:tt) op($($op:tt)+) threshold($t:ident) func($func:ident) )
					$($z:tt)+
				) => { r!(@m
					x(
						rem($rem) input($($input)+) $($x)*
						pattern(($sp,$sn,r)) condition(r $($op)+ Self::$t()) func($func)
					)
					y() $($z)+
				) };
				(@m
					x( rem($rem:tt) input($($input:tt)+) $($x:tt)* )
					y( sign($sp:tt,$sn:tt) func($func:ident) )
					$($z:tt)+
				) => { r!(@m
					x(
						rem($rem) input($($input)+) $($x)*
						pattern(($sp,$sn,_)) func($func)
					)
					y() $($z)+
				) };
				(@m
					x( input($($input:tt)+) $($x:tt)* )
					y( sign($sp:tt,$sn:tt) func($func:ident) )
					$($z:tt)+
				) => { r!(@m
					x(
						input($($input)+) $($x)*
						pattern(($sp,$sn)) func($func)
					)
					y() $($z)+
				) };

				// 正負の符号にマッチ
				(@m x($($x:tt)+) y() z(+ $($z:tt)+) ) => {
					r!(@m x($($x)+) y( sign(true ,false) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y() z(- $($z:tt)+) ) => {
					r!(@m x($($x)+) y( sign(false,true ) ) z($($z)+) )
				};

				// 条件式の不等号にマッチ
				(@m x($($x:tt)+) y($($y:tt)+) z(>= $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ op(>=) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y($($y:tt)+) z(<= $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ op(<=) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y($($y:tt)+) z(> $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ op(>) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y($($y:tt)+) z(< $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ op(<) ) z($($z)+) )
				};

				// 条件式の境界値にマッチ
				(@m x($($x:tt)+) y($($y:tt)+) z(+0.5 $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ threshold(val_p05) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y($($y:tt)+) z(+1.0 $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ threshold(val_p10) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y($($y:tt)+) z(+1.5 $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ threshold(val_p15) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y($($y:tt)+) z(-0.5 $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ threshold(val_m05) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y($($y:tt)+) z(-1.0 $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ threshold(val_m10) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y($($y:tt)+) z(-1.5 $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ threshold(val_m15) ) z($($z)+) )
				};

				// アームの対応関数 (floor/ceil) にマッチ
				(@m x($($x:tt)+) y($($y:tt)+) z(=> $func:ident $($z:tt)*) ) => {
					r!(@m x($($x)+) y($($y)+ func($func) ) z($($z)*) )
				};

				// 全てのアームを処理した後に match 式を生成
				(@m
					x(
						$( rem($rem:tt) )? input($($input:tt)+)
						$( pattern($($p:tt)+) $( condition($($c:tt)+) )? func($func:ident) )+
					)
					y() z()
				) => {
					match $($input)+ {
						$( $($p)+ $( if $($c)+ )? => self.$func() ,)+
						_ => *self
					}
				};

			}

			// 丸め方に合わせて条件分岐
			// 正でも負でもない値は NaN である
			r! { match {
				Down => floor
				Up   => ceil
				TowardZero => {
					+ => floor
					- => ceil
				}
				TowardInfinity => {
					+ => ceil
					- => floor
				}
				ToNearestOrDown => % 1 {
					+ > +0.5 => ceil
					+        => floor
					- > -0.5 => ceil
					-        => floor
				}
				ToNearestOrUp => % 1 {
					+ < +0.5 => floor
					+        => ceil
					- < -0.5 => floor
					-        => ceil
				}
				ToNearestOrTowardZero => % 1 {
					+ > +0.5 => ceil
					+        => floor
					- < -0.5 => floor
					-        => ceil
				}
				ToNearestOrTowardInfinity => % 1 {
					+ < +0.5 => floor
					+        => ceil
					- > -0.5 => ceil
					-        => floor
				}
				ToNearestOrEven => % 2 {
					+ <= +0.5 => floor
					+ <= +1.0 => ceil
					+ <  +1.5 => floor
					+         => ceil
					- >= -0.5 => ceil
					- >= -1.0 => floor
					- >  -1.5 => ceil
					-         => floor
				}
				ToNearestOrOdd => % 2 {
					+ <  +0.5 => floor
					+ <= +1.0 => ceil
					+ <= +1.5 => floor
					+         => ceil
					- >  -0.5 => ceil
					- >= -1.0 => floor
					- >= -1.5 => ceil
					-         => floor
				}
			} }

		}

		fn rounding_with_precision(&self,rule:R,precision:i32) -> Self {
			(
				(*self) * Self::pow10(precision)
			).rounding(rule)
			/ Self::pow10(precision)
		}

	}

	/// 型ジェネリックに floor/ceil 分岐の境界値を与えるトレイト
	trait F: Float {

		fn val_p05() -> Self;
		fn val_p10() -> Self;
		fn val_p15() -> Self;
		fn val_p20() -> Self;
		fn val_m05() -> Self;
		fn val_m10() -> Self;
		fn val_m15() -> Self;
		fn val_m20() -> Self;

		/// 10^p
		fn pow10(p:i32) -> Self;

	}
	/// トレイト `F` の実装を与えるマクロ
	macro_rules! float_impl {
		($fxx:ty) => {
			impl F for $fxx {
				#[inline]
				fn val_p05() -> $fxx {  0.5 }
				#[inline]
				fn val_p10() -> $fxx {  1.0 }
				#[inline]
				fn val_p15() -> $fxx {  1.5 }
				#[inline]
				fn val_p20() -> $fxx {  2.0 }
				#[inline]
				fn val_m05() -> $fxx { -0.5 }
				#[inline]
				fn val_m10() -> $fxx { -1.0 }
				#[inline]
				fn val_m15() -> $fxx { -1.5 }
				#[inline]
				fn val_m20() -> $fxx { -2.0 }
				#[inline]
				fn pow10(p:i32) -> $fxx { 10.0.powi(p) }
			}
		};
	}
	float_impl!(f32);
	float_impl!(f64);

	#[cfg(test)]
	#[test]
	/// 丸める処理が適切に動作するかテストする
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
pub use float_rounding::FloatRoundingRule;



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
