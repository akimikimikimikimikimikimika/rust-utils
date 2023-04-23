use super::*;



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



/// 同一要素からなるタプル型を配列に変換するモジュール
mod tuple_to_array {

	/// タプルを配列に変換します
	pub trait TupleToArray<T,const N:usize> {
		/// タプルを配列に変換します
		fn to_array(self) -> [T;N];
	}

	/// `impl` をマクロによりまとめて実行する
	macro_rules! impl_t2a {
		(indices: $i0:tt $($i:tt)+ ) => {
			impl_t2a! {@each T T $i0 | $($i),+ }
		};
		(@each $t:ident $($tx:ident $x:tt),+ | $y0:tt $(,$y:tt)* ) => {
			impl<$t> TupleToArray<$t,$y0> for ($($tx,)+) {
				fn to_array(self) -> [T;$y0] {
					[ $(self.$x),+ ]
				}
			}
			impl_t2a! {@each $t $($tx $x,)+ $t $y0 | $($y),* }
		};
		(@each $t:ident $($tx:ident $x:tt),+ | ) => {};
	}
	impl_t2a!(indices: 0 1 2 3 4 5 6 7 8 9 10 11 12 );

}
use tuple_to_array::*;



/// 複数個のイテレータによる Zip を実装するモジュール
mod multi_zip {
	use super::*;
	use std::iter::{
		Iterator,
		ExactSizeIterator,
		DoubleEndedIterator,
		FusedIterator
	};

	/// 複数のイテレータのタプルをタプルのイテレータに変換するトレイト
	pub trait IntoZippedIterator where Self: Sized {
		/// * イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		/// * 最大で12個のイテレータまで対応
		fn into_iter(self) -> ZipN<Self>;
		/// * イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		/// * 最大で12個のイテレータまで対応
		fn zip(self) -> ZipN<Self> { self.into_iter() }
	}

	mod zip_tuples {
		use super::*;

		/// 複数のイテレータを単一のイテレータに zip したイテレータです
		pub struct ZipN<T> {
			iter_tuples: T
		}

		/// イテレータの要素数ごとに `ZipN` を実装するマクロ
		macro_rules! impl_zipped_iter {
			( | $t0:ident $i0:tt $( $t:ident $i:tt )+ ) => {
				impl_zipped_iter! { $t0 $i0 | $( $t $i )+ }
			};
			( $( $t:ident $i:tt )+ | $tn:ident $in:tt $( $others:tt )* ) => {

				impl_zipped_iter! { $( $t $i )+ | }

				impl_zipped_iter! { $( $t $i )+ $tn $in | $( $others )* }

			};
			( $( $t:ident $i:tt )+ | ) => {

				impl<$($t),+> IntoZippedIterator for ($($t,)+)
				where $( $t: Iterator ),+ {
					fn into_iter(self) -> ZipN<Self> {
						ZipN { iter_tuples: self }
					}
				}

				impl<$($t),+> Iterator for ZipN<($($t,)+)>
				where $( $t: Iterator ),+
				{

					type Item = ( $( $t::Item, )+ );

					fn next(&mut self) -> Option<Self::Item> {
						Some( ( $( self.iter_tuples.$i.next()?, )+ ) )
					}

					fn size_hint(&self) -> (usize, Option<usize>) {
						let size_hint = ( $( self.iter_tuples.$i.size_hint(), )+ );
						let l = [ $( size_hint.$i.0 ),+ ].minimum();
						let u = [ $( size_hint.$i.1 ),+ ].iter().filter_map(|x| x.as_ref()).min().map(|x| *x);
						(l,u)
					}

				}

				impl<$($t),+> ExactSizeIterator for ZipN<($($t,)+)>
				where $( $t: ExactSizeIterator ),+ {}

				impl<$($t),+> DoubleEndedIterator for ZipN<($($t,)+)>
				where $( $t: DoubleEndedIterator + ExactSizeIterator ),+ {
					fn next_back(&mut self) -> Option<Self::Item> {
						let size = ( $( self.iter_tuples.$i.len(), )+ );
						let size_min = size.to_array().minimum();
						$( for _ in size_min..size.$i {
							self.iter_tuples.$i.next_back();
						} )+
						Some( ( $( self.iter_tuples.$i.next_back()?, )+ ) )
					}
				}

				impl<$($t),+> FusedIterator for ZipN<($($t,)+)>
				where $( $t: FusedIterator ),+ {}

			};
		}
		impl_zipped_iter!( | T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9 T10 10 T11 11 );

	}
	pub use zip_tuples::*;

	mod zip_array {
		use super::*;

		pub struct MultiZip<I> {
			iters: Vec<I>
		}

		pub trait IntoMultiZip<I> {
			fn multi_zip(self) -> MultiZip<I>;
		}
		impl<II,I,T> IntoMultiZip<I> for II
		where II: IntoIterator<Item=I>, I: Iterator<Item=T>
		{
			fn multi_zip(self) -> MultiZip<I> {
				MultiZip {
					iters: self.into_iter().map(|i| i.into_iter() ).collect()
				}
			}
		}

		impl<I,T> Iterator for MultiZip<I>
		where I: Iterator<Item=T>
		{

			type Item = Vec<T>;

			fn next(&mut self) -> Option<Self::Item> {
				if self.iters.is_empty() { return None; }
				let mut is_some = true;
				let values =
				self.iters.iter_mut()
				.filter_map(|i| {
					let v = i.next();
					if v.is_none() { is_some = false; }
					v
				} )
				.collect::<Self::Item>();
				is_some.then_some(values)
			}

			fn size_hint(&self) -> (usize, Option<usize>) {
				if self.iters.is_empty() { return (0,Some(0)); }
				self.iters.iter()
				.map( |i| i.size_hint() )
				.reduce(|(l1,u1),(l2,u2)| (
					l1.min(l2),
					match (u1,u2) {
						(Some(v1),Some(v2)) => Some(v1.min(v2)),
						(Some(v),None)|(None,Some(v)) => Some(v),
						(None,None) => None
					}
				) )
				.unwrap_or((0,Some(0)))
			}

		}

		impl<I,T> ExactSizeIterator for MultiZip<I>
		where I: ExactSizeIterator<Item=T> {}

		impl<I,T> DoubleEndedIterator for MultiZip<I>
		where I: DoubleEndedIterator<Item=T> + ExactSizeIterator {
			fn next_back(&mut self) -> Option<Self::Item> {
				let size =
				self.iters.iter()
				.map( |i| i.len() )
				.collect::<Vec<_>>();
				let size_min = size.clone().minimum();
				( self.iters.iter_mut(), size.into_iter() )
				.into_iter()
				.for_each(|(i,s)| {
					for _ in size_min..s { i.next_back(); }
				});

				let mut is_some = true;
				let values =
				self.iters.iter_mut()
				.filter_map(|i| {
					let v = i.next_back();
					if v.is_none() { is_some = false; }
					v
				})
				.collect::<Self::Item>();
				is_some.then_some(values)
			}
		}

		impl<I,T> FusedIterator for MultiZip<I>
		where I: FusedIterator<Item=T> {}

	}
	pub use zip_array::*;

}
pub use multi_zip::*;



/// カーテジアン積のイテレータのモジュール
mod multi_product {}
