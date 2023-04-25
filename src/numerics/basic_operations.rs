use super::*;



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
pub use operate_and_assign::*;



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

	impl<I:Iter<f64>> FloatMinMax<f64> for I {
		fn minimum(self) -> f64 {
			reduce_ignore_nan(self,min)
		}
		fn maximum(self) -> f64 {
			reduce_ignore_nan(self,max)
		}
		fn minimum_propagate(self) -> f64 {
			reduce_propagate_nan(self,min)
		}
		fn maximum_propagate(self) -> f64 {
			reduce_propagate_nan(self,max)
		}
	}
	impl<I:Iter<f32>> FloatMinMax<f32> for I {
		fn minimum(self) -> f32 {
			reduce_ignore_nan(self,min)
		}
		fn maximum(self) -> f32 {
			reduce_ignore_nan(self,max)
		}
		fn minimum_propagate(self) -> f32 {
			reduce_propagate_nan(self,min)
		}
		fn maximum_propagate(self) -> f32 {
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
