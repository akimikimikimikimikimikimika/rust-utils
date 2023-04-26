use super::*;



/// 拡張した数値を分類するモジュール
mod categorize {
	use super::*;

	/// 浮動小数のカテゴリを示す型
	pub enum FloatCategory {
		NaN,
		ZeroPositive, ZeroNegative,
		Positive, Negative,
		PositiveSubnormal, NegativeSubnormal,
		PositiveInfinity, NegativeInfinity,
	}

	pub trait FloatCategorize {
		/// 浮動小数の分離を提供します
		fn categorize(&self) -> FloatCategory;
		/// 浮動小数が整数であるか判定します
		fn is_integer(&self) -> bool;
	}
	impl<T: Float> FloatCategorize for T {
		fn categorize(&self) -> FloatCategory {
			type C = FloatCategory;
			if self.is_nan() { return C::NaN; }
			use std::num::FpCategory as CB;
			match (self.is_sign_positive(),self.classify()) {
				(_    ,CB::Nan      ) => C::NaN,
				(true ,CB::Zero     ) => C::ZeroPositive,
				(false,CB::Zero     ) => C::ZeroNegative,
				(true ,CB::Normal   ) => C::Positive,
				(false,CB::Normal   ) => C::Negative,
				(true ,CB::Subnormal) => C::PositiveSubnormal,
				(false,CB::Subnormal) => C::NegativeSubnormal,
				(true ,CB::Infinite ) => C::PositiveInfinity,
				(false,CB::Infinite ) => C::NegativeInfinity,
			}
		}
		fn is_integer(&self) -> bool {
			( *self % T::one() ).is_zero()
		}
	}

}
pub use categorize::*;



/// 浮動小数を丸めるメソッドを提供するモジュール
mod rounding {
	use super::*;

	compose_struct! {
		#[derive(Debug,Clone,Copy,PartialEq,Eq)]
		#[pub_all]
		/// ## `FloatRounding`
		/// * 浮動小数を丸めるシステム
		/// * 構造体のフィールドによりオプションを指定して、 .doit() で丸めます。
		/// * デフォルトの設定を使う項目は `..Default::default()` で省略します。
		struct Rounding<T: Float> {
			/// 丸める対象の値 (`f32`, `f64`, `num::Float` 型)
			value: T,
			/// 丸め方を指定します
			strategy = enum Strategy {
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
				ToNearestOrTowardInfinity = default,
				/// 現在の値から最も近い整数に丸め、隣接する整数が同程度に近い場合は偶数になる方を選びます
				ToNearestOrEven,
				/// 現在の値から最も近い整数に丸め、隣接する整数が同程度に近い場合は奇数になる方を選びます
				ToNearestOrOdd,
			},
			/// 10進数において丸める位を指定します
			digit: i32
		}
	}

	impl<T: Float> Default for Rounding<T> {
		fn default() -> Self {
			Self {
				value: T::zero(),
				strategy: Strategy::ToNearestOrTowardInfinity,
				digit: 0
			}
		}
	}

	impl<T> Rounding<T> where T: Float, f32: Into<T> {
		pub fn doit(&self) -> T {

			if self.value.is_nan() { return self.value; }

			let mut x = self.value;

			if self.digit!=0 { x = x * 10.0.into().powi(self.digit); }

			let is_positive = x.is_sign_positive();

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
					[ $strategy => x.$func() ]
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
									is_positive,
									$( x % r!(@rem $rem) )?
								) )
							)
							y() z($($sub_arms)+)
						)
					]
					$($not_yet)*
				) };
				// 全ての丸め方のパースが終了したら呼び出す
				(@arm $([ $arm:ident => $($content:tt)+ ])+ ) => {
					match self.strategy {
						$( Strategy::$arm => $($content)+ ),+
					}
				};

				// 剰余の値にマッチ
				(@rem 1) => { 10.0.into() };
				(@rem 2) => { 20.0.into() };

				// 以下は1つの丸め方に対して条件分岐がある場合を処理している
				// x: パース済 y: パース中 z: 未パース

				// 1つのアームを全てパースし切った後に、 match のパターンを生成
				(@m
					x( rem($rem:tt) input($($input:tt)+) $($x:tt)* )
					y( sign($sp:tt) op($($op:tt)+) threshold($t:literal) func($func:ident) )
					$($z:tt)+
				) => { r!(@m
					x(
						rem($rem) input($($input)+) $($x)*
						pattern(($sp,r)) condition(r $($op)+ $t.into()) func($func)
					)
					y() $($z)+
				) };
				(@m
					x( rem($rem:tt) input($($input:tt)+) $($x:tt)* )
					y( sign($sp:tt) func($func:ident) )
					$($z:tt)+
				) => { r!(@m
					x(
						rem($rem) input($($input)+) $($x)*
						pattern(($sp,_)) func($func)
					)
					y() $($z)+
				) };
				(@m
					x( input($($input:tt)+) $($x:tt)* )
					y( sign($sp:tt) func($func:ident) )
					$($z:tt)+
				) => { r!(@m
					x(
						input($($input)+) $($x)*
						pattern(($sp,)) func($func)
					)
					y() $($z)+
				) };

				// 正負の符号にマッチ
				(@m x($($x:tt)+) y() z(+ $($z:tt)+) ) => {
					r!(@m x($($x)+) y( sign(true ) ) z($($z)+) )
				};
				(@m x($($x:tt)+) y() z(- $($z:tt)+) ) => {
					r!(@m x($($x)+) y( sign(false) ) z($($z)+) )
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
				(@m x($($x:tt)+) y($($y:tt)+) z($threshold:literal $($z:tt)+) ) => {
					r!(@m x($($x)+) y($($y)+ threshold($threshold) ) z($($z)+) )
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
						$( $($p)+ $( if $($c)+ )? => x.$func() ,)+
					}
				};

			}

			// 丸め方に合わせて条件分岐
			// 正でも負でもない値は NaN である
			x = r! { match {
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
					+ >  0.5 => ceil
					+        => floor
					- > -0.5 => ceil
					-        => floor
				}
				ToNearestOrUp => % 1 {
					+ <  0.5 => floor
					+        => ceil
					- < -0.5 => floor
					-        => ceil
				}
				ToNearestOrTowardZero => % 1 {
					+ >  0.5 => ceil
					+        => floor
					- < -0.5 => floor
					-        => ceil
				}
				ToNearestOrTowardInfinity => % 1 {
					+ <  0.5 => floor
					+        => ceil
					- > -0.5 => ceil
					-        => floor
				}
				ToNearestOrEven => % 2 {
					+ <=  0.5 => floor
					+ <=  1.0 => ceil
					+ <   1.5 => floor
					+         => ceil
					- >= -0.5 => ceil
					- >= -1.0 => floor
					- >  -1.5 => ceil
					-         => floor
				}
				ToNearestOrOdd => % 2 {
					+ <   0.5 => floor
					+ <=  1.0 => ceil
					+ <=  1.5 => floor
					+         => ceil
					- >  -0.5 => ceil
					- >= -1.0 => floor
					- >= -1.5 => ceil
					-         => floor
				}
			} };

			if self.digit!=0 { x = x / 10.0.into().powi(self.digit); }

			x

		}
	}

	#[cfg(test)]
	#[test]
	/// 丸める処理が適切に動作するかテストする
	fn test_rounding() {

		macro_rules! test_items {
			( Input: $($iv:literal)+ $( $case:ident: $($rc:literal)+ )+ ) => {
				( [$($iv),+], [ $( (Strategy::$case,[$($rc),+]) ),+ ] )
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
		for (s,exp) in expected {
			for (i,e) in std::iter::zip(input.iter(),exp.into_iter()) {
				let c = Rounding {
					value: *i, strategy: s,
					..Default::default()
				}.doit();
				let mut b = c==e;
				// 0.0 と -0.0 を区別する
				if b && e==0.0 { b = c.is_sign_negative()==e.is_sign_negative(); }
				if !b {
					failed.push(format!(
						"({:+}).rounding(Strategy::{:?}) = {:+} != {:+}",
						i,s,c,e
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
pub use rounding::{
	Rounding as FloatRounding,
	Strategy as FloatRoundingStrategy
};
