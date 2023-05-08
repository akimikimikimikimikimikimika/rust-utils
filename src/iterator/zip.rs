use super::*;

/// イテレータのタプルを zip する関数を含むモジュール
mod for_iters_tuple {

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

}
pub use for_iters_tuple::{
	Zip as ZipForIteratorsTuple,
	IntoIter as IntoTupleZippedIterator
};

/// * イテレータの要素数ごとに `Zip` を実装するマクロ
/// * `impl_zip_iters!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
macro_rules! impl_zip_iters {
	( $( $i:ident $n:tt )+ ) => {
		mod impl_zip_iters {
			use super::{
				ZipForIteratorsTuple as Zip,
				IntoTupleZippedIterator as IntoIter,
				*
			};

			impl_zip_iters! {@each | $( $i $n )+ }
		}
	};
	(@each $( $i:ident $n:tt )* | $in:ident $nn:tt $( $others:tt )* ) => {
		impl_zip_iters! {@each $( $i $n )* | }
		impl_zip_iters! {@each $( $i $n )* $in $nn | $( $others )* }
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
pub(crate) use impl_zip_iters;



#[cfg(feature="parallel")]
/// 並列イテレータのタプルを zip する関数を含むモジュール
mod for_parallel_iters_tuple {

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
		/// * `( (P0,), (P1,), ..., (Pk-1,), (), (Ik+1,), ..., (In,) )` の形式で管理する
		pub(crate) prods_iters_tuple: PIT
	}

	pub(crate) struct ZipProducer<T> {
		pub(crate) prods_tuple: T
	}

}

#[cfg(feature="parallel")]
pub use for_parallel_iters_tuple::{
	Zip as ZipForParallelIteratorsTuple,
	IntoIter as IntoTupleZippedParallelIterator
};

#[cfg(feature="parallel")]
pub(crate) use for_parallel_iters_tuple::{
	ZipProducer as ZipProducerForParallelIteratorsTuple,
	ZipCallback as ZipCallbackForParallelIteratorsTuple
};

/// * イテレータの要素数ごとに `Zip` を実装するマクロ
/// * `impl_zip_parallel_iters!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
macro_rules! impl_zip_parallel_iters {
	( $( $i:ident $p:ident $t:ident $n:tt )+ ) => {
		#[cfg(feature="parallel")]
		mod impl_zip_parallel_iters {
			use super::{
				ZipForParallelIteratorsTuple as Zip,
				ZipCallbackForParallelIteratorsTuple as ZipCallback,
				ZipProducerForParallelIteratorsTuple as ZipProducer,
				IntoTupleZippedParallelIterator as IntoIter,
				rayon_plumbing::*,
				*
			};

			impl_zip_parallel_iters! {@each | $( $i $p $t $n )+ }
		}
	};
	(@each $( $i:ident $p:ident $t:ident $n:tt )* | $in:ident $pn:ident $tn:ident $nn:tt $( $others:tt )* ) => {
		impl_zip_parallel_iters! {@each $( $i $p $t $n )* | }
		impl_zip_parallel_iters! {@each $( $i $p $t $n )* $in $pn $tn $nn | $( $others )* }
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

		impl_zip_parallel_iters!{@cb_entry $( $i $p $t $n )+ }

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

		impl_zip_parallel_iters!{@cb | $i $p $t $n $( $if $pf $tf $nf )* }

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

		impl_zip_parallel_iters!{@cb
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
pub(crate) use impl_zip_parallel_iters;



/// イテレータの配列を zip する関数を含むモジュール
mod for_iters_array {
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
pub use for_iters_array::{
	Zip as ZipForIteratorArray,
	IntoIter as IntoArrayZippedIterator
};
