use super::*;
pub(crate) use std::iter::{
	Iterator,
	ExactSizeIterator,
	DoubleEndedIterator,
	FusedIterator
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



/// 複数個のイテレータによる Zip を実装するモジュール
mod multi_zip {
	use super::*;

	/// イテレータのタプルを zip する関数を含むモジュール
	mod zip_tuples {
		use super::*;

		/// 複数のイテレータのタプルをタプルのイテレータに変換するトレイト
		pub trait IntoIter: Sized {
			/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
			fn into_zipped_iter(self) -> Zip<Self>;
			/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
			fn zip(self) -> Zip<Self> { self.into_zipped_iter() }
		}

		/// 複数のイテレータを単一のイテレータに zip したイテレータ
		pub struct Zip<T> {
			pub(crate) iters_tuple: T
		}

		/// * イテレータの要素数ごとに `Zip` を実装するマクロ
		/// * `impl_zipped_iter!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
		macro_rules! impl_zipped_iter {
			( $( $i:ident $n:tt )+ ) => {
				mod impl_zipped_iter {
					use super::{
						ZipForIteratorsTuple as Zip,
						IntoTupleZippedIterator as IntoIter,
						*
					};

					impl_zipped_iter! {@each | $( $i $n )+ }
				}
			};
			(@each $( $i:ident $n:tt )* | $in:ident $nn:tt $( $others:tt )* ) => {
				impl_zipped_iter! {@each $( $i $n )* | }
				impl_zipped_iter! {@each $( $i $n )* $in $nn | $( $others )* }
			};
			(@each $( $i:ident $n:tt )+ | ) => {

				impl<$($i),+> IntoIter for ($($i,)+)
				where $( $i: Iterator ),+ {
					fn into_zipped_iter(self) -> Zip<Self> {
						Zip { iters_tuple: self }
					}
				}

				impl<$($i),+> Iterator for Zip<($($i,)+)>
				where $( $i: Iterator ),+
				{

					type Item = ( $( $i::Item, )+ );

					fn next(&mut self) -> Option<Self::Item> {
						Some( ( $( self.iters_tuple.$n.next()?, )+ ) )
					}

					fn size_hint(&self) -> (usize, Option<usize>) {
						let size_hint = ( $( self.iters_tuple.$n.size_hint(), )+ );
						let l = [ $( size_hint.$n.0 ),+ ].minimum();
						let u = [ $( size_hint.$n.1 ),+ ].iter().filter_map(|x| x.as_ref()).min().map(|x| *x);
						(l,u)
					}

				}

				impl<$($i),+> ExactSizeIterator for Zip<($($i,)+)>
				where $( $i: ExactSizeIterator ),+ {}

				impl<$($i),+> DoubleEndedIterator for Zip<($($i,)+)>
				where $( $i: DoubleEndedIterator + ExactSizeIterator ),+ {
					fn next_back(&mut self) -> Option<Self::Item> {
						let size = ( $( self.iters_tuple.$n.len(), )+ );
						let size_min = size.to_array().minimum();
						$( for _ in size_min..size.$n {
							self.iters_tuple.$n.next_back();
						} )+
						Some( ( $( self.iters_tuple.$n.next_back()?, )+ ) )
					}
				}

				impl<$($i),+> FusedIterator for Zip<($($i,)+)>
				where $( $i: FusedIterator ),+ {}

			};
			(@each | ) => {};
		}
		pub(crate) use impl_zipped_iter;

	}
	pub use zip_tuples::{
		Zip as ZipForIteratorsTuple,
		IntoIter as IntoTupleZippedIterator
	};
	pub(crate) use zip_tuples::impl_zipped_iter;

	#[cfg(feature="parallel")]
	/// 並列イテレータのタプルを zip する関数を含むモジュール
	mod parallel_zip_tuples {
		use super::*;

		/// 複数の並列イテレータのタプルをタプルのイテレータに変換するトレイト
		pub trait IntoIter: Sized {
			/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
			fn into_zipped_iter(self) -> Zip<Self>;
			/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
			fn zip(self) -> Zip<Self> { self.into_zipped_iter() }
		}

		/// 複数の並列イテレータを単一のイテレータに zip した並列イテレータ
		pub struct Zip<T> {
			pub(crate) iters_tuple: T
		}

		pub(crate) struct ZipCallback<CCB,PIT> {
			pub(crate) child_callback: CCB,
			/// `( (P0,), (P1,), ..., (Pk-1,), (), (Ik+1,), ..., (In,) )`
			pub(crate) prods_iters_tuple: PIT
		}

		pub(crate) struct ZipProducer<T> {
			pub(crate) prods_tuple: T
		}

		/// * イテレータの要素数ごとに `Zip` を実装するマクロ
		/// * `impl_parallel_zipped_iter!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
		macro_rules! impl_parallel_zipped_iter {
			( $( $i:ident $p:ident $t:ident $n:tt )+ ) => {
				mod impl_parallel_zipped_iter {
					use super::{
						ZipForParallelIteratorsTuple as Zip,
						ZipCallbackForParallelIteratorsTuple as ZipCallback,
						ZipProducerForParallelIteratorsTuple as ZipProducer,
						IntoTupleZippedParallelIterator as IntoIter,
						*
					};
					use rayon::iter::{plumbing::*,ParallelIterator,IndexedParallelIterator};

					impl_parallel_zipped_iter! {@each | $( $i $p $t $n )+ }
				}
			};
			(@each $( $i:ident $p:ident $t:ident $n:tt )* | $in:ident $pn:ident $tn:ident $nn:tt $( $others:tt )* ) => {
				impl_parallel_zipped_iter! {@each $( $i $p $t $n )* | }
				impl_parallel_zipped_iter! {@each $( $i $p $t $n )* $in $pn $tn $nn | $( $others )* }
			};
			(@each $( $i:ident $p:ident $t:ident $n:tt )+ | ) => {

				impl<$($i),+> IntoIter for ( $($i,)+ )
				where $( $i: IndexedParallelIterator ),+
				{
					fn into_zipped_iter(self) -> Zip<Self> {
						Zip { iters_tuple: self }
					}
				}

				impl<$($i),+> ParallelIterator for Zip<( $($i,)+ )>
				where $( $i: IndexedParallelIterator ),+
				{

					type Item = ($($i::Item,)+);

					fn drive_unindexed<CC>(self, child_consumer: CC) -> CC::Result
					where CC: UnindexedConsumer<Self::Item>
					{ bridge(self,child_consumer) }

					fn opt_len(&self) -> Option<usize> {
						( $( self.iters_tuple.$n.opt_len(), )+ )
						.zip_options()
						.map(|t| t.minimum() )
					}

				}

				impl<$($p),+> Producer for ZipProducer<( $($p,)+ )>
				where $( $p: Producer ),+
				{
					type Item = ( $($p::Item,)+ );
					type IntoIter = ZipForIteratorsTuple<( $($p::IntoIter,)+ )>;

					fn into_iter(self) -> Self::IntoIter {
						( $(
							self.prods_tuple.$n.into_iter(),
						)+ ).into_zipped_iter()
					}

					fn min_len(&self) -> usize {
						( $(
							self.prods_tuple.$n.min_len(),
						)+ ).minimum()
					}

					fn max_len(&self) -> usize {
						( $(
							self.prods_tuple.$n.max_len(),
						)+ ).maximum()
					}

					fn split_at(self, index: usize) -> (Self, Self) {
						let split_prod = ( $(
							self.prods_tuple.$n.split_at(index),
						)+ );
						(
							Self { prods_tuple: ( $(split_prod.$n.0,)+ ) },
							Self { prods_tuple: ( $(split_prod.$n.1,)+ ) }
						)
					}

				}

				impl_parallel_zipped_iter!{@cb_entry $( $i $p $t $n )+ }

			};
			(@cb_entry
				$i:ident $p:ident $t:ident $n:tt
				$( $if:ident $pf:ident $tf:ident $nf:tt )*
			) => {

				impl<$i $(,$if)*> IndexedParallelIterator for Zip<( $i, $($if,)* )>
				where
					$i: IndexedParallelIterator
					$(, $if: IndexedParallelIterator )*
				{

					fn drive<CC>(self, child_consumer: CC) -> CC::Result
					where CC: Consumer<Self::Item>
					{ bridge(self,child_consumer) }

					fn len(&self) -> usize {
						(
							self.iters_tuple.$n.len(),
							$( self.iters_tuple.$nf.len(), )*
						)
						.minimum()
					}

					fn with_producer<CCB>(self, child_callback: CCB) -> CCB::Output
					where CCB: ProducerCallback<Self::Item>
					{
						self.iters_tuple.$n.with_producer(ZipCallback {
							child_callback,
							prods_iters_tuple: (
								(), $( (self.iters_tuple.$nf,), )*
							)
						})
					}

				}

				impl_parallel_zipped_iter!{@cb | $i $p $t $n $( $if $pf $tf $nf )* }

			};
			(@cb
				$( $ip:ident $pp:ident $tp:ident $np:tt )* |
				$i:ident $p:ident $t:ident $n:tt
				$in:ident $pn:ident $tn:ident $nn:tt
				$( $if:ident $pf:ident $tf:ident $nf:tt )*
			) => {

				impl<CCB,$in$(,$pp)*$(,$if)*,$($tp,)*$t,$tn$(,$tf)*> ProducerCallback<$t> for ZipCallback<CCB,( $(($pp,),)* (), ($in,) $(,($if,))* )>
				where
					CCB: ProducerCallback<($($tp,)*$t,$tn$(,$tf)*)>,
					$( $pp: Producer<Item=$tp>, )*
					$in: IndexedParallelIterator<Item=$tn>, $tn: Send
					$(, $if: IndexedParallelIterator<Item=$tf>, $tf: Send )*
				{
					type Output = CCB::Output;
					fn callback<$p>(self, parent_producer: $p) -> Self::Output
					where $p: Producer<Item=$t>
					{
						self.prods_iters_tuple.$nn.0.with_producer(ZipCallback {
							child_callback: self.child_callback,
							prods_iters_tuple: (
								$( (self.prods_iters_tuple.$np.0,), )*
								(parent_producer,),
								(),
								$( (self.prods_iters_tuple.$nf.0,), )*
							)
						})
					}
				}

				impl_parallel_zipped_iter!{@cb
					$( $ip $pp $tp $np )* $i $p $t $n |
					$in $pn $tn $nn
					$( $if $pf $tf $nf )*
				}

			};
			(@cb
				$( $ip:ident $pp:ident $tp:ident $np:tt )* |
				$i:ident $p:ident $t:ident $n:tt
			) => {

				impl<CCB,$($pp,)*$($tp,)*$t> ProducerCallback<$t> for ZipCallback<CCB, ( $( ($pp,), )* (), ) >
				where CCB: ProducerCallback<($($tp,)*$t,)>, $( $pp: Producer<Item=$tp> ),*
				{
					type Output = CCB::Output;
					fn callback<$p>(self, parent_producer: $p) -> Self::Output
					where $p: Producer<Item=$t>
					{
						self.child_callback.callback(ZipProducer {
							prods_tuple: ( $(self.prods_iters_tuple.$np.0,)* parent_producer, )
						})
					}
				}

			};
			(@each | ) => {};
		}
		pub(crate) use impl_parallel_zipped_iter;

	}
	#[cfg(feature="parallel")]
	pub use parallel_zip_tuples::{
		Zip as ZipForParallelIteratorsTuple,
		IntoIter as IntoTupleZippedParallelIterator
	};
	#[cfg(feature="parallel")]
	pub(crate) use parallel_zip_tuples::{
		impl_parallel_zipped_iter,
		ZipProducer as ZipProducerForParallelIteratorsTuple,
		ZipCallback as ZipCallbackForParallelIteratorsTuple
	};

	/// イテレータの配列を zip する関数を含むモジュール
	mod zip_array {
		use super::*;

		pub struct Zip<I> {
			iters: Vec<I>
		}

		pub trait IntoIter<I> {
			fn zip(self) -> Zip<I>;
		}
		impl<II,I,T> IntoIter<I> for II
		where II: IntoIterator<Item=I>, I: Iterator<Item=T>
		{
			fn zip(self) -> Zip<I> {
				Zip {
					iters: self.into_iter().map(|i| i.into_iter() ).collect()
				}
			}
		}

		impl<I,T> Iterator for Zip<I>
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

		impl<I,T> ExactSizeIterator for Zip<I>
		where I: ExactSizeIterator<Item=T> {}

		impl<I,T> DoubleEndedIterator for Zip<I>
		where I: DoubleEndedIterator<Item=T> + ExactSizeIterator {
			fn next_back(&mut self) -> Option<Self::Item> {
				let size =
				self.iters.iter()
				.map( |i| i.len() )
				.collect::<Vec<_>>();
				let size_min = size.clone().minimum();
				( self.iters.iter_mut(), size.into_iter() )
				.into_zipped_iter()
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

		impl<I,T> FusedIterator for Zip<I>
		where I: FusedIterator<Item=T> {}

	}
	pub use zip_array::{
		Zip as ZipForIteratorArray,
		IntoIter as IntoArrayZippedIterator
	};

}
pub use multi_zip::*;



/// 複数個のイテレータによりチェーンを実装するモジュール
mod multi_chain {
    use super::*;

	/// 複数のイテレータのタプルをチェーンしたイテレータに変換するトレイト
	pub trait IntoIter: Sized {
		/// イテレータのタプル `(I1,I2,I3,...)` を `I1`→`I2`→`I3` という順に連結した1つのイテレータに変換します
		fn into_chained_iter(self) -> Chain<Self>;
		/// イテレータのタプル `(I1,I2,I3,...)` を `I1`→`I2`→`I3` という順に連結した1つのイテレータに変換します
		fn chain(self) -> Chain<Self> { self.into_chained_iter() }
	}

	/// 複数のイテレータをチェーンする (連続に繋げる) イテレータです
	pub struct Chain<T> {
		pub(crate) iters_tuple: T,
		pub(crate) current: usize,
		pub(crate) current_back: usize
	}

	/// * 複数のイテレータに対する `Chain` トレイトを実装するマクロ
	/// * `impl_chain_iter!( I0 0 I1 1 I2 2 ... I(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! impl_chain_iter {
		( $( $i:ident $n:tt )+ ) => {
			mod impl_chained_iter {
				use super::{
					ChainForIteratorTuple as Chain,
					IntoChainedIteratorForIteratorsTuple as IntoIter,
					*
				};

				impl_chain_iter! {@each T | $( $i $n )+ }
			}
		};
		(@each $t:ident $( $i:ident $n:tt )* | $in:ident $nn:tt $( $others:tt )* ) => {
			impl_chain_iter! {@each $t $( $i $n )* | }
			impl_chain_iter! {@each $t $( $i $n )* $in $nn | $($others)* }
		};
		(@each $t:ident $( $i:ident $n:tt )+ | ) => {

			impl<$t,$($i),+> IntoIter for ($($i,)+)
			where $( $i: Iterator<Item=$t> ),+
			{
				fn into_chained_iter(self) -> Chain<Self> {
					Chain { iters_tuple: self, current: 0, current_back: 0 }
				}
			}

			impl<$t,$($i),+> Iterator for Chain<($($i,)+)>
			where $( $i: Iterator<Item=$t> ),+
			{
				type Item = $t;

				fn next(&mut self) -> Option<Self::Item> {
					$( if self.current==$n {
						if let s @ Some(_) = self.iters_tuple.$n.next() { return s; }
						self.current += 1;
					} )+
					None
				}

				fn size_hint(&self) -> (usize, Option<usize>) {
					let size_hint = ( $( self.iters_tuple.$n.size_hint(),)+ );
					let l = 0 $(+ size_hint.$n.0 )+;
					let u = ( $(size_hint.$n.1,)+ )
					.zip_options()
					.map(|t| 0 $(+ t.$n)+ );
					(l,u)
				}
			}

			impl_chain_iter! {@backward $t 0 | $( $i $n )+ }

			impl<$t,$($i),+> ExactSizeIterator for Chain<($($i,)+)>
			where $( $i: ExactSizeIterator<Item=$t> ),+ {}

			impl<$t,$($i),+> FusedIterator for Chain<($($i,)+)>
			where $( $i: FusedIterator<Item=$t> ),+ {}

		};
		(@each $t:ident | ) => {};
		(@backward
			$t:ident $n_largest:tt
			$( $i:ident $n:tt )* |
			$in:ident $nn:tt $( $others:tt )*
		) => {
			impl_chain_iter! {@backward $t $nn $in $nn $( $i $n )* | $($others)* }
		};
		(@backward
			$t:ident $n_largest:tt
			$( $i:ident $n:tt )+ |
		) => {
			impl<$t,$($i),+> DoubleEndedIterator for Chain<($($i,)+)>
			where $( $i: DoubleEndedIterator<Item=$t> + ExactSizeIterator ),+
			{
				fn next_back(&mut self) -> Option<Self::Item> {
					$( if self.current_back==($n_largest-$n) {
						if let s @ Some(_) = self.iters_tuple.$n.next_back() { return s; }
						self.current_back += 1;
					} )+
					None
				}
			}
		};
	}
	pub(crate) use impl_chain_iter;

}
pub use multi_chain::{
	Chain as ChainForIteratorTuple,
	IntoIter as IntoChainedIteratorForIteratorsTuple
};
pub(crate) use multi_chain::impl_chain_iter;



/// カーテジアン積のイテレータのモジュール
mod multi_product {
}
