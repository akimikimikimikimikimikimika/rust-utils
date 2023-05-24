//! イテレータを zip するトレイトやイテレータをまとめたモジュール

use super::*;

/// イテレータのタプルを zip する関数を含むモジュール
mod for_iters {

	/// 複数のイテレータのタプルをタプルのイテレータに変換するトレイト
	pub trait IntoIter: Sized {
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		fn into_zipped_iter(self) -> Zip<Self> { self.zip() }
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		fn zip(self) -> Zip<Self>;
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します。要素数が一致していないとパニックを発生させます。
		fn zip_eq(self) -> ZipEq<Self>;
	}

	/// 複数のイテレータを単一のイテレータに zip したイテレータ
	pub struct Zip<I> {
		pub(crate) iters: I
	}

	/// 複数のイテレータを単一のイテレータに zip したイテレータ。要素数が一致していないとパニックを発する。
	pub struct ZipEq<I> {
		pub(crate) iters: I
	}

	/// 複数のイテレータを単一のイテレータに zip したイテレータ。先に `None` に達したイテレータがあれば、残りはデフォルト値を返していく。
	pub struct ZipLongest<I,V> {
		pub(crate) iters: I,
		pub(crate) values: V
	}

	/// * イテレータの要素数ごとに `Zip` を実装するマクロ
	/// * `impl_zip_iters!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	/// * 異なる型パラメータとタプルのインデクスを交互に並べる
	macro_rules! impl_zip_iters {
		// マクロのエントリポイント: 全ての実装をモジュールで囲む
		( $( $i:ident $n:tt )+ ) => {
			mod impl_zip_iters {
				use super::*;
				use ZipForIteratorsTuple as Zip;
				use ZipEqForIteratorsTuple as ZipEq;
				use IntoTupleZippedIterator as IntoIter;

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
					Zip { iters: self }
				}
				fn zip_eq(self) -> ZipEq<Self> {
					ZipEq { iters: self }
				}
			}

			impl<$($i),+> Iterator for Zip<($($i,)+)>
			where $( $i: Iterator ),+
			{

				type Item = ( $( $i::Item, )+ );

				fn next(&mut self) -> Option<Self::Item> {
					Some( ( $( self.iters.$n.next()?, )+ ) )
				}

				fn nth(&mut self,n:usize) -> Option<Self::Item> {
					Some( ( $( self.iters.$n.nth(n)?, )+ ) )
				}

				fn size_hint(&self) -> (usize, Option<usize>) {
					let size_hint = ( $( self.iters.$n.size_hint(), )+ );
					let l = [ $( size_hint.$n.0 ),+ ].minimum();
					let u = [ $( size_hint.$n.1 ),+ ].iter().filter_map(|x| x.as_ref()).min().map(|x| *x);
					(l,u)
				}

			}

			impl<$($i),+> Iterator for ZipEq<($($i,)+)>
			where $( $i: Iterator ),+
			{

				type Item = ( $( $i::Item, )+ );

				fn next(&mut self) -> Option<Self::Item> {
					impl_zip_iters!{@zip_eq
						target( ( $(self.iters.$n.next(), )+ ) )
						indices($($i)+)
					}
				}

				fn nth(&mut self,n:usize) -> Option<Self::Item> {
					impl_zip_iters!{@zip_eq
						target( ( $(self.iters.$n.nth(n), )+ ) )
						indices($($i)+)
					}
				}

				fn size_hint(&self) -> (usize, Option<usize>) {
					let size_hint = ( $( self.iters.$n.size_hint(), )+ );
					let l = [ $( size_hint.$n.0 ),+ ].minimum();
					let u = [ $( size_hint.$n.1 ),+ ].iter().filter_map(|x| x.as_ref()).min().map(|x| *x);
					(l,u)
				}

			}

			impl<$($i),+> ExactSizeIterator for Zip<($($i,)+)>
			where $( $i: ExactSizeIterator ),+ {}

			impl<$($i),+> ExactSizeIterator for ZipEq<($($i,)+)>
			where $( $i: ExactSizeIterator ),+ {}

			impl<$($i),+> DoubleEndedIterator for Zip<($($i,)+)>
			where $( $i: DoubleEndedIterator + ExactSizeIterator ),+ {

				fn next_back(&mut self) -> Option<Self::Item> {
					let size = ( $( self.iters.$n.len(), )+ );
					let size_min = size.to_array().minimum();
					$( for _ in size_min..size.$n {
						self.iters.$n.next_back();
					} )+
					Some( ( $( self.iters.$n.next_back()?, )+ ) )
				}

				fn nth_back(&mut self,n:usize) -> Option<Self::Item> {
					let size = ( $( self.iters.$n.len(), )+ );
					let size_min = size.to_array().minimum();
					$( for _ in size_min..size.$n {
						self.iters.$n.next_back();
					} )+
					Some( ( $( self.iters.$n.nth_back(n)?, )+ ) )
				}

			}

			impl<$($i),+> DoubleEndedIterator for ZipEq<($($i,)+)>
			where $( $i: DoubleEndedIterator + ExactSizeIterator ),+ {

				fn next_back(&mut self) -> Option<Self::Item> {
					( $( self.iters.$n.len(), )+ ).len_equality();
					Some( ( $( self.iters.$n.next_back()?, )+ ) )
				}

				fn nth_back(&mut self,n:usize) -> Option<Self::Item> {
					( $( self.iters.$n.len(), )+ ).len_equality();
					Some( ( $( self.iters.$n.nth_back(n)?, )+ ) )
				}

			}

			impl<$($i),+> FusedIterator for Zip<($($i,)+)>
			where $( $i: FusedIterator ),+ {}

			impl<$($i),+> Clone for Zip<($($i,)+)>
			where $( $i: Iterator + Clone ),+
			{
				fn clone(&self) -> Self {
					Self {
						iters: ( $(self.iters.$n.clone(), )+ )
					}
				}
			}

			impl<$($i),+> Clone for ZipEq<($($i,)+)>
			where $( $i: Iterator + Clone ),+
			{
				fn clone(&self) -> Self {
					Self {
						iters: ( $(self.iters.$n.clone(), )+ )
					}
				}
			}

		};
		// `|` の前に要素が全くない場合
		(@each | ) => {};

		// `ZipEq` の `.next()` や `.nth()` の条件分岐: エントリポイント (1つだけの場合)
		(@zip_eq target($($t:tt)+) indices($i:tt)) => {
			$($t)+.0.map(|v| (v,) )
		};
		// `ZipEq` の `.next()` や `.nth()` の条件分岐: エントリポイント
		(@zip_eq target($($t:tt)+) indices($($i:tt)+)) => {
			impl_zip_iters!{@zip_eq
				target($($t)+)
				all_some() all_none() not_yet($($i)+)
			}
		};
		// `ZipEq` の `.next()` や `.nth()` の条件分岐: 各要素に対する実装
		(@zip_eq
			target($($t:tt)+)
			all_some($($s:tt)*)
			all_none($($n:tt)*)
			$( one_none($i:tt: $($o:tt)*) )*
			not_yet($y0:tt $($y:tt)*)
		) => {
			impl_zip_iters!{@zip_eq
				target($($t)+)
				all_some($($s)* Some(_),)
				all_none($($n)* None,)
				$( one_none($i: $($o)* _,) )*
				one_none($y0: $($s)* None,)
				not_yet($($y)*)
			}
		};
		// `ZipEq` の `.next()` や `.nth()` の条件分岐: 最後に呼び出され、組み立てる
		(@zip_eq
			target($($t:tt)+)
			all_some($($s:tt)+)
			all_none($($n:tt)+)
			$( one_none($i:tt: $($o:tt)+) )+
			not_yet()
		) => {
			match $($t)+ {
				p @ ($($s)+) => p.zip_options(),
				($($n)+) => None,
				$( ($($o)+) => {
					panic!(concat!("インデクス ",stringify!($i)," の要素が空になりました"));
				}, )+
			}
		};
	}
	pub(crate) use impl_zip_iters;

}
pub use for_iters::{
	Zip as ZipForIteratorsTuple,
	ZipEq as ZipEqForIteratorsTuple,
	IntoIter as IntoTupleZippedIterator
};
pub(crate) use for_iters::impl_zip_iters;



#[cfg(feature="parallel")]
/// 並列イテレータのタプルを zip する関数を含むモジュール
mod for_parallel_iters {

	/// 複数の並列イテレータのタプルをタプルのイテレータに変換するトレイト
	pub trait IntoIter: Sized {
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルの並列イテレータ `ParallelIterator<Item=(T1,T2,T3,...)>` に変換します
		fn into_zipped_iter(self) -> Zip<Self> { self.zip() }
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルの並列イテレータ `ParallelIterator<Item=(T1,T2,T3,...)>` に変換します
		fn zip(self) -> Zip<Self>;
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルの並列イテレータ `ParallelIterator<Item=(T1,T2,T3,...)>` に変換します。要素数が全て等しい必要があります。
		fn zip_eq(self) -> Zip<Self>;
	}

	/// 複数の並列化可能なアイテムから並列化したタプルのイテレータに変換するトレイト
	pub trait ParallelIntoIter {
		type ItersTuple;
		/// 並列化可能なアイテムのタプル `(I1,I2,I3,...)` をタプルの並列イテレータ `ParallelIterator<Item=(T1,T2,T3,...)>` に変換します
		fn parallel_zip(self) -> Zip<Self::ItersTuple>;
	}

	/// 複数の並列イテレータを単一のイテレータに zip した並列イテレータ
	pub struct Zip<I> {
		pub(crate) iters: I
	}

	pub(crate) struct ZipCallback<CCB,PIT> {
		pub(crate) child_callback: CCB,
		/// * `( (P0,), (P1,), ..., (Pk-1,), (), (Ik+1,), ..., (In,) )` の形式で管理する
		pub(crate) prods_iters: PIT
	}

	pub(crate) struct ZipProducer<P> {
		pub(crate) producers: P
	}

	/// * イテレータの要素数ごとに `Zip` を実装するマクロ
	/// * `impl_zip_parallel_iters!( I0 P0 T0 0 I1 P1 T1 1 I2 P2 T2 2 ... I(N-1) P(N-1) T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	/// * `I*` `P*` `T*` の異なる型パラメータとタプルのインデクスをこの順で並べていく
	macro_rules! impl_zip_parallel_iters {
		// マクロのエントリポイント: 全ての実装をモジュールで囲む
		( $( $i:ident $p:ident $t:ident $n:tt )+ ) => {
			mod impl_zip_parallel_iters {
				use super::*;
				use ZipForParallelIteratorsTuple as Zip;
				use ZipCallbackForParallelIteratorsTuple as ZipCallback;
				use ZipProducerForParallelIteratorsTuple as ZipProducer;
				use IntoTupleZippedParallelIterator as IntoIter;
				use IntoTupleZippedParallelIteratorFromIntoParallelIterator as ParallelIntoIter;
				use ZipForIteratorsTuple as ZipSerial;
				use rayon_plumbing::*;

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
					Zip { iters: self }
				}
				fn zip_eq(self) -> Zip<Self> {
					( $( self.$n.len(), )+ ).len_equality();
					self.zip()
				}
			}

			impl<$($i),+> ParallelIntoIter for ( $($i,)+ )
			where $( $i: IntoParallelIterator ),+
			{
				type ItersTuple = ( $($i::Iter,)+ );
				fn parallel_zip(self) -> Zip<Self::ItersTuple> {
					Zip { iters: ( $( self.$n.into_par_iter(), )+ ) }
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
						iters: ( $( self.iters.$n.into_par_iter(), )+ )
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
					( $( self.iters.$n.opt_len(), )+ )
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
						self.producers.$n.into_iter(),
					)+ ).into_zipped_iter()
				}

				fn min_len(&self) -> usize {
					( $(
						self.producers.$n.min_len(),
					)+ ).minimum()
				}

				fn max_len(&self) -> usize {
					( $(
						self.producers.$n.max_len(),
					)+ ).maximum()
				}

				fn split_at(self, index: usize) -> (Self, Self) {
					let split_prod = ( $(
						self.producers.$n.split_at(index),
					)+ );
					(
						Self { producers: ( $(split_prod.$n.0,)+ ) },
						Self { producers: ( $(split_prod.$n.1,)+ ) }
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
						self.iters.$n.len(),
						$( self.iters.$nf.len(), )*
					)
					.minimum()
				}

				fn with_producer<CCB>(self, child_callback: CCB) -> CCB::Output
				where CCB: ProducerCallback<Self::Item>
				{
					self.iters.$n.with_producer(ZipCallback {
						child_callback,
						prods_iters: (
							(), $( (self.iters.$nf,), )*
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
				CCB: ProducerCallback<($($tp,)*$t,$tn$(,$tf)*)>
				$(, $pp: Producer<Item=$tp> )*
				, $in: IndexedParallelIterator<Item=$tn>, $tn: Send
				$(, $if: IndexedParallelIterator<Item=$tf>, $tf: Send )*
			{
				type Output = CCB::Output;
				fn callback<$p>(self, parent_producer: $p) -> Self::Output
				where $p: Producer<Item=$t>
				{
					self.prods_iters.$nn.0.with_producer(ZipCallback {
						child_callback: self.child_callback,
						prods_iters: (
							$( (self.prods_iters.$np.0,), )*
							(parent_producer,),
							(),
							$( (self.prods_iters.$nf.0,), )*
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
			where CCB: ProducerCallback<($($tp,)*$t,)> $(, $pp: Producer<Item=$tp> )*
			{
				type Output = CCB::Output;
				fn callback<$p>(self, parent_producer: $p) -> Self::Output
				where $p: Producer<Item=$t>
				{
					self.child_callback.callback(ZipProducer {
						producers: ( $(self.prods_iters.$np.0,)* parent_producer, )
					})
				}
			}

		};
	}
	pub(crate) use impl_zip_parallel_iters;

}
#[cfg(feature="parallel")]
pub use for_parallel_iters::{
	Zip as ZipForParallelIteratorsTuple,
	IntoIter as IntoTupleZippedParallelIterator,
	ParallelIntoIter as IntoTupleZippedParallelIteratorFromIntoParallelIterator
};
#[cfg(feature="parallel")]
pub(crate) use for_parallel_iters::{
	ZipProducer as ZipProducerForParallelIteratorsTuple,
	ZipCallback as ZipCallbackForParallelIteratorsTuple,
	impl_zip_parallel_iters
};



/// `ZipEq` 向けの `len_equality` 関数を提供するモジュール
mod len_equality {

	/// 要素数が合致しているか合致する内部向けトレイト。合致しない場合はパニックを発する。
	pub(crate) trait LenEquality {
		fn len_equality(self);
	}

	/// `len_equality` をまとめて定義するマクロ
	macro_rules! impl_len_equality {
		// マクロのエントリポイント: 全ての実装をモジュールで囲む
		(indices: $($n:tt)+ ) => {
			mod len_equality {
				use super::*;
				use ZipEqLenEquality as LenEquality;

				impl_len_equality! {@each | $( usize $n )+ }
			}
		};
		// `|` より前にある要素のみの場合と、1つだけ要素を増やした場合に分ける
		(@each
			$( $up:ident $np:tt )* |
			$uc:ident $nc:tt $( $un:ident $nn:tt )*
		) => {
			impl_len_equality!{@each $( $up $np )* | }
			impl_len_equality!{@each $( $up $np )* $uc $nc | $( $un $nn )* }
		};
		// `|` の前に1つだけ要素がある場合
		(@each usize 0 | ) => {
			impl LenEquality for (usize,) {
				fn len_equality(self) {}
			}
		};
		// 全ての要素が `|` より前にある場合に実装を行う
		(@each $( $u:ident $n:tt )+ | ) => {
			impl LenEquality for ($($u,)+) {
				fn len_equality(self) {
					if impl_len_equality!{@ne self -> for $($n)+ } {
						let src = [
							"要素数が合致しません:".to_string(),
							$( format!(
								concat!("iters.",stringify!($n),".len() = {}"),
								self.$n
							), )+
							String::new()
						].join("\n");
						panic!("{}",src);
					}
				}
			}
		};
		// `|` の前に要素が全くない場合
		(@each | ) => {};

		// 非等価性の判定: 要素を2つずつ取ってまとめていく
		(@ne
			$s:ident -> $( ($cond:expr) )*
			for $n0:tt $n1:tt $($n:tt)*
		) => {
			impl_len_equality! {@ne
				$s -> $( ($cond) )* ($s.$n0!=$s.$n1)
				for $n1 $($n)*
			}
		};
		// 非等価性の判定: 2要素のペアを全て作り終えたらそれを1つに繋げる
		(@ne $s:ident -> $( ($cond:expr) )+ for $nl:tt ) => {
			$( ($cond) )||+
		};
	}
	pub(crate) use impl_len_equality;

}
pub(crate) use len_equality::{
	LenEquality as ZipEqLenEquality,
	impl_len_equality
};



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
