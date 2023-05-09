use super::*;
pub(crate) use std::iter::{
	Iterator,
	ExactSizeIterator,
	DoubleEndedIterator,
	FusedIterator
};
#[cfg(feature="parallel")]
pub(crate) use rayon::iter::{
	plumbing as rayon_plumbing,
	ParallelIterator,
	IndexedParallelIterator,
	IntoParallelIterator
};



/// 有限回のみ繰り返すイテレータを生成するモジュール
mod cycle_n {
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
pub use cycle_n::IteratorCycleNExtension;



/// イテレータに最大/最小を同時に計算するメソッドを追加するモジュール
mod min_max {
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
pub use min_max::IteratorMinMaxExtension;



/// カーテジアン積のイテレータのモジュール
mod cartesian_product {
    use super::*;

	/// 複数のイテレータのカーテジアン積をとったイテレータ
	pub struct CartesianProduct<I,O,V> {
		iters_tuple: I,
		iters_original_tuple: O,
		current_val_tuple: Option<V>
	}
	type Product<I,O,V> = CartesianProduct<I,O,V>;

	/// 複数のイテレータのタプルをカーテジアン積をとった単一のイテレータに変換するトレイト
	pub trait IntoIter<O,V>: Sized {
		/// イテレータのタプル `(I1,I2,I3,...)` をカーテジアン積をとったイテレータ `Iterator<Item=(T1,T2,T3,...)` に変換します。各イテレータが `Clone` を実装していなければなりません。
		fn cartesian_product(self) -> Product<Self,O,V>;
	}

	impl<T1,T2,I1,I2> IntoIter<((),I2),(T1,())> for (I1,I2)
	where
		I1: Iterator<Item=T1>, T1: Clone,
		I2: Iterator<Item=T2> + Clone
	{
		fn cartesian_product(mut self) -> Product<Self,((),I2),(T1,())> {
			Product {
				iters_original_tuple: ((),self.1.clone()),
				current_val_tuple: (self.0.next(),Some(()))
				.zip_options(),
				iters_tuple: self
			}
		}
	}

	impl<T1,T2,I1,I2> Iterator for Product<(I1,I2),((),I2),(T1,())>
	where
		I1: Iterator<Item=T1>, T1: Clone,
		I2: Iterator<Item=T2> + Clone
	{
		type Item = (T1,T2);

		fn next(&mut self) -> Option<Self::Item> {
			let Self {
				iters_tuple: ref mut it,
				iters_original_tuple: ref iot,
				current_val_tuple: ref mut cvo,
			} = self;
			let cv = cvo.as_mut()?;

			if let Some(v) = it.1.next() {
				return Some( (cv.0.clone(),v) )
			}

			it.1 = iot.1.clone();
			if let Some(v) = it.0.next() {
				cv.0 = v;
				return Some( (cv.0.clone(),it.1.next()?) )
			}

			None
		}

		fn size_hint(&self) -> (usize, Option<usize>) {
			// it1 + iot1 * it0
			// it2 + iot2 * ( it1 + iot1 * it0 )
			// it3 + iot3 * ( it2 + iot2 * ( it1 + iot1 * it0 ) )
			// ...
			let Self {
				iters_tuple: ref it,
				iters_original_tuple: ref iot,
				..
			} = self;

			let mut ma = it.0.size_hint();
			ma = size_hint_mul_add(ma,iot.1.size_hint(),it.1.size_hint());
			// ma = size_hint_mul_add(ma,iot.2.size_hint(),it.2.size_hint());
			// ma = size_hint_mul_add(ma,iot.3.size_hint(),it.3.size_hint());
			ma
		}

	}

	impl<T1,T2,T3,I1,I2,I3> IntoIter<((),I2,I3),(T1,T2,())> for (I1,I2,I3)
	where
		I1: Iterator<Item=T1>, T1: Clone,
		I2: Iterator<Item=T2> + Clone, T2: Clone,
		I3: Iterator<Item=T3> + Clone
	{
		fn cartesian_product(mut self) -> Product<Self,((),I2,I3),(T1,T2,())> {
			Product {
				iters_original_tuple: ((),self.1.clone(),self.2.clone()),
				current_val_tuple: (self.0.next(),self.1.next(),Some(()))
				.zip_options(),
				iters_tuple: self
			}
		}
	}

	impl<T1,T2,T3,I1,I2,I3> Iterator for Product<(I1,I2,I3),((),I2,I3),(T1,T2,())>
	where
		I1: Iterator<Item=T1>, T1: Clone,
		I2: Iterator<Item=T2> + Clone, T2: Clone,
		I3: Iterator<Item=T3> + Clone
	{
		type Item = (T1,T2,T3);

		fn next(&mut self) -> Option<Self::Item> {
			let Self {
				iters_tuple: ref mut it,
				iters_original_tuple: ref iot,
				current_val_tuple: ref mut cvo,
			} = self;
			let cv = cvo.as_mut()?;

			if let Some(v) = it.2.next() {
				return Some( (cv.0.clone(),cv.1.clone(),v) )
			}

			it.2 = iot.2.clone();
			if let Some(v) = it.1.next() {
				cv.1 = v;
				return Some( (cv.0.clone(),cv.1.clone(),it.2.next()?) )
			}

			it.1 = iot.1.clone();
			if let Some(v) = it.0.next() {
				cv.0 = v;
				cv.1 = it.1.next()?;
				return Some( (cv.0.clone(),cv.1.clone(),it.2.next()?) )
			}

			None
		}

		fn size_hint(&self) -> (usize, Option<usize>) {
			let Self {
				iters_tuple: ref it,
				iters_original_tuple: ref iot,
				..
			} = self;

			let mut ma = it.0.size_hint();
			ma = size_hint_mul_add(ma,iot.1.size_hint(),it.1.size_hint());
			ma = size_hint_mul_add(ma,iot.2.size_hint(),it.2.size_hint());
			ma
		}

	}

	type UL = (usize,Option<usize>);
	/// サイズヒントの計算に役立つ積と和の計算
	fn size_hint_mul_add(a:UL,b:UL,c:UL) -> UL {
		(
			a.0 * b.0 + c.0,
			(a.1,b.1,c.1)
			.zip_options()
			.map(|(a,b,c)| a * b + c )
		)
	}

}
pub use cartesian_product::{
	CartesianProduct as ProductForIteratorsTuple,
	IntoIter as IntoTupleProductIterator
};
