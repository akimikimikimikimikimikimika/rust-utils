//! * イテレータに対する通常の `map` 関数を拡張したイテレータを提供するモジュール
//! * 具体的には `Result<T,E>` → `Result<U,E>` をクロージャ `Fn(T)->U` により写像させる `map_ok` や、 `T: Into<U>` を用いて `T` → `U` を写像させる `map_into` などが含まれる。
//! * また、カスタムの写像定義を作りやすいように構築されている。
//! * `itertools` の `map_ok` や　`map_into` の実装と同じく、直列/並列ごとに一般的なイテレータを実装してから、 `map_ok` や `map_into` それぞれごとの特殊化した機能を組み込んでいる。

use super::*;

/// 直列イテレータを写像する
pub mod for_serial_iter {
	use super::*;

	/// 直列イテレータの写像関数を定義するトレイトです。写像の入力型を型パラメータ `Input` で指定して、連想型 `Output` で写像関数の出力型を定めます。 `call_mut` メソッドを実装して具体的な処理内容を指定します。
	pub trait ExtendedMapFn<Input> {
		/// 写像関数の出力型
		type Output;
		/// 直列のイテレータに対して写像の仕方を定義するメソッドです。引数として `&mut self` を持つため、写像関数の保持するデータを書き換えることが可能です。例えばこのトレイトを実装した構造体でクロージャ `FnMut` を持ち `call_mut` 内で呼び出すことができます。
		fn call_mut(&mut self,input:Input) -> Self::Output;
	}
	use ExtendedMapFn as MapFn;

	/// イテレータを写像関数により写像するイテレータです。型パラメータとして写像元のイテレータ `I` と、写像関数 `F` を与えます。
	pub struct ExtendedMap<I,F> {
		pub(super) iter: I,
		pub(super) map_fn: F
	}
	use ExtendedMap as Map;

	impl<I,F> Iterator for Map<I,F>
	where I: Iterator, F: MapFn<I::Item>
	{
		type Item = F::Output;

		fn next(&mut self) -> Option<Self::Item> {
			let Self { iter, ref mut map_fn } = self;
			iter.next()
			.map( |i| map_fn.call_mut(i) )
		}

		fn nth(&mut self, n: usize) -> Option<Self::Item> {
			let Self { iter, ref mut map_fn } = self;
			iter.nth(n)
			.map( |i| map_fn.call_mut(i) )
		}

		fn size_hint(&self) -> (usize, Option<usize>) {
			self.iter.size_hint()
		}

		fn fold<A,FF>(mut self,init: A,mut fold_func: FF) -> A
		where FF: FnMut(A,Self::Item) -> A,
		{
			let Self { iter, ref mut map_fn } = self;
			iter
			.fold(
				init,
				move |acc,v| fold_func(acc,map_fn.call_mut(v))
			)
		}

		fn collect<C>(mut self) -> C
		where C: FromIterator<Self::Item>,
		{
			let Self { iter, ref mut map_fn } = self;
			iter
			.map(move |v| map_fn.call_mut(v) )
			.collect()
		}
	}

	impl<I,F> DoubleEndedIterator for Map<I,F>
	where I: DoubleEndedIterator, F: MapFn<I::Item>
	{
		fn next_back(&mut self) -> Option<Self::Item> {
			let Self { iter, ref mut map_fn } = self;
			iter
			.next_back()
			.map( |i| map_fn.call_mut(i) )
		}

		fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
			let Self { iter, ref mut map_fn } = self;
			iter.nth_back(n)
			.map( |i| map_fn.call_mut(i) )
		}
	}

	impl<I,F> ExactSizeIterator for Map<I,F>
	where I: ExactSizeIterator, F: MapFn<I::Item> {
		fn len(&self) -> usize { self.iter.len() }
	}

}

/// 並列イテレータを写像する
pub mod for_parallel_iter {
	use super::*;
	use rayon_plumbing::*;

	// ここでは独自の ParallelIterator を定義する際の見本となるように、 ParallelIterator を構成する各要素の意義が分かりやすいよう、多くのコメントを付している

	/// 並列イテレータの写像関数を定義するトレイトです。写像の入力型を型パラメータ `Input` で指定して、直列の場合の写像関数で定義された連想型 `Output` で写像関数の出力型を定めます。 `call` メソッドを実装して具体的な処理内容を指定します。
	pub trait ExtendedMapFn<Input>: MapFnSerial<Input> + Send + Sync {
		/// 並列のイテレータに対して写像の仕方を定義するメソッドです。
		fn call(&self,input:Input) -> Self::Output;
	}
	use ExtendedMapFn as MapFn;
	use for_serial_iter::ExtendedMapFn as MapFnSerial;

	/// 並列イテレータを写像関数により写像する並列イテレータです。型パラメータとして写像元のイテレータ `I` と、写像関数 `F` を与えます。
	pub struct ExtendedMap<I,F> {
		/// 1つ上の階層の `ParallelIterator`
		pub(super) parent_iterator: I,
		/// 写像関数
		pub(super) map_fn: F
	}
	use ExtendedMap as Map;
	use for_serial_iter::ExtendedMap as MapSerial;

	/// 同等の直列イテレータから並列イテレータを生成するトレイト
	impl<I,F,TI,TO> IntoParallelIterator for MapSerial<I,F>
	where
		I: IntoParallelIterator<Item=TI>,
		F: MapFn<TI,Output=TO>,
		TO: Send
	{
		type Item = TO;
		type Iter = Map<I::Iter,F>;
		/// 並列イテレータへの変換を行う
		fn into_par_iter(self) -> Self::Iter {
			Map {
				parent_iterator: self.iter.into_par_iter(),
				map_fn: self.map_fn
			}
		}
	}

	/// 並列イテレータの実装 (要素数が決まっているとは限らない)
	impl<I,F,TI,TO> ParallelIterator for Map<I,F>
	where
		I: ParallelIterator<Item=TI>,
		F: MapFn<TI,Output=TO>,
		TO: Send
	{
		type Item = TO;

		/// 必須: このステップで要素を生成して下流に渡すために、下流の `Consumer` を受け取って、このステップの `Consumer` を生成し、上流に渡す。
		fn drive_unindexed<CC>(self,child_consumer:CC) -> CC::Result
		where CC: UnindexedConsumer<Self::Item>
		{
			// 下流のコンシューマを束ねたコンシューマを生成し、上流に渡す
			let consumer = MapConsumer {
				child_consumer, map_fn: &self.map_fn
			};
			self.parent_iterator.drive_unindexed(consumer)
		}

		/// 任意: はっきり決められる場合はこのイテレータが生成する要素数を返す。要素数が決まる場合は `UnindexedConsumer` のメソッドは使うべきではない。
		fn opt_len(&self) -> Option<usize> {
			self.parent_iterator.opt_len()
		}
	}

	/// 並列イテレータの実装。要素数が決まっていなければならない。
	impl<I,F,TI,TO> IndexedParallelIterator for Map<I,F>
	where
		I: IndexedParallelIterator<Item=TI>,
		F: MapFn<TI,Output=TO>,
		TO: Send
	{
		/// 必須: このステップで要素を生成して下流に渡すために、下流の `Consumer` を受け取って、このステップの `Consumer` を生成し、上流に渡す。
		fn drive<CC>(self,child_consumer:CC) -> CC::Result
		where CC: Consumer<Self::Item>
		{
			// 下流のコンシューマを束ねたコンシューマを生成し、上流に渡す
			let consumer = MapConsumer {
				child_consumer, map_fn: &self.map_fn
			};
			self.parent_iterator.drive(consumer)
		}

		/// 必須: このイテレータが生成する要素数を与える。
		fn len(&self) -> usize {
			self.parent_iterator.len()
		}

		/// 必須: このステップの `Producer` を作成するために、下流の `Callback` を受け取り、このステップの `Callback` を用意して、上流に渡す。
		fn with_producer<CCB>(self,child_callback:CCB) -> CCB::Output
		where CCB: ProducerCallback<Self::Item>
		{
			// 下流のコールバックを束ねたコールバックを生成し、上流に渡す
			let callback = MapCallback {
				child_callback, map_fn: self.map_fn
			};
			self.parent_iterator.with_producer(callback)
		}
	}

	/// `ProducerCallback`: このステップの `Producer` を生成するコールバック関数。上流に渡され、上流から順に `Producer` が再帰的に生成する。
	struct MapCallback<CCB,F> {
		/// 1つ下の階層の `Callback`
		child_callback: CCB,
		/// 写像関数
		map_fn: F
	}

	/// コールバックの実装
	impl<CCB,F,TI,TO> ProducerCallback<TI> for MapCallback<CCB,F>
	where
		CCB: ProducerCallback<TO>,
		F: MapFn<TI,Output=TO>
	{
		type Output = CCB::Output;

		/// 必須: 上流のプロデューサと併せてこのステップのプロデューサを生成するコールバック。上流のコールバックから呼び出され、下流のコールバックを呼び出す。
		fn callback<PP>(self, parent_producer: PP) -> Self::Output
		where PP: Producer<Item=TI> {
			self.child_callback.callback(MapProducer {
				parent_producer,
				map_fn: &self.map_fn
			})
		}
	}

	// 以下は、並列イテレータのプロデューサの役割を担う部品を生成するセクション

	/// `Producer`: 分割されて、実際に処理を行う `IntoIterator` に変換可能な型。上流から下流に向かって進んでいく。
	struct MapProducer<'f,PP,F> {
		/// 1つ上の階層の `Producer`
		parent_producer: PP,
		/// 写像関数 (リファレンス)
		map_fn: &'f F
	}

	/// プロデューサの実装
	impl<'f,PP,F> Producer for MapProducer<'f,PP,F>
	where
		PP: Producer, F: MapFn<PP::Item>
	{
		type Item = F::Output;
		type IntoIter = MapSerialRef<'f,PP::IntoIter,F>;

		/// 必須: `Producer` をこれ以上分割せず、上流の `Producer` も含めてまとめてイテレータに変換する。
		fn into_iter(self) -> Self::IntoIter {
			MapSerialRef {
				iter: self.parent_producer.into_iter(),
				map_fn: self.map_fn
			}
		}

		/// 必須: `Producer` を `index` の位置で2つに分割して2つの `Producer` を用意する。上流の分割も行う。
		fn split_at(self,index:usize) -> (Self, Self) {
			let (left,right) = self.parent_producer.split_at(index);
			(
				Self { parent_producer: left , map_fn: self.map_fn },
				Self { parent_producer: right, map_fn: self.map_fn }
			)
		}

		/// 任意: 直列で処理する可能性のある最小の要素数を返す。標準値は1 (全ての要素を別スレッドで並列に処理する場合)
		fn min_len(&self) -> usize {
			self.parent_producer.min_len()
		}

		/// 任意: 直列で処理する可能性のある最大の要素数を返す。標準値は MAX (全くスレッド分割を行わない場合)
		fn max_len(&self) -> usize {
			self.parent_producer.max_len()
		}

		/// 任意: この `Producer` をイテレートして各要素を下流の `Folder` に渡すために、下流の `Folder` を受け取って、このステップの `Folder` を用意して、上流に渡す。
		fn fold_with<CF>(self, child_folder:CF) -> CF
		where CF: Folder<Self::Item>
		{
			self.parent_producer
			.fold_with( MapFolder {
				child_folder,
				map_fn: self.map_fn
			} )
			.child_folder
		}
	}

	// 以下は、並列イテレータのコンシューマの役割を担う部品を生成するセクション

	/// `Consumer`: 分割されて、実際に処理を行う `Folder` に変換可能な型。分割したものは `Reducer` により1つにまとめられる。下流から上流に向かって進んでいく。
	struct MapConsumer<'f,CC,F> {
		/// 1つ下の階層の `Consumer`
		child_consumer: CC,
		/// 写像関数 (リファレンス)
		map_fn: &'f F
	}

	/// コンシューマの実装
	impl<'f,CC,F,TI,TO> Consumer<TI> for MapConsumer<'f,CC,F>
	where
		CC: Consumer<TO>, F: MapFn<TI,Output=TO>
	{
		type Folder = MapFolder<'f,CC::Folder,F>;
		/// `Reducer`: 分割された2つのイテレータを処理し切った後に、それぞれの `Result` を単一の `Result` に結合する
		type Reducer = CC::Reducer;
		type Result = CC::Result;

		/// 必須: `Consumer` を `index` の位置で2つに分割して2つの `Consumer` を用意する。下流の分割も行い、下流からの `Reducer` をそのまま上流に渡す。
		fn split_at(self, index: usize) -> (Self, Self, Self::Reducer) {
			let (left,right,reducer) = self.child_consumer.split_at(index);
			(
				Self { child_consumer: left , map_fn: self.map_fn },
				Self { child_consumer: right, map_fn: self.map_fn },
				reducer
			)
		}

		/// 必須: この `Consumer` を要素1つ1つに対して処理できる `Folder` に変換する。
		fn into_folder(self) -> Self::Folder {
			Self::Folder {
				child_folder: self.child_consumer.into_folder(),
				map_fn: self.map_fn
			}
		}

		/// 必須: この `Consumer` がこれ以上の要素を処理し切れないかどうかを判定する。下流に問い合わせる。
		fn full(&self) -> bool {
			self.child_consumer.full()
		}
	}

	/// コンシューマの実装。任意の場所で分割できる。
	impl<'f,CC,F,TI,TO> UnindexedConsumer<TI> for MapConsumer<'f,CC,F>
	where
		CC: UnindexedConsumer<TO>,
		F: MapFn<TI,Output=TO>
	{
		/// この `Consumer` を任意の場所で区切って、その左側を返す。
		fn split_off_left(&self) -> Self {
			Self {
				child_consumer: self.child_consumer.split_off_left(),
				map_fn: self.map_fn
			}
		}

		/// 分割されて得られた結果を1つにまとめるための `Reducer` を返す。
		fn to_reducer(&self) -> Self::Reducer {
			self.child_consumer.to_reducer()
		}
	}

	/// `Folder`: 要素を1つ単位で、或いはイテレータとしてまとめて処理を行って (`consume`) 下流に渡し、下流から結果を受け取る (`complete`) 分割したものは `Reducer` により1つにまとめられる。
	struct MapFolder<'f,CF,F> {
		/// 1つ下の階層の `Folder`
		child_folder: CF,
		/// 写像関数
		map_fn: &'f F
	}

	impl<'f,CF,F,TI,TO> Folder<TI> for MapFolder<'f,CF,F>
	where
		CF: Folder<TO>,
		F: MapFn<TI,Output=TO>
	{
		/// 下流で `fold` された結果を返す
		type Result = CF::Result;

		/// 必須: 1つの要素を受け取り、処理を行った上で下流に投げて、自分自身を返す。
		fn consume(mut self,input:TI) -> Self {
			let output = self.map_fn.call(input);
			self.child_folder = self.child_folder.consume(output);
			self
		}

		/// 任意: イテレータにより要素をまとめて受け取り、処理を行った上で下流に投げて、自分自身を返す。
		fn consume_iter<I>(mut self,iter:I) -> Self
		where I: IntoIterator<Item=TI>
		{
			let output_iter = MapSerialRef {
				iter: iter.into_iter(),
				map_fn: self.map_fn
			};
			self.child_folder = self.child_folder.consume_iter(output_iter);
			self
		}

		/// 必須: 全ての値を処理し終えたことを伝える。下流まで伝播させる。
		fn complete(self) -> Self::Result {
			self.child_folder.complete()
		}

		/// 必須: この `Folder` がこれ以上の要素を処理し切れないかどうかを判定する。下流に問い合わせる。
		fn full(&self) -> bool {
			self.child_folder.full()
		}
	}

	// 以下は、並列イテレータを分割して直列化した際のイテレータを用意するセクション
	// 通常は対応する直列イテレータが存在すれば、それを利用したらいいのだが、直列の場合と違って、並列イテレータの写像はミュータブルなキャプチャを認めておらず、また写像関数 `map_fn` を参照の形で保持したいから、わざわざ別のイテレータを用意している。

	/// 並列イテレータを直列化した際に使う直列のイテレータ
	struct MapSerialRef<'f,I,F> {
		iter: I,
		map_fn: &'f F
	}

	impl<'f,I,F> Iterator for MapSerialRef<'f,I,F>
	where I: Iterator, F: MapFn<I::Item>
	{
		type Item = F::Output;

		fn next(&mut self) -> Option<Self::Item> {
			let Self { iter, ref map_fn } = self;
			iter.next()
			.map( |i| map_fn.call(i) )
		}

		fn nth(&mut self, n: usize) -> Option<Self::Item> {
			let Self { iter, ref map_fn } = self;
			iter.nth(n)
			.map( |i| map_fn.call(i) )
		}

		fn size_hint(&self) -> (usize, Option<usize>) {
			self.iter.size_hint()
		}

		fn fold<A,FF>(self,init: A,mut fold_func: FF) -> A
		where FF: FnMut(A,Self::Item) -> A,
		{
			let Self { iter, ref map_fn } = self;
			iter
			.fold(
				init,
				move |acc,v| fold_func(acc,map_fn.call(v))
			)
		}

		fn collect<C>(self) -> C
		where C: FromIterator<Self::Item>,
		{
			let Self { iter, ref map_fn } = self;
			iter
			.map(move |v| map_fn.call(v) )
			.collect()
		}
	}

	impl<'f,I,F> DoubleEndedIterator for MapSerialRef<'f,I,F>
	where I: DoubleEndedIterator, F: MapFn<I::Item>
	{
		fn next_back(&mut self) -> Option<Self::Item> {
			let Self { iter, ref map_fn } = self;
			match iter.next_back() {
				Some(i) => Some(map_fn.call(i)),
				None => None
			}
		}

		fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
			let Self { iter, ref map_fn } = self;
			match iter.nth_back(n) {
				Some(i) => Some(map_fn.call(i)),
				None => None
			}
		}
	}

	impl<'f,I,F> ExactSizeIterator for MapSerialRef<'f,I,F>
	where I: ExactSizeIterator, F: MapFn<I::Item> {
		fn len(&self) -> usize { self.iter.len() }
	}

}



/// 具体的なケースに対して実装を行う
mod iter_impl {
	use super::*;
	use std::marker::PhantomData;
	use for_serial_iter::{
		ExtendedMap as Map,
		ExtendedMapFn as MapFn
	};
	use for_parallel_iter::{
		ExtendedMap as ParallelMap,
		ExtendedMapFn as ParallelMapFn
	};

	/// 簡単な表式で写像関数を定義するマクロ
	macro_rules! make {
		( [
			$( $module_name:ident : {
				item_type: { $($it:tt)+ }
				items: [ $($items:tt)+ ]
			} )+
		] ) => {
			make!{@each {
				modules: [
					$( {
						name: $module_name
						item_type: { $($it)+ }
						items: [ $($items)+ ]
						source: { use super::*; }
						exports: [ ]
						parallel_exports: [ ]
					} )+
				]
				into_map: {
					into_map_trait: {}
					into_parallel_map_trait: {}
				}
			} }
		};
		(@each {
			modules: [ {
				// モジュールの名前
				name: $mn:ident
				// このモジュール内の写像関数が対象とする写像元イテレータのデータ型
				// `(型パラメータ),+:(入力の型)` という形で定義する
				item_type: { $($t:ident),+: $ti:ty }
				// モジュールに含まれる写像関数
				items: [ {
					// `IntoMap`, `IntoParallelMap` におけるイテレータを生成する関数名
					name_fn: $nf:ident
					// 写像関数をカプセルした直列イテレータの型名
					name_iter_serial: $nis:ident
					// 写像関数をカプセルした並列イテレータの型名
					name_iter_parallel: $nip:ident
					// 写像関数となる構造体の型名
					name_map_fn: $nmf:ident
					// 写像関数を説明する文字列リテラル (省略可)
					$( desc: $desc:literal )?
					// イテレータ生成にあたって必要な引数 (省略可)
					// 写像関数の構造体のフィールドの型にもなる
					$( params: [ $( $pi:tt: $pt:ty ),+ ] )?
					// 写像関数の構造体が `PhantomData` を使って保持する必要のある型 (省略可)
					$( phantom_params: [ $( $ppt:ty ),+ ] )?
					// 写像関数の構造体を定義するのに必要な型パラメータ
					$( map_fn_type_params: [ $($ftp:ident),+ ] )?
					// 写像関数が `MapFn` や `ParallelMapFn` トレイトの `impl` を与えるために必要な型パラメータ (省略可)
					$( impl_type_params: [ $($itp:ident),+ ] )?
					output_type: { $to:ty }
					// `MapFn` や `IntoMap` の実装に必要な `where` 節 (省略可)
					$( where_serial: { $($ws:tt)+ } )?
					// `ParallelMapFn` や `ParallelIntoMap` の実装に必要な `where` 節 (省略可)
					$( where_parallel: { $($wp:tt)+ } )?
					// 写像関数の中身
					// `self,input -> (処理内容)` という形で与える
					call: { $si:ident, $ii:ident -> $($body:tt)+ }
				} $($other_items:tt)* ]
				source: { $($src:tt)* }
				exports: [ $($exp:ident),* ]
				parallel_exports: [ $($exp_p:ident),* ]
			} $($other_modules:tt)* ]
			into_map: {
				into_map_trait: { $($im:tt)* }
				into_parallel_map_trait: { $($ipm:tt)* }
			}
		} ) => {
			make! {@each {
				modules: [ {
					name: $mn
					item_type: { $($t),+: $ti }
					items: [ $($other_items)* ]
					source: { $($src)*

						$( #[doc=concat!(
							"`",stringify!($nf),"()` にて生成されるイテレータを構成する `ExtendedMap` 向けの関数。", $desc
						)] )?
						pub struct $nmf
						<$($($ftp,)+)?>
						($($(pub(super) $pt,)+)? $($(pub(super) PhantomData<$ppt>,)+)?);

						impl<$($t,)+ $($($itp,)+)?> MapFn<$ti>
						for $nmf <$($($ftp,)+)?>
						$( where $($ws)+ )?
						{
							type Output = $to;
							fn call_mut(&mut $si,$ii:$ti) -> $to {
								$($body)+
							}
						}

						#[cfg(feature="parallel")]
						impl<$($t,)+ $($($itp,)+)?> ParallelMapFn<$ti>
						for $nmf <$($($ftp,)+)?>
						$( where $($wp)+ )?
						{
							fn call(&$si,$ii:$ti) -> $to {
								$($body)+
							}
						}

						$( #[doc=concat!(
							"`",stringify!($nf),"()` にて生成されるイテレータ。", $desc
						)] )?
						pub type $nis<I $($(,$ftp)+)?> = Map<I,$nmf <$($($ftp,)+)?>>;

						#[cfg(feature="parallel")]
						$( #[doc=concat!(
							"`",stringify!($nf),"()` にて生成される並列イテレータ。", $desc
						)] )?
						pub type $nip<I $($(,$ftp)+)?> = ParallelMap<I,$nmf <$($($ftp,)+)?>>;

					}
					exports: [ $($exp,)* $nis ]
					parallel_exports: [ $($exp_p,)* $nip ]
				} $($other_modules)* ]
				into_map: {
					into_map_trait: { $($im)*
						$( #[doc=$desc] )?
						fn $nf <$($t,)+ $($($itp,)+)?> (self $($(,$pi:$pt)+)?)
						-> $nis<Self $($(,$ftp)+)?>
						where Self: Iterator<Item=$ti> $(, $($ws)+ )?
						{ Map { iter: self, map_fn: $mn::$nmf ($($($pi,)+)? $($(PhantomData::<$ppt>,)+)?) } }
					}
					into_parallel_map_trait: { $($ipm)*
						$( #[doc=$desc] )?
						fn $nf<$($t,)+ $($($itp,)+)?> (self $($(,$pi:$pt)+)?)
						-> $nip<Self $($(,$ftp)+)?>
						where Self: ParallelIterator<Item=$ti> $(, $($wp)+ )?
						{ ParallelMap { parent_iterator: self, map_fn: $mn::$nmf ($($($pi,)+)? $($(PhantomData::<$ppt>,)+)?) } }
					}
				}
			} }
		};
		(@each {
			modules: [
				{
					name: $mn:ident
					item_type: { $($t:ident),+: $ti:ty }
					items: [ ]
					source: { $($src:tt)+ }
					exports: [ $($exp:ident),+ ]
					parallel_exports: [ $($exp_p:ident),+ ]
				}
				$($other_modules:tt)*
			]
			into_map: {
				into_map_trait: { $($im:tt)+ }
				into_parallel_map_trait: { $($ipm:tt)+ }
			}
		} ) => {
			pub mod $mn { $($src)+ }
			pub use $mn::{ $($exp),+ };
			#[cfg(feature="parallel")]
			pub use $mn::{ $($exp_p),+ };

			make! {@each {
				modules: [ $($other_modules)* ]
				into_map: {
					into_map_trait: { $($im)+ }
					into_parallel_map_trait: { $($ipm)+ }
				}
			} }
		};
		(@each {
			modules: [ ]
			into_map: {
				into_map_trait: { $($im:tt)+ }
				into_parallel_map_trait: { $($ipm:tt)+ }
			}
		} ) => {

			/// 標準の `Iterator` を拡張して、イテレータ要素に対する様々な写像操作を行うイテレータアダプタを返すメソッドを提供します
			pub trait IntoMap: Sized
			{ $($im)+ }

			impl<I: Iterator> IntoMap for I {}

			#[cfg(feature="parallel")]
			/// rayon の並列イテレータ `ParallelIterator` を拡張して、イテレータ要素に対する様々な写像操作を行うイテレータアダプタを返すメソッドを提供します
			pub trait IntoParallelMap: Sized
			{ $($ipm)+ }

			#[cfg(feature="parallel")]
			impl<I: ParallelIterator> IntoParallelMap for I {}

		};
	}

	make! { [
		for_result: {
			item_type: { T,E: Result<T,E> }
			items: [
				{
					name_fn: map_ok
					name_iter_serial: MapOk
					name_iter_parallel: ParallelMapOk
					name_map_fn: MapOkFn
					desc: "イテレータ要素の `Result<T,E>` 型の `Ok` の部分の値 `T` を `U` に写像させて `Result<U,E>` にするイテレータを返します。 `Err` の場合はそのまま残します。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ U, F ]
					output_type: { Result<U,E> }
					where_serial: { F: FnMut(T) -> U }
					where_parallel: { F: Fn(T) -> U + Send + Sync }
					call: { self,input -> input.map(|i| self.0(i) ) }
				}
				{
					name_fn: map_ok_or
					name_iter_serial: MapOkOr
					name_iter_parallel: ParallelMapOkOr
					name_map_fn: MapOkOrFn
					desc: "イテレータ要素の `Result<T,E>` 型を `U` 型に写像させるイテレータを返します。入力が `Ok` の場合はクロージャにより `T` 型の値を `U` に写像させて出力し、 `Err` の場合はイテレータに与えられた `U` 型の値を複製して返します。"
					params: [ default:U, f:F ]
					map_fn_type_params: [ U, F ]
					impl_type_params: [ U, F ]
					output_type: { U }
					where_serial: { U: Clone, F: FnMut(T) -> U }
					where_parallel: { U: Clone + Send + Sync, F: Fn(T) -> U + Send + Sync }
					call: { self,input -> input.map_or_else(|_| self.0.clone(),|i| self.1(i) ) }
				}
				{
					name_fn: map_err
					name_iter_serial: MapErr
					name_iter_parallel: ParallelMapErr
					name_map_fn: MapErrFn
					desc: "イテレータ要素の `Result<T,E>` 型の `Err` の部分の値 `E` を `G` に写像させて `Result<T,G>` にするイテレータを返します。 `Ok` の場合はそのまま残します。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ G, F ]
					output_type: { Result<T,G> }
					where_serial: { F: FnMut(E) -> G }
					where_parallel: { F: Fn(E) -> G + Send + Sync }
					call: { self,input -> input.map_err(|i| self.0(i) ) }
				}
				{
					name_fn: map_err_or
					name_iter_serial: MapErrOr
					name_iter_parallel: ParallelMapErrOr
					name_map_fn: MapErrOrFn
					desc: "イテレータ要素の `Result<T,E>` 型を `U` 型に写像させるイテレータを返します。入力が `Err` の場合はクロージャにより `E` 型の値を `U` に写像させて出力し、 `Ok` の場合はイテレータに与えられた `U` 型の値を複製して返します。"
					params: [ f:F, default:U ]
					map_fn_type_params: [ F, U ]
					impl_type_params: [ F, U ]
					output_type: { U }
					where_serial: { F: FnMut(E) -> U, U: Clone }
					where_parallel: { F: Fn(E) -> U + Send + Sync, U: Clone + Send + Sync }
					call: { self,input -> input.map_or_else(|i| self.0(i),|_| self.1.clone() ) }
				}
				{
					name_fn: map_ok_or_else
					name_iter_serial: MapOkOrElse
					name_iter_parallel: ParallelMapOkOrElse
					name_map_fn: MapOkOrElseFn
					desc: "イテレータ要素の `Result<T,E>` 型を `U` 型に写像させるイテレータを返します。入力が `Ok` の場合はクロージャ `FO` により `T` 型の値を `U` に写像させて出力し、入力が `Err` の場合はクロージャ `FE` により `U` 型に写像させて出力します。"
					params: [ fn_err:FE, fn_ok:FO ]
					map_fn_type_params: [ FE, FO ]
					impl_type_params: [ U, FE, FO ]
					output_type: { U }
					where_serial: { FE: FnMut(E) -> U, FO: FnMut(T) -> U }
					where_parallel: { FE: Fn(E) -> U + Send + Sync, FO: Fn(T) -> U + Send + Sync }
					call: { self,input -> input.map_or_else(|i| self.0(i),|i| self.1(i) ) }
				}
				{
					name_fn: unwrap_ok_or
					name_iter_serial: UnwrapOkOr
					name_iter_parallel: ParallelUnwrapOkOr
					name_map_fn: UnwrapOkOrFn
					desc: "イテレータ要素の `Result<T,E>` 型をアンラップして `T` 型にするイテレータを返します。入力が `Ok` の場合は `T` 型の内包データが出力され、入力が `Err` の場合は引数に与えられた `T` 型の値を複製して返します。"
					params: [ default:T ]
					map_fn_type_params: [ T ]
					output_type: { T }
					where_serial: { T: Clone }
					where_parallel: { T: Clone + Send + Sync }
					call: { self,input -> input.map_or_else(|_| self.0.clone(), |i| i ) }
				}
				{
					name_fn: unwrap_ok_or_else
					name_iter_serial: UnwrapOkOrElse
					name_iter_parallel: ParallelUnwrapOkOrElse
					name_map_fn: UnwrapOkOrElseFn
					desc: "イテレータ要素の `Result<T,E>` 型をアンラップして `T` 型にするイテレータを返します。入力が `Ok` の場合は `T` 型の内包データが出力され、入力が `Err` の場合はクロージャを実行し、 `T` 型の返値を返します。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ F ]
					output_type: { T }
					where_serial: { F: FnMut(E) -> T }
					where_parallel: { F: Fn(E) -> T + Send + Sync }
					call: { self,input -> input.map_or_else(|i| self.0(i),|i| i ) }
				}
				{
					name_fn: unwrap_ok_or_default
					name_iter_serial: UnwrapOkOrDefault
					name_iter_parallel: ParallelUnwrapOkOrDefault
					name_map_fn: UnwrapOkOrDefaultFn
					desc: "イテレータ要素の `Result<T,E>` 型をアンラップして `T` 型にするイテレータを返します。入力が `Ok` の場合は `T` 型の内包データが出力され、入力が `Err` の場合は `T` のデフォルト値を返します。"
					output_type: { T }
					where_serial: { T: Default }
					where_parallel: { T: Default }
					call: { self,input -> input.map_or_else(|_| T::default(), |i| i ) }
				}
				{
					name_fn: unwrap_err_or
					name_iter_serial: UnwrapErrOr
					name_iter_parallel: ParallelUnwrapErrOr
					name_map_fn: UnwrapErrOrFn
					desc: "イテレータ要素の `Result<T,E>` 型をアンラップして `E` 型にするイテレータを返します。入力が `Err` の場合は `E` 型の内包データが出力され、入力が `Ok` の場合は引数に与えられた `E` 型の値を複製して返します。"
					params: [ default:E ]
					map_fn_type_params: [ E ]
					output_type: { E }
					where_serial: { E: Clone }
					where_parallel: { E: Clone + Send + Sync }
					call: { self,input -> input.map_or_else(|i| i, |_| self.0.clone() ) }
				}
				{
					name_fn: unwrap_err_or_else
					name_iter_serial: UnwrapErrOrElse
					name_iter_parallel: ParallelUnwrapErrOrElse
					name_map_fn: UnwrapErrOrElseFn
					desc: "イテレータ要素の `Result<T,E>` 型をアンラップして `E` 型にするイテレータを返します。入力が `Err` の場合は内包データが出力され、入力が `Ok` の場合はクロージャを実行し、 `E` 型の返値を返します。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ F ]
					output_type: { E }
					where_serial: { F: FnMut(T) -> E }
					where_parallel: { F: Fn(T) -> E + Send + Sync }
					call: { self,input -> input.map_or_else(|i| i,|i| self.0(i) ) }
				}
				{
					name_fn: unwrap_err_or_default
					name_iter_serial: UnwrapErrOrDefault
					name_iter_parallel: ParallelUnwrapErrOrDefault
					name_map_fn: UnwrapErrOrDefaultFn
					desc: "イテレータ要素の `Result<T,E>` 型をアンラップして `E` 型にするイテレータを返します。入力が `Err` の場合は `E` 型の内包データが出力され、入力が `Ok` の場合は `E` のデフォルト値を返します。"
					output_type: { E }
					where_serial: { E: Default }
					where_parallel: { E: Default }
					call: { self,input -> input.map_or_else(|i| i, |_| E::default() ) }
				}
				{
					name_fn: ok_and_then
					name_iter_serial: OkAndThen
					name_iter_parallel: ParallelOkAndThen
					name_map_fn: OkAndThenFn
					desc: "イテレータ要素の `Result<T,E>` 型に対して論理積をとるイテレータを返します。つまり新しいイテレータは、入力の `Result<T,E>` 型が `Ok` の場合にクロージャを実行し、その返値が `Ok` の場合のみ `Ok` を返します。クロージャの返値が `Err` の場合と入力が `Err` の場合は `Err` を返します。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ U, F ]
					output_type: { Result<U,E> }
					where_serial: { F: FnMut(T) -> Result<U,E> }
					where_parallel: { F: Fn(T) -> Result<U,E> + Send + Sync }
					call: { self,input -> input.and_then(|i| self.0(i) ) }
				}
				{
					name_fn: ok_or_else
					name_iter_serial: OkOrElse
					name_iter_parallel: ParallelOkOrElse
					name_map_fn: OkOrElseFn
					desc: "イテレータ要素の `Result<T,E>` 型に対して論理和をとるイテレータを返します。つまり新しいイテレータは、入力の `Result<T,E>` 型が `Ok` の場合はそのまま返され、 `Err` の場合にクロージャを実行し、その返値が返されます。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ G, F ]
					output_type: { Result<T,G> }
					where_serial: { F: FnMut(E) -> Result<T,G> }
					where_parallel: { F: Fn(E) -> Result<T,G> + Send + Sync }
					call: { self,input -> input.or_else(|i| self.0(i) ) }
				}
			]
		}
		for_option: {
			item_type: { T: Option<T> }
			items: [
				{
					name_fn: map_some
					name_iter_serial: MapSome
					name_iter_parallel: ParallelMapSome
					name_map_fn: MapSomeFn
					desc: "イテレータ要素の `Option<T>` 型を `Option<U>` 型に写像するイテレータを返します。入力が `Some` の場合はクロージャにより `T` 型の値を `U` に写像させて `Some` として返し、入力が `None` の場合はそのまま返します。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ U, F ]
					output_type: { Option<U> }
					where_serial: { F: FnMut(T) -> U }
					where_parallel: { F: Fn(T) -> U + Send + Sync }
					call: { self,input -> input.map(|i| self.0(i) ) }
				}
				{
					name_fn: unwrap_some_or
					name_iter_serial: UnwrapSomeOr
					name_iter_parallel: ParallelUnwrapSomeOr
					name_map_fn: UnwrapSomeOrFn
					desc: "イテレータ要素の `Option<T>` 型をアンラップして `T` 型にするイテレータを返します。入力が `Some` の場合は `T` 型の内包データが返され、入力が `None` の場合はイテレータに与えられた値を複製して返します。"
					params: [ default:T ]
					map_fn_type_params: [ T ]
					output_type: { T }
					where_serial: { T: Clone }
					where_parallel: { T: Clone + Send + Sync }
					call: { self,input -> input.unwrap_or_else(|| self.0.clone() ) }
				}
				{
					name_fn: unwrap_some_or_else
					name_iter_serial: UnwrapSomeOrElse
					name_iter_parallel: ParallelUnwrapSomeOrElse
					name_map_fn: UnwrapSomeOrElseFn
					desc: "イテレータ要素の `Option<T>` 型をアンラップして `T` 型にするイテレータを返します。入力が `Some` の場合は `T` 型の内包データが返され、入力が `None` の場合はクロージャを実行し、 `T` 型の返値を返します。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ F ]
					output_type: { T }
					where_serial: { F: FnMut() -> T }
					where_parallel: { F: Fn() -> T + Send + Sync }
					call: { self,input -> input.unwrap_or_else(|| self.0() ) }
				}
				{
					name_fn: unwrap_some_or_default
					name_iter_serial: UnwrapSomeOrDefault
					name_iter_parallel: ParallelUnwrapSomeOrDefault
					name_map_fn: UnwrapSomeOrDefaultFn
					desc: "イテレータ要素の `Option<T>` 型をアンラップして `T` 型にするイテレータを返します。入力が `Some` の場合は `T` 型の内包データが返され、入力が `None` の場合は `T` のデフォルト値を返します。"
					output_type: { T }
					where_serial: { T: Default }
					where_parallel: { T: Default }
					call: { self,input -> input.unwrap_or_default() }
				}
				{
					name_fn: some_and_then
					name_iter_serial: SomeAndThen
					name_iter_parallel: ParallelSomeAndThen
					name_map_fn: SomeAndThenFn
					desc: "イテレータ要素の `Option<T>` 型に対して論理積をとるイテレータを返します。つまり新しいイテレータは、入力の `Option<T>` 型が `Some` の場合にクロージャを実行し、その返値が `Some` の場合のみ `Some` を返します。クロージャの返値が `None` の場合と、入力が `None` の場合は `None` を返します。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ U, F ]
					output_type: { Option<U> }
					where_serial: { F: FnMut(T) -> Option<U> }
					where_parallel: { F: Fn(T) -> Option<U> + Send + Sync }
					call: { self,input -> input.and_then(|i| self.0(i) ) }
				}
				{
					name_fn: some_or_else
					name_iter_serial: SomeOrElse
					name_iter_parallel: ParallelSomeOrElse
					name_map_fn: SomeOrElseFn
					desc: "イテレータ要素の `Option<T>` 型に対して論理和をとるイテレータを返します。つまり新しいイテレータは、入力の `Option<T>` 型が `Some` の場合はそのまま返され、 `None` の場合にクロージャを実行し、その返値が返されます。"
					params: [ f:F ]
					map_fn_type_params: [ F ]
					impl_type_params: [ F ]
					output_type: { Option<T> }
					where_serial: { F: FnMut() -> Option<T> }
					where_parallel: { F: Fn() -> Option<T> + Send + Sync }
					call: { self,input -> input.or_else(|| self.0() ) }
				}
			]
		}
		for_impl_into: {
			item_type: { T: T }
			items: [
				{
					name_fn: map_into
					name_iter_serial: MapInto
					name_iter_parallel: ParallelMapInto
					name_map_fn: MapIntoFn
					desc: "イテレータ要素の `T` 型を `U` 型にするイテレータを返します。変換にあたっては `Into` トレイトに依拠します。"
					phantom_params: [ U ]
					map_fn_type_params: [ U ]
					impl_type_params: [ U ]
					output_type: { U }
					where_serial: { T: Into<U> }
					where_parallel: { T: Into<U>, U: Send + Sync }
					call: { self,input -> input.into() }
				}
			]
		}
		for_const_fn: {
			item_type: { T: T }
			items: [
				{
					name_fn: map_const_fn
					name_iter_serial: MapConst
					name_iter_parallel: ParallelMapConst
					name_map_fn: MapConstFn
					desc: "与えられた関数を実行してイテレータ要素の `T` 型を `U` 型にするイテレータを返します。通常の `.map(|T|->U)` とは異なり、引数として受け取れる関数は関数ポインタのみです。従って制約があるものの、イテレータアダプタの型を簡素にできます。"
					params: [ f: fn(T) -> U ]
					map_fn_type_params: [ T, U ]
					impl_type_params: [ U ]
					output_type: { U }
					call: { self,input -> self.0(input) }
				}
			]
		}
	] }

}
pub use iter_impl::*;

/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::{
		for_serial_iter::{
			ExtendedMap as ExtendedMapForIterator,
			ExtendedMapFn as ExtendedMapFnForIterator
		},
		IntoMap
	};
	#[cfg(feature="parallel")]
	pub use super::{
		for_parallel_iter::{
			ExtendedMap as ExtendedMapForParallelIterator,
			ExtendedMapFn as ExtendedMapFnForParallelIterator
		},
		IntoParallelMap
	};
}
