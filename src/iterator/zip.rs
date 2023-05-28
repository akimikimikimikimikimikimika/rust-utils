//! イテレータを zip するトレイトやイテレータをまとめたモジュール

use super::*;

/// イテレータのタプルを zip する関数を含むモジュール
pub mod for_iters {

	/// 複数のイテレータのタプルをタプルのイテレータに変換するトレイト
	pub trait IntoZip: Sized {
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		fn into_zipped_iter(self) -> Zip<Self> { self.zip() }
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します
		fn zip(self) -> Zip<Self>;
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します。要素数が一致していないとパニックを発生させます。
		fn zip_eq(self) -> ZipEq<Self>;
	}

	pub trait IntoZipLongest: Sized {
		type Item;
		type Iter;
		/// イテレータのタプル `(I1,I2,I3,...)` をタプルのイテレータ `Iterator<Item=(T1,T2,T3,...)>` に変換します。要素数が一致しておらず、先に末尾に達したイテレータはデフォルト値を返します。
		fn zip_longest(self,default:Self::Item) -> Self::Iter;
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
	/// * `implement!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	/// * 異なる型パラメータとタプルのインデクスを交互に並べる
	macro_rules! implement {
		// マクロのエントリポイント: 全ての実装をモジュールで囲む
		( $( $i:ident $t:ident $n:tt )+ ) => {
			mod impl_zip_iters {
				use super::*;
				use std::sync::Arc;
				use crate::iterator::zip::{
					for_iters::*,
					len_equality::LenEquality
				};

				/// 内部からのみアクセス可能で `ZipLongest` 向けの実装を提供する `Iterator` トレイトと同じメソッドを持つ構造体
				struct ZLImpl<I,V> {
					iters: I,
					values: V
				}

				// `|` で要素を区切り、要素数ごとにマクロで実装を定義
				implement! {@each | $( $i $t $n )+ }
			}
		};
		// `|` より前にある要素のみの場合と、1つだけ要素を増やした場合に分ける
		(@each $( $i:ident $t:ident $n:tt )* | $in:ident $tn:ident $nn:tt $( $others:tt )* ) => {
			implement! {@each $( $i $t $n )* | }
			implement! {@each $( $i $t $n )* $in $tn $nn | $( $others )* }
		};
		// 全ての要素が `|` より前にある場合に実装を行う
		(@each $( $i:ident $t:ident $n:tt )+ | ) => {

			impl<$($i),+> IntoZip for ($($i,)+)
			where $( $i: Iterator ),+ {
				fn zip(self) -> Zip<Self> {
					Zip { iters: self }
				}
				fn zip_eq(self) -> ZipEq<Self> {
					ZipEq { iters: self }
				}
			}

			impl<$($i),+,$($t),+> IntoZipLongest for ($($i,)+)
			where $( $i: Iterator<Item=$t>, $t: Clone ),+ {
				type Item = ( $( $t, )+ );
				type Iter = ZipLongest<Self,Self::Item>;

				fn zip_longest(self,default:Self::Item) -> Self::Iter {
					Self::Iter {
						iters: self,
						values: default
					}
				}
			}

			impl<$($i),+,$($t),+> Iterator for Zip<($($i,)+)>
			where $( $i: Iterator<Item=$t> ),+
			{

				type Item = ( $( $t, )+ );

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

			impl<$($i),+,$($t),+> Iterator for ZipEq<($($i,)+)>
			where $( $i: Iterator<Item=$t> ),+
			{

				type Item = ( $( $t, )+ );

				fn next(&mut self) -> Option<Self::Item> {
					implement!{@zip_eq
						target( ( $(self.iters.$n.next(), )+ ) )
						indices($($i)+)
					}
				}

				fn nth(&mut self,n:usize) -> Option<Self::Item> {
					implement!{@zip_eq
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

			impl<$($i),+,$($t),+> Iterator for ZipLongest<($($i,)+),($($t,)+)>
			where $( $i: Iterator<Item=$t>, $t: Clone ),+
			{
				type Item = ( $($t,)+ );

				fn next(&mut self) -> Option<Self::Item> {
					ZLImpl { iters: &mut self.iters, values: &self.values }
					.next()
				}

				fn size_hint(&self) -> (usize,Option<usize>) {
					ZLImpl { iters: &self.iters, values: &self.values }
					.size_hint()
				}
			}

			// 並列の `ZipLongest` 向けにデフォルト値のタプルを `Arc` にしたイテレータも用意
			// 機能としては `Arc` のない場合と全く同じ
			// `Arc` がある場合とない場合の両方に対応するために `ZLImpl` という内部構造体を使用している
			impl<$($i),+,$($t),+> Iterator for ZipLongest<($($i,)+),Arc<($($t,)+)>>
			where $( $i: Iterator<Item=$t>, $t: Clone + Send + Sync ),+
			{
				type Item = ( $($t,)+ );

				fn next(&mut self) -> Option<Self::Item> {
					ZLImpl { iters: &mut self.iters, values: &*self.values }
					.next()
				}

				fn size_hint(&self) -> (usize,Option<usize>) {
					ZLImpl { iters: &self.iters, values: &*self.values }
					.size_hint()
				}
			}

			impl<'a,$($i),+,$($t),+> ZLImpl<&'a mut ($($i,)+),&'a ($($t,)+)>
			where $( $i: Iterator<Item=$t>, $t: Clone ),+
			{
				fn next(&mut self) -> Option<($($t,)+)> {
					let t = ( $( self.iters.$n.next(), )+ );
					if matches!(t,implement!{@repeat $( $n None )+ }) { return None; }
					Some( ( $(
						t.$n.unwrap_or_else(|| self.values.$n.clone() ),
					)+ ) )
				}
			}

			impl<'a,$($i),+,$($t),+> ZLImpl<&'a ($($i,)+),&'a ($($t,)+)>
			where $( $i: Iterator<Item=$t>, $t: Clone ),+
			{
				fn size_hint(&self) -> (usize,Option<usize>) {
					let size_hint = ( $( self.iters.$n.size_hint(), )+ );
					let l = [ $( size_hint.$n.0 ),+ ].minimum();
					let u = [ $( size_hint.$n.1 ),+ ].iter().filter_map(|x| x.as_ref()).max().map(|x| *x);
					(l,u)
				}
			}

			impl<$($i),+> ExactSizeIterator for Zip<($($i,)+)>
			where $( $i: ExactSizeIterator ),+ {}

			impl<$($i),+> ExactSizeIterator for ZipEq<($($i,)+)>
			where $( $i: ExactSizeIterator ),+ {}

			impl<$($i),+,$($t,)+> ExactSizeIterator for ZipLongest<($($i,)+),($($t,)+)>
			where $( $i: ExactSizeIterator<Item=$t>, $t: Clone ),+ {}

			impl<$($i),+,$($t,)+> ExactSizeIterator for ZipLongest<($($i,)+),Arc<($($t,)+)>>
			where $( $i: ExactSizeIterator<Item=$t>, $t: Clone + Send + Sync ),+ {}

			impl<$($i),+> DoubleEndedIterator for Zip<($($i,)+)>
			where $( $i: DoubleEndedIterator + ExactSizeIterator ),+
			{

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
			where $( $i: DoubleEndedIterator + ExactSizeIterator ),+
			{

				fn next_back(&mut self) -> Option<Self::Item> {
					( $( self.iters.$n.len(), )+ ).len_equality();
					Some( ( $( self.iters.$n.next_back()?, )+ ) )
				}

				fn nth_back(&mut self,n:usize) -> Option<Self::Item> {
					( $( self.iters.$n.len(), )+ ).len_equality();
					Some( ( $( self.iters.$n.nth_back(n)?, )+ ) )
				}

			}

			impl<$($i),+,$($t,)+> DoubleEndedIterator for ZipLongest<($($i,)+),($($t,)+)>
			where $( $i: DoubleEndedIterator<Item=$t> + ExactSizeIterator, $t: Clone ),+
			{
				fn next_back(&mut self) -> Option<Self::Item> {
					ZLImpl { iters: &mut self.iters, values: &self.values }
					.next_back()
				}
			}

			impl<$($i),+,$($t,)+> DoubleEndedIterator for ZipLongest<($($i,)+),Arc<($($t,)+)>>
			where $( $i: DoubleEndedIterator<Item=$t> + ExactSizeIterator, $t: Clone + Send + Sync ),+
			{
				fn next_back(&mut self) -> Option<Self::Item> {
					ZLImpl { iters: &mut self.iters, values: &*self.values }
					.next_back()
				}
			}

			impl<'a,$($i),+,$($t),+> ZLImpl<&'a mut ($($i,)+),&'a ($($t,)+)>
			where $( $i: DoubleEndedIterator<Item=$t> + ExactSizeIterator, $t: Clone ),+
			{
				fn next_back(&mut self) -> Option<($($t,)+)> {
					let len = ( $( self.iters.$n.len(), )+ );
					let lm = len.maximum();
					if lm==0 { return None; }
					Some( ( $(
						if len.$n<lm { self.values.$n.clone() }
						else { self.iters.$n.next_back()? }
					,)+ ) )
				}
			}

			impl<$($i),+> FusedIterator for Zip<($($i,)+)>
			where $( $i: FusedIterator ),+ {}

			impl<$($i),+> FusedIterator for ZipEq<($($i,)+)>
			where $( $i: FusedIterator ),+ {}

			impl<$($i),+,$($t),+> FusedIterator for ZipLongest<($($i,)+),($($t,)+)>
			where $( $i: FusedIterator<Item=$t>, $t: Clone ),+ {}

			impl<$($i),+> Clone for Zip<($($i,)+)>
			where $( $i: Iterator + Clone ),+
			{
				fn clone(&self) -> Self {
					Self {
						iters: ( $( self.iters.$n.clone(), )+ )
					}
				}
			}

			impl<$($i),+> Clone for ZipEq<($($i,)+)>
			where $( $i: Iterator + Clone ),+
			{
				fn clone(&self) -> Self {
					Self {
						iters: ( $( self.iters.$n.clone(), )+ )
					}
				}
			}

			impl<$($i),+,$($t),+> Clone for ZipLongest<($($i,)+),($($t,)+)>
			where $( $i: Iterator<Item=$t> + Clone, $t: Clone ),+
			{
				fn clone(&self) -> Self {
					Self {
						iters: ( $( self.iters.$n.clone(), )+ ),
						values: ( $( self.values.$n.clone(), )+ )
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
			implement!{@zip_eq
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
			implement!{@zip_eq
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

		// 受け取ったアイテムの数だけの要素を含むタプルを返す
		(@repeat $( $phantom:tt $value:ident )+ ) => {
			( $($value,)+ )
		}
	}
	pub(crate) use implement;

}



#[cfg(feature="parallel")]
/// 並列イテレータのタプルを zip する関数を含むモジュール
pub mod for_parallel_iters {

	/// 複数の並列イテレータのタプルをタプルのイテレータに変換するトレイト
	pub trait IntoZip: Sized {
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルの並列イテレータ `ParallelIterator<Item=(T1,T2,T3,...)>` に変換します
		fn into_zipped_iter(self) -> Zip<Self> { self.zip() }
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルの並列イテレータ `ParallelIterator<Item=(T1,T2,T3,...)>` に変換します
		fn zip(self) -> Zip<Self>;
		/// 並列イテレータのタプル `(I1,I2,I3,...)` をタプルの並列イテレータ `ParallelIterator<Item=(T1,T2,T3,...)>` に変換します。要素数が全て等しい必要があります。
		fn zip_eq(self) -> Zip<Self>;
	}

	/// 複数の並列イテレータのタプルをタプルのイテレータに変換するトレイト
	pub trait IntoZipLongest: Sized {
		type Item;
		fn zip_longest(self,default:Self::Item) -> ZipLongest<Self,Self::Item>;
	}

	/// 複数の並列化可能なアイテムから並列化したタプルのイテレータに変換するトレイト
	pub trait IntoParallelZip {
		type ItersTuple;
		/// 並列化可能なアイテムのタプル `(I1,I2,I3,...)` をタプルの並列イテレータ `ParallelIterator<Item=(T1,T2,T3,...)>` に変換します
		fn parallel_zip(self) -> Zip<Self::ItersTuple>;
	}

	/// 複数の並列イテレータを単一のイテレータに zip した並列イテレータ
	pub struct Zip<I> {
		pub(crate) iters: I
	}

	/// 複数の並列イテレータを単一のイテレータに zip した並列イテレータ。要素数が一致しない場合は、デフォルト値を返す。
	pub struct ZipLongest<I,V> {
		pub(crate) iters: I,
		pub(crate) values: V
	}

	pub(crate) struct ZipCallback<CCB,PIT> {
		pub(crate) child_callback: CCB,
		/// * `( (P0,), (P1,), ..., (Pk-1,), (), (Ik+1,), ..., (In,) )` の形式で管理する
		pub(crate) prods_iters: PIT
	}

	pub(crate) struct ZipProducer<P> {
		pub(crate) producers: P
	}

	use std::sync::Arc;
	pub(crate) struct ZipLongestProducer<P,V> {
		pub(crate) producers: P,
		pub(crate) values: Arc<V>
	}

	/// * イテレータの要素数ごとに `Zip` を実装するマクロ
	/// * `implement!( I0 P0 T0 0 I1 P1 T1 1 I2 P2 T2 2 ... I(N-1) P(N-1) T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	/// * `I*` `P*` `T*` の異なる型パラメータとタプルのインデクスをこの順で並べていく
	macro_rules! implement {
		// マクロのエントリポイント: 全ての実装をモジュールで囲む
		( $( $i:ident $p:ident $t:ident $n:tt )+ ) => {
			mod impl_zip_parallel_iters {
				use super::*;
				use std::sync::Arc;
				use crate::iterator::zip::{
					for_parallel_iters::*,
					for_iters::{
						Zip as ZipSerial,
						ZipLongest as ZipLongestSerial
					},
					len_equality::LenEquality
				};
				use rayon_plumbing::*;

				implement! {@each | $( $i $p $t $n )+ }
			}
		};

		// `|` より前にある要素のみの場合と、1つだけ要素を増やした場合に分ける
		(@each
			$( $i:ident $p:ident $t:ident $n:tt )* |
			$in:ident $pn:ident $tn:ident $nn:tt
			$( $others:tt )*
		) => {
			implement! {@each $( $i $p $t $n )* | }
			implement! {@each $( $i $p $t $n )* $in $pn $tn $nn | $( $others )* }
		};
		// 全ての要素が `|` より前にある場合に実装を行う
		(@each $( $i:ident $p:ident $t:ident $n:tt )+ | ) => {

			impl<$($i),+> IntoZip for ($($i,)+)
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

			impl<$($i),+,$($t),+> IntoZipLongest for ($($i,)+)
			where $( $i: IndexedParallelIterator<Item=$t>, $t: Clone + Send + Sync ),+
			{
				type Item = ($($t,)+);
				fn zip_longest(self,default:Self::Item) -> ZipLongest<Self,Self::Item> {
					ZipLongest { iters: self, values: default }
				}
			}

			impl<$($i),+> IntoParallelZip for ($($i,)+)
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

			impl<$($i),+> ParallelIterator for Zip<($($i,)+)>
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

			impl<$($i),+,$($t),+> ParallelIterator for ZipLongest<($($i,)+),($($t,)+)>
			where $( $i: IndexedParallelIterator<Item=$t>, $t: Clone + Send + Sync ),+
			{

				type Item = ($($i::Item,)+);

				fn drive_unindexed<CC>(self, child_consumer: CC) -> CC::Result
				where CC: UnindexedConsumer<Self::Item>
				{ bridge(self,child_consumer) }

				fn opt_len(&self) -> Option<usize> {
					( $( self.iters.$n.opt_len(), )+ )
					.zip_options()
					.map(|t| t.maximum() )
				}

			}

			impl<$($p),+> Producer for ZipProducer<($($p,)+)>
			where $( $p: Producer ),+
			{
				type Item = ( $($p::Item,)+ );
				type IntoIter = ZipSerial<( $($p::IntoIter,)+ )>;

				fn into_iter(self) -> Self::IntoIter {
					ZipSerial { iters: ( $(
						self.producers.$n.into_iter(),
					)+ ) }
				}

				fn min_len(&self) -> usize {
					( $(
						self.producers.$n.min_len(),
					)+ ).minimum()
				}

				fn max_len(&self) -> usize {
					( $(
						self.producers.$n.max_len(),
					)+ ).minimum()
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

			impl<$($p),+,$($t),+> Producer for ZipLongestProducer<($($p,)+),($($t,)+)>
			where $( $p: Producer<Item=$t>, $t: Clone + Send + Sync ),+
			{
				type Item = ($($t,)+);
				type IntoIter = ZipLongestSerial<($($p::IntoIter,)+),Arc<($($t,)+)>>;

				fn into_iter(self) -> Self::IntoIter {
					ZipLongestSerial {
						iters: ( $( self.producers.$n.into_iter(), )+ ),
						values: self.values
					}
				}

				fn min_len(&self) -> usize {
					( $( self.producers.$n.min_len(), )+ )
					.maximum()
				}

				fn max_len(&self) -> usize {
					( $( self.producers.$n.max_len(), )+ )
					.maximum()
				}

				fn split_at(self, index: usize) -> (Self, Self) {
					let split_prod = ( $( self.producers.$n.split_at(index), )+ );
					(
						Self {
							producers: ( $( split_prod.$n.0, )+ ),
							values: self.values.clone()
						},
						Self {
							producers: ( $( split_prod.$n.1, )+ ),
							values: self.values
						}
					)
				}
			}

			implement!{@cb_entry $( $i $p $t $n )+ }

		};
		// `|` の前に要素が全くない場合
		(@each | ) => {};

		// `ProducerCallback` の実装のエントリポイント: `IndexedParallelIterator` の実装を行う
		(@cb_entry
			$i:ident $p:ident $t:ident $n:tt
			$( $if:ident $pf:ident $tf:ident $nf:tt )*
		) => {

			impl<$i$(,$if)*> IndexedParallelIterator for Zip<( $i, $($if,)* )>
			where
				$i: IndexedParallelIterator,
				$( $if: IndexedParallelIterator, )*
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

			impl<$i$(,$if)*,$t$(,$tf)*> IndexedParallelIterator for ZipLongest<($i,$($if,)*),($t,$($tf,)*)>
			where
				$i: IndexedParallelIterator<Item=$t>,
				$( $if: IndexedParallelIterator<Item=$tf>, )*
				$t: Clone + Send + Sync,
				$( $tf: Clone + Send + Sync, )*
			{

				fn drive<CC>(self, child_consumer: CC) -> CC::Result
				where CC: Consumer<Self::Item>
				{ bridge(self,child_consumer) }

				fn len(&self) -> usize {
					(
						self.iters.$n.len(),
						$( self.iters.$nf.len(), )*
					).maximum()
				}


				fn with_producer<CCB>(self, child_callback: CCB) -> CCB::Output
				where CCB: ProducerCallback<Self::Item>
				{
					self.iters.$n
					.with_producer(ZipCallback {
						child_callback,
						prods_iters: (
							(self.values.$n,),
							$( (self.values.$nf,self.iters.$nf), )*
						)
					})
				}

			}

			implement!{@cb | $i $p $t $n $( $if $pf $tf $nf )* }

		};
		// `ProducerCallback` の実装: N個の要素があれば、最初の N-1 個についてはここで実装を行う
		(@cb
			$( $ip:ident $pp:ident $tp:ident $np:tt )* |
			$i:ident $p:ident $t:ident $n:tt
			$in:ident $pn:ident $tn:ident $nn:tt
			$( $if:ident $pf:ident $tf:ident $nf:tt )*
		) => {

			impl< CCB $(,$pp)*, $in$(,$if)*, $($tp,)*$t,$tn$(,$tf)* >
			ProducerCallback<$t>
			for ZipCallback<CCB,( $(($pp,),)* (), ($in,) $(,($if,))* )>
			where
				CCB: ProducerCallback<($($tp,)*$t,$tn$(,$tf)*)>,
				$( $pp: Producer<Item=$tp>, )*
				$in: IndexedParallelIterator<Item=$tn>,
				$( $if: IndexedParallelIterator<Item=$tf>, )*
				$tn: Send, $( $tf: Send, )*
			{
				type Output = CCB::Output;
				fn callback<$p>(self, parent_producer: $p) -> Self::Output
				where $p: Producer<Item=$t>
				{
					self.prods_iters.$nn.0
					.with_producer( ZipCallback {
						child_callback: self.child_callback,
						prods_iters: (
							$( self.prods_iters.$np, )*
							(parent_producer,),
							(),
							$( self.prods_iters.$nf, )*
						)
					} )
				}
			}

			impl< CCB $(,$pp)*, $in$(,$if)*, $($tp,)*$t,$tn$(,$tf)* >
			ProducerCallback<$t>
			for ZipCallback<CCB,( $(($tp,$pp),)* ($t,), ($tn,$in) $(,($tf,$if))* )>
			where
				CCB: ProducerCallback<($($tp,)*$t,$tn$(,$tf)*)>,
				$( $pp: Producer<Item=$tp>, )*
				$in: IndexedParallelIterator<Item=$tn>,
				$( $if: IndexedParallelIterator<Item=$tf>, )*
				$( $tp: Clone + Send + Sync, )*
				$t: Clone + Send + Sync,
				$tn: Clone + Send + Sync,
				$( $tf: Clone + Send + Sync, )*
			{
				type Output = CCB::Output;
				fn callback<$p>(self, parent_producer: $p) -> Self::Output
				where $p: Producer<Item=$t>
				{
					self.prods_iters.$nn.1
					.with_producer( ZipCallback {
						child_callback: self.child_callback,
						prods_iters: (
							$( self.prods_iters.$np, )*
							(self.prods_iters.$n.0,parent_producer),
							(self.prods_iters.$nn.0,),
							$( self.prods_iters.$nf, )*
						)
					} )
				}
			}

			implement!{@cb
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

			impl< CCB, $($pp,)* $($tp,)*$t > ProducerCallback<$t> for ZipCallback<CCB, ( $( ($pp,), )* (), ) >
			where
				CCB: ProducerCallback<($($tp,)*$t,)>,
				$( $pp: Producer<Item=$tp>, )*
			{
				type Output = CCB::Output;
				fn callback<$p>(self, parent_producer: $p) -> Self::Output
				where $p: Producer<Item=$t>
				{
					self.child_callback
					.callback( ZipProducer {
						producers: (
							$( self.prods_iters.$np.0, )*
							parent_producer,
						)
					} )
				}
			}

			impl< CCB, $($pp,)* $($tp,)*$t > ProducerCallback<$t> for ZipCallback<CCB, ( $( ($tp,$pp), )* ($t,), ) >
			where
				CCB: ProducerCallback<($($tp,)*$t,)>,
				$( $pp: Producer<Item=$tp>, )*
				$( $tp: Clone + Send + Sync, )*
				$t: Clone + Send + Sync
			{
				type Output = CCB::Output;
				fn callback<$p>(self, parent_producer: $p) -> Self::Output
				where $p: Producer<Item=$t>
				{
					self.child_callback
					.callback( ZipLongestProducer {
						values: Arc::new( (
							$( self.prods_iters.$np.0, )*
							self.prods_iters.$n.0,
						) ),
						producers: (
							$( self.prods_iters.$np.1, )*
							parent_producer,
						)
					} )
				}
			}

		};
	}
	pub(crate) use implement;

}



/// `ZipEq` 向けの `len_equality` 関数を提供するモジュール
pub(crate) mod len_equality {

	/// 要素数が合致しているか合致する内部向けトレイト。合致しない場合はパニックを発する。
	pub(crate) trait LenEquality {
		fn len_equality(self);
	}

	/// `len_equality` をまとめて定義するマクロ
	macro_rules! implement {
		// マクロのエントリポイント: 全ての実装をモジュールで囲む
		(indices: $($n:tt)+ ) => {
			mod len_equality {
				use crate::iterator::zip::len_equality::*;

				implement! {@each | $( usize $n )+ }
			}
		};
		// `|` より前にある要素のみの場合と、1つだけ要素を増やした場合に分ける
		(@each
			$( $up:ident $np:tt )* |
			$uc:ident $nc:tt $( $un:ident $nn:tt )*
		) => {
			implement!{@each $( $up $np )* | }
			implement!{@each $( $up $np )* $uc $nc | $( $un $nn )* }
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
					if implement!{@ne self -> for $($n)+ } {
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
			implement! {@ne
				$s -> $( ($cond) )* ($s.$n0!=$s.$n1)
				for $n1 $($n)*
			}
		};
		// 非等価性の判定: 2要素のペアを全て作り終えたらそれを1つに繋げる
		(@ne $s:ident -> $( ($cond:expr) )+ for $nl:tt ) => {
			$( ($cond) )||+
		};
	}
	pub(crate) use implement;

}



/// イテレータの配列を zip する関数を含むモジュール
pub mod for_iters_array {
	use super::*;
	use crate::prelude::*;

	/// 複数のイテレータの配列をベクタのイテレータに変換するトレイト
	pub struct Zip<I> {
		iters: Vec<I>
	}

	pub trait IntoZip<I> {
		/// イテレータの配列 `[I;N]` や `Vec<I>` などを配列のイテレータ `Iterator<Item=Vec<T>>` に変換します
		fn zip(self) -> Zip<I>;
	}
	impl<II,I,T> IntoZip<I> for II
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



/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::{
		for_iters::{
			IntoZip as IntoZipForIterators,
			IntoZipLongest as IntoZipLongestForIterators
		},
		for_iters_array::
		IntoZip as IntoArrayZippedIterator
	};
	#[cfg(feature="parallel")]
	pub use super::for_parallel_iters::{
		IntoZipLongest as IntoZipLongestForParallelIterators,
		IntoParallelZip as IntoZipForParallelIteratorsFromSerial
	};
}
