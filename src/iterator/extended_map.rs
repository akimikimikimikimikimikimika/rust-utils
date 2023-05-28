//! * イテレータに対する通常の `map` 関数を拡張したイテレータを提供するモジュール
//! * 具体的には `Result<T,E>` → `Result<U,E>` をクロージャ `Fn(T)->U` により写像させる `map_ok` や、 `T: Into<U>` を用いて `T` → `U` を写像させる `map_into` などが含まれる。
//! * また、カスタムの写像定義を作りやすいように構築されている。
//! * `itertools` の `map_ok` や　`map_into` の実装と同じく、直列/並列ごとに一般的なイテレータを実装してから、 `map_ok` や `map_into` それぞれごとの特殊化した機能を組み込んでいる。

use super::*;

/// 直列イテレータを写像する
pub mod for_serial_iter {
	use super::*;

	/// イテレータを写像するイテレータ
	pub struct ExtendedMap<I,F> {
		pub(super) iter: I,
		pub(super) map_fn: F
	}
	use ExtendedMap as Map;

	/// イテレータの写像関数を定義するトレイト
	pub trait ExtendedMapFn<Input> {
		type Output;
		/// 写像の仕方を定義するメソッド。外の変数をミュータブルにキャプチャしていても構わない
		fn call_mut(&mut self,input:Input) -> Self::Output;
	}
	use ExtendedMapFn as MapFn;

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

	// ここでは独自の ParallelIterator を定義する際の見本となるように、 ParallelIterator を構成する各要素のい意義が分かりやすいよう、多くのコメントを付している

	/// 並列イテレータの写像関数を定義するトレイト。利便性のために直列の写像関数のトレイトを継承しており、スレッド間のデータ移動にも対応していなければならない。
	pub trait ExtendedMapFn<Input>: MapFnSerial<Input> + Sync + Send {
		fn call(&self,input:Input) -> Self::Output;
	}
	use ExtendedMapFn as MapFn;
	use for_serial_iter::ExtendedMapFn as MapFnSerial;

	/// 並列イテレータを写像する並列イレテータ
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

	macro_rules! make {
		(
			item_type: { $($it:tt)+ }
			items: [ $($items:tt)+ ]
		) => {
			make!{@each
				item_type: { $($it)+ }
				items: [ $($items)+ ]
				into_map_trait: {} into_map_impl: {}
				into_parallel_map_trait: {} into_parallel_map_impl: {}
			}
		};
		(@each
			item_type: { $($t:ident),+: $ti:ty }
			items: [ {
				name_fn: $nf:ident
				name_iter_serial: $nis:ident
				name_iter_parallel: $nip:ident
				name_map_fn: $nmf:ident
				$( desc: $desc:literal )?
				$( params: [ $( $pi:tt: $pt:ident ),+ ] )?
				$( phantom_params: [ $( $ppt:ident ),+ ] )?
				$( type_params: [ $($tp:ident),+ ] )?
				output_type: { $to:ty }
				$( where_serial: { $($ws:tt)+ } )?
				$( where_parallel: { $($wp:tt)+ } )?
				call: { $si:ident, $ii:ident -> $($body:tt)+ }
			} $($other_items:tt)* ]
			into_map_trait: { $($im_t:tt)* }
			into_map_impl: { $($im_i:tt)* }
			into_parallel_map_trait: { $($ipm_t:tt)* }
			into_parallel_map_impl: { $($ipm_i:tt)* }
		) => {

			$( #[doc=concat!(
				"`",stringify!($nf),"()` にて生成されるイテレータを構成する `ExtendedMap` 向けの関数。", $desc
			)] )?
			pub struct $nmf
			<$($($pt,)+)? $($($ppt,)+)?>
			($($($pt,)+)? $($(PhantomData<$ppt>,)+)?);

			impl<$($t,)+ $($($tp,)+)?> MapFn<$ti>
			for $nmf <$($($pt,)+)? $($($ppt,)+)?>
			$( where $($ws)+ )?
			{
				type Output = $to;
				fn call_mut(&mut $si,$ii:$ti) -> $to {
					$($body)+
				}
			}

			#[cfg(feature="parallel")]
			impl<$($t,)+ $($($tp,)+)?> ParallelMapFn<$ti>
			for $nmf <$($($pt,)+)? $($($ppt,)+)?>
			$( where $($wp)+ )?
			{
				fn call(&$si,$ii:$ti) -> $to {
					$($body)+
				}
			}

			$( #[doc=concat!(
				"`",stringify!($nf),"()` にて生成されるイテレータ。", $desc
			)] )?
			pub type $nis<I $($(,$pt)+)? $($(,$ppt)+)?> = Map<I,$nmf <$($($pt,)+)? $($($ppt,)+)?>>;

			#[cfg(feature="parallel")]
			$( #[doc=concat!(
				"`",stringify!($nf),"()` にて生成される並列イテレータ。", $desc
			)] )?
			pub type $nip<I $($(,$pt)+)? $($(,$ppt)+)?> = ParallelMap<I,$nmf <$($($pt,)+)? $($($ppt,)+)?>>;

			make!{@each
				item_type: { $($t),+: $ti }
				items: [ $($other_items)* ]
				into_map_trait: { $($im_t)*
					$( #[doc=$desc] )?
					fn $nf $(<$($tp),+>)? (self $($(,$pi:$pt)+)?)
					-> $nis<Self $($(,$pt)+)? $($(,$ppt)+)?>
					$( where $($ws)+ )?;
				}
				into_map_impl: { $($im_i)*
					fn $nf $(<$($tp),+>)? (self $($(,$pi:$pt)+)?)
					-> $nis<Self $($(,$pt)+)? $($(,$ppt)+)?>
					$( where $($ws)+ )?
					{ Map { iter: self, map_fn: $nmf ($($($pi,)+)? $($(PhantomData::<$ppt>,)+)?) } }
				}
				into_parallel_map_trait: { $($ipm_t)*
					$( #[doc=$desc] )?
					fn $nf $(<$($tp),+>)? (self $($(,$pi:$pt)+)?)
					-> $nip<Self $($(,$pt)+)? $($(,$ppt)+)?>
					$( where $($wp)+ )?;
				}
				into_parallel_map_impl: { $($ipm_i)*
					fn $nf $(<$($tp),+>)? (self $($(,$pi:$pt)+)?)
					-> $nip<Self $($(,$pt)+)? $($(,$ppt)+)?>
					$( where $($wp)+ )?
					{ ParallelMap { parent_iterator: self, map_fn: $nmf ($($($pi,)+)? $($(PhantomData::<$ppt>,)+)?) } }
				}
			}

		};
		(@each
			item_type: { $($t:ident),+: $ti:ty }
			items: []
			into_map_trait: { $($im_t:tt)+ }
			into_map_impl: { $($im_i:tt)+ }
			into_parallel_map_trait: { $($ipm_t:tt)+ }
			into_parallel_map_impl: { $($ipm_i:tt)+ }
		) => {

			/// イテレータを拡張して、写像関連のイテレータを生成するメソッドを提供するトレイト
			pub trait IntoMap<$($t),+>: Sized
			{ $($im_t)+ }

			impl<I $(,$t)+> IntoMap<$($t),+> for I
			where I: Iterator<Item=$ti>
			{ $($im_i)+ }

			#[cfg(feature="parallel")]
			/// 並列イテレータを拡張して、写像関連のイテレータを生成するメソッドを提供するトレイト
			pub trait IntoParallelMap<$($t),+>: Sized
			{ $($ipm_t)+ }

			#[cfg(feature="parallel")]
			impl<I $(,$t)+> IntoParallelMap<$($t),+> for I
			where I: ParallelIterator<Item=$ti>
			{ $($ipm_i)+ }

		};
	}

	pub mod for_result {
		use super::*;

		make! {
			item_type: { T,E: Result<T,E> }
			items: [
				{
					name_fn: map_ok
					name_iter_serial: MapOk
					name_iter_parallel: ParallelMapOk
					name_map_fn: MapOkFn
					desc: "`Result<T,E>` 型の `Ok` の部分の値 `T` を `U` に写像させて `Result<U,E>` にする。 `Err` の場合はそのまま返される。"
					params: [ f:F ]
					type_params: [ U, F ]
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
					desc: "`Result<T,E>` 型を `U` 型に写像する。入力が `Ok` の場合はクロージャにより `T` 型の値を `U` に写像させて出力し、 `Err` の場合はイテレータに与えられた `U` 型の値を複製して出力する。"
					params: [ default:U, f:F ]
					type_params: [ U, F ]
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
					desc: "`Result<T,E>` 型の `Err` の部分の値 `E` を `G` に写像させて `Result<T,G>` にする。 `Ok` の場合はそのまま返される。"
					params: [ f:F ]
					type_params: [ G, F ]
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
					desc: "`Result<T,E>` 型を `U` 型に写像する。入力が `Err` の場合はクロージャにより `E` 型の値を `U` に写像させて出力し、 `Ok` の場合はイテレータに与えられた `U` 型の値を複製して出力する。"
					params: [ f:F, default:U ]
					type_params: [ F, U ]
					output_type: { U }
					where_serial: { F: FnMut(E) -> U, U: Clone }
					where_parallel: { F: Fn(E) -> U + Send + Sync, U: Clone + Send + Sync }
					call: { self,input -> input.map_or_else(|i| self.0(i),|_| self.1.clone() ) }
				}
				{
					name_fn: map_or_else
					name_iter_serial: MapOrElse
					name_iter_parallel: ParallelMapOrElse
					name_map_fn: MapOrElseFn
					desc: "`Result<T,E>` 型を `U` 型に写像する。入力が `Ok` の場合はクロージャ `FO` により `T` 型の値を `U` に写像させて出力し、入力が `Err` の場合はクロージャ `FE` により `U` 型に写像させて出力する。"
					params: [ fn_err:FE, fn_ok:FO ]
					type_params: [ U, FE, FO ]
					output_type: { U }
					where_serial: { FE: FnMut(E) -> U, FO: FnMut(T) -> U }
					where_parallel: { FE: Fn(E) -> U + Send + Sync, FO: Fn(T) -> U + Send + Sync }
					call: { self,input -> input.map_or_else(|i| self.0(i),|i| self.1(i) ) }
				}
				{
					name_fn: unwrap_or
					name_iter_serial: UnwrapOr
					name_iter_parallel: ParallelUnwrapOr
					name_map_fn: UnwrapOrFn
					desc: "`Result<T,E>` 型をアンラップする。入力が `Ok` の場合は `T` 型の内包データが出力され、入力が `Err` の場合はイテレータに与えられた値を複製して出力する。"
					params: [ default:T ]
					output_type: { T }
					where_serial: { T: Clone }
					where_parallel: { T: Clone + Send + Sync }
					call: { self,input -> input.map_or_else(|_| self.0.clone(), |i| i ) }
				}
				{
					name_fn: unwrap_or_else
					name_iter_serial: UnwrapOrElse
					name_iter_parallel: ParallelUnwrapOrElse
					name_map_fn: UnwrapOrElseFn
					desc: "`Result<T,E>` 型をアンラップする。入力が `Ok` の場合は内包データが出力され、入力が `Err` の場合はクロージャを実行し、 `T` 型の返値を出力する。"
					params: [ f:F ]
					type_params: [ F ]
					output_type: { T }
					where_serial: { F: FnMut(E) -> T }
					where_parallel: { F: Fn(E) -> T + Send + Sync }
					call: { self,input -> input.map_or_else(|i| self.0(i),|i| i ) }
				}
				{
					name_fn: unwrap_or_default
					name_iter_serial: UnwrapOrDefault
					name_iter_parallel: ParallelUnwrapOrDefault
					name_map_fn: UnwrapOrDefaultFn
					desc: "`Result<T,E>` 型をアンラップする。入力が `Ok` の場合は `T` 型の内包データが出力され、入力が `Err` の場合は `T` のデフォルト値を出力する。"
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
					desc: "`Result<T,E>` 型をアンラップする。入力が `Err` の場合は `E` 型の内包データが出力され、入力が `Ok` の場合はイテレータに与えられた値を複製して出力する。"
					params: [ default:E ]
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
					desc: "`Result<T,E>` 型をアンラップする。入力が `Err` の場合は内包データが出力され、入力が `Ok` の場合はクロージャを実行し、 `E` 型の返値を出力する。"
					params: [ f:F ]
					type_params: [ F ]
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
					desc: "`Result<T,E>` 型をアンラップする。入力が `Err` の場合は `E` 型の内包データが出力され、入力が `Ok` の場合は `E` のデフォルト値を出力する。"
					output_type: { E }
					where_serial: { E: Default }
					where_parallel: { E: Default }
					call: { self,input -> input.map_or_else(|i| i, |_| E::default() ) }
				}
				{
					name_fn: and_then
					name_iter_serial: AndThen
					name_iter_parallel: ParallelAndThen
					name_map_fn: AndThenFn
					desc: "論理積をとる。入力の `Result<T,E>` 型が `Ok` の場合にクロージャを実行し、その返値が `Ok` の場合のみ `Ok` が出力される。返値が `Err` の場合と、入力が `Err` の場合は `Err` が出力される。"
					params: [ f:F ]
					type_params: [ U, F ]
					output_type: { Result<U,E> }
					where_serial: { F: FnMut(T) -> Result<U,E> }
					where_parallel: { F: Fn(T) -> Result<U,E> + Send + Sync }
					call: { self,input -> input.and_then(|i| self.0(i) ) }
				}
				{
					name_fn: or_else
					name_iter_serial: OrElse
					name_iter_parallel: ParallelOrElse
					name_map_fn: OrElseFn
					desc: "論理和をとる。入力の `Result<T,E>` 型が `Ok` の場合はそのまま出力され、 `Err` の場合にクロージャを実行し、その返値が出力される。"
					params: [ f:F ]
					type_params: [ G, F ]
					output_type: { Result<T,G> }
					where_serial: { F: FnMut(E) -> Result<T,G> }
					where_parallel: { F: Fn(E) -> Result<T,G> + Send + Sync }
					call: { self,input -> input.or_else(|i| self.0(i) ) }
				}
			]
		}
	}

	pub mod for_option {
		use super::*;

		make! {
			item_type: { T: Option<T> }
			items: [
				{
					name_fn: map_some
					name_iter_serial: MapSome
					name_iter_parallel: ParallelMapSome
					name_map_fn: MapSomeFn
					desc: "`Option<T>` 型を `Option<U>` 型に写像する。入力が `Some` の場合はクロージャにより `T` 型の値を `U` に写像させて `Some` として出力し、入力が `None` の場合はそのまま出力させる。"
					params: [ f:F ]
					type_params: [ U, F ]
					output_type: { Option<U> }
					where_serial: { F: FnMut(T) -> U }
					where_parallel: { F: Fn(T) -> U + Send + Sync }
					call: { self,input -> input.map(|i| self.0(i) ) }
				}
				{
					name_fn: map_some_or
					name_iter_serial: MapSomeOr
					name_iter_parallel: ParallelMapSomeOr
					name_map_fn: MapSomeOrFn
					desc: "`Option<T>` 型を `U` 型に写像する。入力が `Some` の場合はクロージャにより `T` 型の値を `U` に写像させて出力し、 `None` の場合はイテレータに与えられた `U` 型の値を複製して出力する。"
					params: [ default:U, f:F ]
					type_params: [ U, F ]
					output_type: { U }
					where_serial: { U: Clone, F: FnMut(T) -> U }
					where_parallel: { U: Clone + Send + Sync, F: Fn(T) -> U + Send + Sync }
					call: { self,input -> input.map_or_else(|| self.0.clone(),|i| self.1(i) ) }
				}
				{
					name_fn: map_some_or_else
					name_iter_serial: MapSomeOrElse
					name_iter_parallel: ParallelMapSomeOrElse
					name_map_fn: MapSomeOrElseFn
					desc: "`Option<T>` 型を `U` 型に写像する。入力が `Some` の場合はクロージャ `FS` により `T` 型の値を `U` に写像させて出力し、入力が `None` の場合はもう1つのクロージャ `FN` を実行して、 `U` 型の返値を出力する。"
					params: [ fn_none:FN, fn_some:FS ]
					type_params: [ U, FN, FS ]
					output_type: { U }
					where_serial: { FN: FnMut() -> U, FS: FnMut(T) -> U }
					where_parallel: { FN: Fn() -> U + Send + Sync, FS: Fn(T) -> U + Send + Sync }
					call: { self,input -> input.map_or_else(|| self.0(),|i| self.1(i) ) }
				}
				{
					name_fn: unwrap_or
					name_iter_serial: UnwrapOr
					name_iter_parallel: ParallelUnwrapOr
					name_map_fn: UnwrapOrFn
					desc: "`Option<T>` 型をアンラップする。入力が `Some` の場合は `T` 型の内包データが出力され、入力が `None` の場合はイテレータに与えられた値を複製して出力する。"
					params: [ default:T ]
					output_type: { T }
					where_serial: { T: Clone }
					where_parallel: { T: Clone + Send + Sync }
					call: { self,input -> input.unwrap_or_else(|| self.0.clone() ) }
				}
				{
					name_fn: unwrap_or_else
					name_iter_serial: UnwrapOrElse
					name_iter_parallel: ParallelUnwrapOrElse
					name_map_fn: UnwrapOrElseFn
					desc: "`Option<T>` 型をアンラップする。入力が `Some` の場合は内包データが出力され、入力が `None` の場合はクロージャを実行し、 `T` 型の返値を出力する。"
					params: [ f:F ]
					type_params: [ F ]
					output_type: { T }
					where_serial: { F: FnMut() -> T }
					where_parallel: { F: Fn() -> T + Send + Sync }
					call: { self,input -> input.unwrap_or_else(|| self.0() ) }
				}
				{
					name_fn: unwrap_or_default
					name_iter_serial: UnwrapOrDefault
					name_iter_parallel: ParallelUnwrapOrDefault
					name_map_fn: UnwrapOrDefaultFn
					desc: "`Option<T>` 型をアンラップする。入力が `Some` の場合は `T` 型の内包データが出力され、入力が `None` の場合は `T` のデフォルト値を出力する。"
					output_type: { T }
					where_serial: { T: Default }
					where_parallel: { T: Default }
					call: { self,input -> input.unwrap_or_default() }
				}
				{
					name_fn: and_then
					name_iter_serial: AndThen
					name_iter_parallel: ParallelAndThen
					name_map_fn: AndThenFn
					desc: "論理積をとる。入力の `Option<T>` 型が `Some` の場合にクロージャを実行し、その返値が `Some` の場合のみ `Some` が出力される。返値が `None` の場合と、入力が `None` の場合は `None` が出力される。"
					params: [ f:F ]
					type_params: [ U, F ]
					output_type: { Option<U> }
					where_serial: { F: FnMut(T) -> Option<U> }
					where_parallel: { F: Fn(T) -> Option<U> + Send + Sync }
					call: { self,input -> input.and_then(|i| self.0(i) ) }
				}
				{
					name_fn: or_else
					name_iter_serial: OrElse
					name_iter_parallel: ParallelOrElse
					name_map_fn: OrElseFn
					desc: "論理和をとる。入力の `Option<T>` 型が `Some` の場合はそのまま出力され、 `None` の場合にクロージャを実行し、その返値が出力される。"
					params: [ f:F ]
					type_params: [ F ]
					output_type: { Option<T> }
					where_serial: { F: FnMut() -> Option<T> }
					where_parallel: { F: Fn() -> Option<T> + Send + Sync }
					call: { self,input -> input.or_else(|| self.0() ) }
				}
			]
		}
	}

	pub mod for_impl_into {
		use super::*;

		make! {
			item_type: { T: T }
			items: [
				{
					name_fn: map_into
					name_iter_serial: MapInto
					name_iter_parallel: ParallelMapInto
					name_map_fn: MapIntoFn
					desc: "`Into` トレイトに依拠して `T` を `U` に変換する。"
					phantom_params: [ U ]
					type_params: [ U ]
					output_type: { U }
					where_serial: { T: Into<U> }
					where_parallel: { T: Into<U>, U: Send + Sync }
					call: { self,input -> input.into() }
				}
			]
		}
	}

}
pub use iter_impl::*;

/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::{
		for_serial_iter::{
			ExtendedMap as ExtendedMapForIterator,
			ExtendedMapFn as ExtendedMapFnForIterator
		},
		for_result::IntoMap as MapExtensionForResultIterator,
		for_option::IntoMap as MapExtensionForOptionIterator,
		for_impl_into::IntoMap as MapExtensionForImplIntoIterator
	};
	#[cfg(feature="parallel")]
	pub use super::{
		for_parallel_iter::{
			ExtendedMap as ExtendedMapForParallelIterator,
			ExtendedMapFn as ExtendedMapFnForParallelIterator
		},
		for_result::IntoParallelMap as MapExtensionForResultParallelIterator,
		for_option::IntoParallelMap as MapExtensionForOptionParallelIterator,
		for_impl_into::IntoParallelMap as MapExtensionForImplIntoParallelIterator
	};
}
