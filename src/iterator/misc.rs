use super::*;



/// 有限回のみ繰り返すイテレータを生成するモジュール
pub mod cycle_n {
	use super::compose_struct;

	compose_struct! {
		pub trait ICS = Iterator + Clone + Sized;
	}

	pub trait IteratorCycleNExtension<I: ICS> {
		/// 有限回のみ繰り返すイテレータを生成する
		fn cycle_n(self,repeat:usize) -> CycleN<I>;
	}

	impl<I: ICS> IteratorCycleNExtension<I> for I {
		fn cycle_n(self,repeat:usize) -> CycleN<I> {
			CycleN { iterator: self.clone(), original: self, whole_count: repeat, current_count: repeat }
		}
	}

	/// 有限回のみ繰り返すイテレータ
	#[derive(Clone)]
	pub struct CycleN<I: ICS> {
		original: I,
		iterator: I,
		whole_count: usize,
		current_count: usize
	}

	impl<I: ICS> Iterator for CycleN<I> {

		type Item = I::Item;

		#[inline]
		fn next(&mut self) -> Option<Self::Item> {
			match (self.iterator.next(),self.current_count) {
				(_,0) => None,
				(None,1) => None,
				(None,_) => {
					self.current_count -= 1;
					self.iterator = self.original.clone();
					self.iterator.next()
				},
				(s,_) => s
			}
		}

		#[inline]
		fn size_hint(&self) -> (usize, Option<usize>) {
			match (self.original.size_hint(),self.whole_count) {
				((0,Some(0)),_)|(_,0) => (0,Some(0)),
				((l,u),n) => (
					l.checked_mul(n).unwrap_or(usize::MAX),
					u.and_then(|u| u.checked_mul(n) )
				)
			}
		}

	}

}



/// イテレータに最大/最小を同時に計算するメソッドを追加するモジュール
pub mod min_max {
	use super::*;
	use std::cmp::{
		Ordering,Ord,
		min_by,max_by
	};

	compose_struct! {
		pub type OptMinMax<T> = Option<(T,T)>;
		pub trait Iter<T> = Iterator<Item=T> + Sized;
		pub trait Item = Clone + Ord;
		pub trait OrdFn<T> = FnMut(&T,&T) -> Ordering;
	}

	pub trait IteratorMinMaxExtension<I,T> {
		/// イテレータに対して最大値と最小値の両方を同時に計算する
		fn min_max(self) -> OptMinMax<T>;
		/// イテレータに対して指定した計算方法を用いて最大値と最小値の両方を同時に計算する
		fn min_max_by(self,compare:impl OrdFn<T>) -> OptMinMax<T>;
	}

	impl<I:Iter<T>,T:Item> IteratorMinMaxExtension<I,T> for I {

		fn min_max(self) -> OptMinMax<T> {
			self.min_max_by(Ord::cmp)
		}

		fn min_max_by(mut self,mut compare:impl OrdFn<T>)
		-> OptMinMax<T> {
			let first = self.next()?;
			Some( self.fold(
				(first.clone(),first),
				move |(min_val,max_val),item| {
					(
						min_by(min_val,item.clone(),&mut compare),
						max_by(max_val,item,&mut compare)
					)
				}
			) )
		}

	}

}



/// 遅延評価に基づいて1個或いは0個の要素を返すイテレータを提供するモジュール
mod one_or_zero {
	use super::*;
	use std::marker::PhantomData;

	/// 遅延評価に基づいて1個或いは0個の要素を返すイテレータ
	pub struct OneOrZero<T,F> {
		func: Option<F>,
		phantom: PhantomData<T>
	}

	impl<T,F> Iterator for OneOrZero<T,F>
	where F: FnOnce() -> Option<T>
	{
		type Item = T;
		fn next(&mut self) -> Option<Self::Item> {
			let f_opt = self.func.take();
			f_opt.and_then(|f| f() )
		}
		fn size_hint(&self) -> (usize, Option<usize>) {
			if self.func.is_some() { (0,Some(1)) }
			else { (0,Some(0)) }
		}
	}

	impl<T,F> DoubleEndedIterator for OneOrZero<T,F>
	where F: FnOnce() -> Option<T>
	{
		fn next_back(&mut self) -> Option<Self::Item> {
			let f_opt = self.func.take();
			f_opt.and_then(|f| f() )
		}
	}

	impl<T,F> FusedIterator for OneOrZero<T,F>
	where F: FnOnce() -> Option<T> {}

	/// 渡された関数を実行して `Some(T)` なら1個、 `None` なら 0 個の要素を返すイテレータを生成します
	pub fn one_or_zero<T,F>(f:F) -> OneOrZero<T,F>
	where F: FnOnce() -> Option<T>
	{
		OneOrZero { func: Some(f), phantom: PhantomData }
	}

}



pub(crate) mod for_prelude {
	pub use super::{
		cycle_n::IteratorCycleNExtension,
		min_max::IteratorMinMaxExtension,
		one_or_zero::one_or_zero
	};
}
