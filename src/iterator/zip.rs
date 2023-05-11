use super::*;

/// イテレータのタプルを zip する関数を含むモジュール
mod for_iters_tuple {

	/// 複数のイテレータのタプルをタプルのイテレータに変換するトレイト
	pub trait IntoIter: Sized {
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		fn into_zipped_iter(self) -> Zip<Self> { self.zip() }
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		fn zip(self) -> Zip<Self>;
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
/// * 異なる型パラメータとタプルのインデクスを交互に並べる
macro_rules! impl_zip_iters {
	// マクロのエントリポイント: 全ての実装をモジュールで囲む
	( $( $i:ident $n:tt )+ ) => {
		mod impl_zip_iters {
			use super::{
				ZipForIteratorsTuple as Zip,
				IntoTupleZippedIterator as IntoIter,
				*
			};

			// `|` で要素を区切り、要素数ごとにマクロで実装を定義
			impl_zip_iters! {@each | $( $i $n )+ }
		}
	};
	// `|` より前にある要素のみの場合と、1つだけ要素を増やした場合に分ける
	(@each $( $i:ident $n:tt )* | $in:ident $nn:tt $( $others:tt )* ) => {
		impl_zip_iters! {@each $( $i $n )* | }
		impl_zip_iters! {@each $( $i $n )* $in $nn | $( $others )* }
	};
	// 全ての要素が `|` より前にある場合に実装を行う
	(@each $( $i:ident $n:tt )+ | ) => {

		impl<$($i),+> IntoIter for ($($i,)+)
		where $( $i: Iterator ),+ {
			fn zip(self) -> Zip<Self> {
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
	// `|` の前に要素が全くない場合
	(@each | ) => {};
}
pub(crate) use impl_zip_iters;



#[cfg(feature="parallel")]
/// 並列イテレータのタプルを zip する関数を含むモジュール
mod for_parallel_iters_tuple {

	/// 複数の並列イテレータのタプルをタプルのイテレータに変換するトレイト
	pub trait IntoIter: Sized {
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		fn into_zipped_iter(self) -> Zip<Self> { self.zip() }
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		fn zip(self) -> Zip<Self>;
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します。要素数が全て等しい必要があります。
		fn zip_eq(self) -> Zip<Self>;
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
/// * `impl_zip_parallel_iters!( I0 P0 T0 0 I1 P1 T1 1 I2 P2 T2 2 ... I(N-1) P(N-1) T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
/// * `I*` `P*` `T*` の異なる型パラメータとタプルのインデクスをこの順で並べていく
macro_rules! impl_zip_parallel_iters {
	// マクロのエントリポイント: 全ての実装をモジュールで囲む
	( $( $i:ident $p:ident $t:ident $n:tt )+ ) => {
		#[cfg(feature="parallel")]
		mod impl_zip_parallel_iters {
			use super::{
				ZipForParallelIteratorsTuple as Zip,
				ZipCallbackForParallelIteratorsTuple as ZipCallback,
				ZipProducerForParallelIteratorsTuple as ZipProducer,
				IntoTupleZippedParallelIterator as IntoIter,
				ZipForIteratorsTuple as ZipSerial,
				rayon_plumbing::*,
				*
			};

			impl_zip_parallel_iters! {@each | $( $i $p $t $n )+ }
		}
	};

	// `|` より前にある要素のみの場合と、1つだけ要素を増やした場合に分ける
	(@each
		$( $i:ident $p:ident $t:ident $n:tt )* |
		$in:ident $pn:ident $tn:ident $nn:tt
		$( $others:tt )*
	) => {
		impl_zip_parallel_iters! {@each $( $i $p $t $n )* | }
		impl_zip_parallel_iters! {@each $( $i $p $t $n )* $in $pn $tn $nn | $( $others )* }
	};
	// 全ての要素が `|` より前にある場合に実装を行う
	(@each $( $i:ident $p:ident $t:ident $n:tt )+ | ) => {

		impl<$($i),+> IntoIter for ( $($i,)+ )
		where $( $i: IndexedParallelIterator ),+
		{
			fn zip(self) -> Zip<Self> {
				Zip { iters_tuple: self }
			}
			fn zip_eq(self) -> Zip<Self> {
				impl_zip_parallel_iters!{@zip_eq_cond self $($n)+ }
				self.zip()
			}
		}

		impl<$($i),+,$($t),+> IntoParallelIterator for ZipSerial<($($i,)+)>
		where
			$( $i: IntoParallelIterator + Iterator<Item=$t>, $t: Send, )+
			Zip<($($i::Iter,)+)>: ParallelIterator<Item=($($t,)+)>
		{
			type Item = ($($t,)+);
			type Iter = Zip<($($i::Iter,)+)>;

			fn into_par_iter(self) -> Self::Iter {
				Zip {
					iters_tuple: ( $( self.iters_tuple.$n.into_par_iter(), )+ )
				}
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
	// `|` の前に要素が全くない場合
	(@each | ) => {};

	// `ProducerCallback` の実装のエントリポイント: `IndexedParallelIterator` の実装を行う
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
	// `ProducerCallback` の実装: N個の要素があれば、最初の N-1 個についてはここで実装を行う
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
	// `ProducerCallback` の実装: 最後の要素はここで実装を行う
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

	// `IntoIter` の `zip_eq` の条件式を用意: 要素数が1個の場合は判定なし
	(@zip_eq_cond self 0 ) => {};
	// `IntoIter` の `zip_eq` の条件式を用意: 要素数が複数の場合
	(@zip_eq_cond $s:ident $($n:tt)+ ) => {
		let l = ( $( $s.$n.len(), )+ );
		if impl_zip_parallel_iters!{@ne l -> for $($n)+ } {
			let src = [
				"要素数が合致しません:".to_string(),
				$( format!(
					concat!("iters.",stringify!($n),".len() = {}"),
					l.$n
				), )+
				String::new()
			].join("\n");
			panic!("{}",src);
		}
	};
	// zip_eq_cond 向けの非等価性の判定
	(@ne
		$l:ident -> $( ($cond:expr) )*
		for $n0:tt $n1:tt $($n:tt)*
	) => {
		impl_zip_parallel_iters! {@ne
			$l -> $( ($cond) )* ($l.$n0!=$l.$n1)
			for $n1 $($n)*
		}
	};
	(@ne $l:ident -> $( ($cond:expr) )+ for $nl:tt ) => {
		$( ($cond) )||+
	};
}
pub(crate) use impl_zip_parallel_iters;



/// イテレータの配列を zip する関数を含むモジュール
mod for_iters_array {
	use super::*;

	/// 複数のイテレータの配列をベクタのイテレータに変換するトレイト
	pub struct Zip<I> {
		iters: Vec<I>
	}

	pub trait IntoIter<I> {
		/// イテレータの配列 `[I;N]` や `Vec<I>` などを配列のイテレータ `Iterator<Item=Vec<T>>` に変換します
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
	Zip as ZipForIteratorsArray,
	IntoIter as IntoArrayZippedIterator
};
