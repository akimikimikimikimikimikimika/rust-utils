use super::*;



/// 標準の複合代入演算子と同様の機能を幾つかの演算に適用する
pub mod operate_and_assign {
	/// ブール値の演算子の複合代入版
	pub trait BoolAssign {
		fn and_assign(&mut self,rhs:Self);
		fn or_assign(&mut self,rhs:Self);
		fn not(&mut self);
	}
	impl BoolAssign for bool {
		fn and_assign(&mut self,rhs:Self) {
			*self = (*self) && rhs
		}
		fn or_assign(&mut self,rhs:Self) {
			*self = (*self) || rhs
		}
		fn not(&mut self) {
			*self = ! (*self)
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
	impl MinMaxAssignForFloat for f32 {
		fn min_assign(&mut self,rhs:Self) {
			*self = (*self).min(rhs);
		}
		fn max_assign(&mut self,rhs:Self) {
			*self = (*self).max(rhs);
		}
	}
	impl MinMaxAssignForFloat for f64 {
		fn min_assign(&mut self,rhs:Self) {
			*self = (*self).min(rhs);
		}
		fn max_assign(&mut self,rhs:Self) {
			*self = (*self).max(rhs);
		}
	}

}



/// `Ord` に従う型の最大/最小を多数の要素に対して実行する
pub mod min_max {

	/// 最大/最小に関するトレイトをまとめて定義する
	macro_rules! trait_def {
		( $($name:ident)+ ) => { $(
			/// 複数の要素の中から最大/最小を決定する
			pub trait $name<T> {
				/// 複数の要素の中から最小の値を計算します
				fn minimum(self) -> T;
				/// 複数の要素の中から最大の値を計算します
				fn maximum(self) -> T;
			}
		)+ };
	}
	trait_def!( MinMaxArray MinMaxTuple );

	impl<T,I> MinMaxArray<T> for I
	where I: IntoIterator<Item=T>, T: Ord
	{
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

	/// * タプル `(T,T,...)` の各要素に対して、最大/最小を決定するトレイト `MinMaxTuple` の実装をまとめて行うマクロ
	/// * `implement!(indices: 1 2 ... (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! implement {
		(indices: $($i:tt)+ ) => {
			mod impl_min_max {
				use super::*;
				use crate::numerics::basic_operations::min_max::*;

				implement! {@each T | $($i),+ }
			}
		};
		(@each $t:ident $($tx:ident $x:tt),* | $y0:tt $(,$y:tt)* ) => {
			implement! {@each $t $($tx $x),* | }
			implement! {@each $t $($tx $x,)* $t $y0 | $($y),* }
		};
		(@each $t:ident $($tx:ident $x:tt),* | ) => {
			impl<$t> MinMaxTuple<$t> for ($t,$($tx),*) where $t: Ord {
				fn minimum(self) -> $t {
					self.0 $( .min(self.$x) )*
				}
				fn maximum(self) -> $t {
					self.0 $( .max(self.$x) )*
				}
			}
		};
	}
	pub(crate) use implement;

}



#[cfg(feature="numerics")]
/// `Float` に従う型の最大/最小を多数の要素でも使えるようにする。 NaN の伝播の仕方に合わせて複数のメソッドを用意する。
pub mod float_min_max {
	use super::*;

	compose_struct! {
		trait Iter<T> = IntoIterator<Item=T> where T: ?Sized;
		trait ReduceFn<T> = Fn(T,T) -> T;
	}

	pub trait FloatMinMax<T> {
		/// 複数の浮動小数の中から最小の値を与えます。値に NaN が含まれていれば無視されます。全ての値が NaN の場合や値が含まれていない場合は NaN を返します。
		fn minimum(self) -> T;
		/// 複数の浮動小数の中から最大の値を与えます。値に NaN が含まれていれば無視されます。全ての値が NaN の場合や値が含まれていない場合は NaN を返します。
		fn maximum(self) -> T;
		/// 複数の浮動小数の中から最小の値を与えます。値のうちどれか1つでも NaN がある場合や値が含まれていない場合 NaN を返します。
		fn minimum_propagate(self) -> T;
		/// 複数の浮動小数の中から最大の値を与えます。値のうちどれか1つでも NaN がある場合や値が含まれていない場合 NaN を返します。
		fn maximum_propagate(self) -> T;
	}

	// ここで IntoIterator<Item=T> の代わりに Iter<T> を使うことはできない。 MinMax トレイトと衝突してしまう。
	impl<T,I> FloatMinMax<T> for I where I: IntoIterator<Item=T>, T: Float {
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
/// ある型に対して取りうる最大/最小の値を得る。
pub mod maximum_minimum {
	use num::Bounded;

	/// 指定した型で取りうる最大の値を返します
	pub fn maximum_value<T>() -> T where T: Bounded {
		T::max_value()
	}
	/// 指定した型で取りうる最小の値を返します
	pub fn minimum_value<T>() -> T where T: Bounded {
		T::min_value()
	}
}



/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::{
		operate_and_assign::{
			BoolAssign,
			MinMaxAssignForOrd,
			MinMaxAssignForFloat
		},
		min_max::{ MinMaxArray, MinMaxTuple }
	};
	#[cfg(feature="numerics")]
	pub use super::{
		float_min_max::FloatMinMax,
		maximum_minimum::{
			maximum_value, minimum_value
		}
	};
}
