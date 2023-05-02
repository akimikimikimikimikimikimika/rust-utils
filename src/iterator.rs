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
		pub trait IntoZippedIterator: Sized {
			/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
			fn into_zipped_iter(self) -> ZipN<Self>;
			/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
			fn zip(self) -> ZipN<Self> { self.into_zipped_iter() }
		}

		/// 複数のイテレータを単一のイテレータに zip したイテレータです
		pub struct ZipN<T> {
			pub(crate) iter_tuples: T
		}

		/// * イテレータの要素数ごとに `ZipN` を実装するマクロ
		/// * `impl_zipped_iter!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
		macro_rules! impl_zipped_iter {
			( $( $i:ident $n:tt )+ ) => {
				impl_zipped_iter! {@each | $( $i $n )+ }
			};
			(@each $( $i:ident $n:tt )* | $in:ident $nn:tt $( $others:tt )* ) => {
				impl_zipped_iter! {@each $( $i $n )* | }
				impl_zipped_iter! {@each $( $i $n )* $in $nn | $( $others )* }
			};
			(@each $( $i:ident $n:tt )+ | ) => {

				impl<$($i),+> IntoZippedIterator for ($($i,)+)
				where $( $i: Iterator ),+ {
					fn into_zipped_iter(self) -> ZipN<Self> {
						ZipN { iter_tuples: self }
					}
				}

				impl<$($i),+> Iterator for ZipN<($($i,)+)>
				where $( $i: Iterator ),+
				{

					type Item = ( $( $i::Item, )+ );

					fn next(&mut self) -> Option<Self::Item> {
						Some( ( $( self.iter_tuples.$n.next()?, )+ ) )
					}

					fn size_hint(&self) -> (usize, Option<usize>) {
						let size_hint = ( $( self.iter_tuples.$n.size_hint(), )+ );
						let l = [ $( size_hint.$n.0 ),+ ].minimum();
						let u = [ $( size_hint.$n.1 ),+ ].iter().filter_map(|x| x.as_ref()).min().map(|x| *x);
						(l,u)
					}

				}

				impl<$($i),+> ExactSizeIterator for ZipN<($($i,)+)>
				where $( $i: ExactSizeIterator ),+ {}

				impl<$($i),+> DoubleEndedIterator for ZipN<($($i,)+)>
				where $( $i: DoubleEndedIterator + ExactSizeIterator ),+ {
					fn next_back(&mut self) -> Option<Self::Item> {
						let size = ( $( self.iter_tuples.$n.len(), )+ );
						let size_min = size.to_array().minimum();
						$( for _ in size_min..size.$n {
							self.iter_tuples.$n.next_back();
						} )+
						Some( ( $( self.iter_tuples.$n.next_back()?, )+ ) )
					}
				}

				impl<$($i),+> FusedIterator for ZipN<($($i,)+)>
				where $( $i: FusedIterator ),+ {}

			};
			(@each | ) => {};
		}
		pub(crate) use impl_zipped_iter;

	}
	pub use zip_tuples::*;

	/// イテレータの配列を zip する関数を含むモジュール
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

		impl<I,T> FusedIterator for MultiZip<I>
		where I: FusedIterator<Item=T> {}

	}
	pub use zip_array::*;

}
pub use multi_zip::*;



/// 複数個のイテレータによりチェーンを実装するモジュール
mod multi_chain {
    use super::*;

	/// 複数のイテレータのタプルをチェーンしたイテレータに変換するトレイト
	pub trait IntoChainedIterator: Sized {
		/// イテレータのタプル `(I1,I2,I3,...)` を `I1`→`I2`→`I3` という順に連結した1つのイテレータに変換します
		fn into_chained_iter(self) -> Chain<Self>;
	}

	/// 複数のイテレータをチェーンする (連続に繋げる) イテレータです
	pub struct Chain<T> {
		pub(crate) iter_tuples: T,
		pub(crate) current: usize,
		pub(crate) current_back: usize
	}

	/// * 複数のイテレータに対する `Chain` トレイトを実装するマクロ
	/// * `impl_chain_iter!( I0 0 I1 1 I2 2 ... I(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! impl_chain_iter {
		( $( $i:ident $n:tt )+ ) => {
			impl_chain_iter! {@each T | $( $i $n )+ }
		};
		(@each $t:ident $( $i:ident $n:tt )* | $in:ident $nn:tt $( $others:tt )* ) => {
			impl_chain_iter! {@each $t $( $i $n )* | }
			impl_chain_iter! {@each $t $( $i $n )* $in $nn | $($others)* }
		};
		(@each $t:ident $( $i:ident $n:tt )+ | ) => {

			impl<$t,$($i),+> IntoChainedIterator for ($($i,)+)
			where $( $i: Iterator<Item=$t> ),+
			{
				fn into_chained_iter(self) -> Chain<Self> {
					Chain { iter_tuples: self, current: 0, current_back: 0 }
				}
			}

			impl<$t,$($i),+> Iterator for Chain<($($i,)+)>
			where $( $i: Iterator<Item=$t> ),+
			{
				type Item = $t;

				fn next(&mut self) -> Option<Self::Item> {
					$( if self.current==$n {
						if let s @ Some(_) = self.iter_tuples.$n.next() { return s; }
						self.current += 1;
					} )+
					None
				}

				fn size_hint(&self) -> (usize, Option<usize>) {
					let size_hint = ( $( self.iter_tuples.$n.size_hint(),)+ );
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
						if let s @ Some(_) = self.iter_tuples.$n.next_back() { return s; }
						self.current_back += 1;
					} )+
					None
				}
			}
		};
	}
	pub(crate) use impl_chain_iter;

}
pub use multi_chain::*;



/// カーテジアン積のイテレータのモジュール
mod multi_product {}



#[cfg(test)]
#[test]
fn test() {
	let iter = (
		(1..=3).map(|x| x*x ),
		(1..=3).map(|x| x*x*x ),
	).chain();
	let src = iter
	.map(|x| format!("{}",x) )
	.collect::<Vec<_>>()
	.join(",");
	println!("{}",src);
}
