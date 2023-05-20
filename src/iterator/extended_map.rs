//! * イテレータに対する通常の `map` 関数を拡張したイテレータを提供するモジュール
//! * 具体的には `Result<T,E>` → `Result<U,E>` をクロージャ `Fn(T)->U` により写像させる `map_ok` や、 `T: Into<U>` を用いて `T` → `U` を写像させる `map_into` などが含まれる。
//! * また、カスタムの写像定義を作りやすいように構築されている。
//! * `itertools` の `map_ok` や　`map_into` の実装と同じく、直列/並列ごとに一般的なイテレータを実装してから、 `map_ok` や `map_into` それぞれごとの特殊化した機能を組み込んでいる。

use super::*;

/// 直列イテレータを写像する
mod for_serial_iter {
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
pub use for_serial_iter::{
	ExtendedMap as ExtendedMapForIterator,
	ExtendedMapFn as ExtendedMapFnForIterator
};


/// 並列イテレータを写像する
mod for_parallel_iter {
	use super::*;
	use rayon_plumbing::*;

	// ここでは独自の ParallelIterator を定義する際の見本となるように、 ParallelIterator を構成する各要素のい意義が分かりやすいよう、多くのコメントを付している

	/// 並列イテレータの写像関数を定義するトレイト。利便性のために直列の写像関数のトレイトを継承しており、スレッド間のデータ移動にも対応していなければならない。
	pub trait ExtendedMapFn<Input>: MapFnSerial<Input> + Sync + Send {
		fn call(&self,input:Input) -> Self::Output;
	}
	use ExtendedMapFn as MapFn;
	use ExtendedMapFnForIterator as MapFnSerial;

	/// 並列イテレータを写像する並列イレテータ
	pub struct ExtendedMap<I,F> {
		/// 1つ上の階層の `ParallelIterator`
		pub(super) parent_iterator: I,
		/// 写像関数
		pub(super) map_fn: F
	}
	use ExtendedMap as Map;
	use ExtendedMapForIterator as MapSerial;

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
			match iter.next() {
				Some(i) => Some(map_fn.call(i)),
				None => None
			}
		}

		fn nth(&mut self, n: usize) -> Option<Self::Item> {
			let Self { iter, ref map_fn } = self;
			match iter.nth(n) {
				Some(i) => Some(map_fn.call(i)),
				None => None
			}
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
pub use for_parallel_iter::{
	ExtendedMap as ExtendedMapForParallelIterator,
	ExtendedMapFn as ExtendedMapFnForParallelIterator
};



/// 具体的なケースに対して実装を行う
mod specific_cases {
	use super::*;
	use ExtendedMapForIterator as Map;
	use ExtendedMapForParallelIterator as ParallelMap;
	use ExtendedMapFnForIterator as MapFn;
	use ExtendedMapFnForParallelIterator as ParallelMapFn;

	pub struct MapOkFn<F>(F);

	impl<T,U,E,F> MapFn<Result<T,E>> for MapOkFn<F>
	where F: FnMut(T) -> U
	{
		type Output = Result<U,E>;
		fn call_mut(&mut self,input:Result<T,E>) -> Self::Output {
			input.map(|i| self.0(i) )
		}
	}

	impl<T,U,E,F> ParallelMapFn<Result<T,E>> for MapOkFn<F>
	where F: Fn(T) -> U + Sync + Send
	{
		fn call(&self,input:Result<T,E>) -> Self::Output {
			input.map(|i| self.0(i) )
		}
	}

	/// `.map_ok()` にて生成される、 `Result` 型の `Ok` の部分の値を写像させたイテレータ。
	pub type MapOk<I,F> = Map<I,MapOkFn<F>>;
	/// `.map_ok()` にて生成される、 `Result` 型の `Ok` の部分の値を写像させた並列イテレータ。
	pub type ParallelMapOk<I,F> = ParallelMap<I,MapOkFn<F>>;

	pub struct MapErrFn<F>(F);

	impl<T,E,G,F> MapFn<Result<T,E>> for MapErrFn<F>
	where F: FnMut(E) -> G
	{
		type Output = Result<T,G>;
		fn call_mut(&mut self,input:Result<T,E>) -> Self::Output {
			input.map_err(|i| self.0(i) )
		}
	}

	impl<T,E,G,F> ParallelMapFn<Result<T,E>> for MapErrFn<F>
	where F: Fn(E) -> G + Sync + Send
	{
		fn call(&self,input:Result<T,E>) -> Self::Output {
			input.map_err(|i| self.0(i) )
		}
	}

	/// `.map_err()` にて生成される、 `Result` 型の `Err` の部分の値を写像させたイテレータ。
	pub type MapErr<I,F> = Map<I,MapErrFn<F>>;
	/// `.map_err()` にて生成される、 `Result` 型の `Err` の部分の値を写像させた並列イテレータ。
	pub type ParallelMapErr<I,F> = ParallelMap<I,MapErrFn<F>>;

	pub struct MapSomeFn<F>(F);

	impl<T,U,F> MapFn<Option<T>> for MapSomeFn<F>
	where F: FnMut(T) -> U
	{
		type Output = Option<U>;
		fn call_mut(&mut self,input:Option<T>) -> Self::Output {
			input.map(|i| self.0(i) )
		}
	}

	impl<T,U,F> ParallelMapFn<Option<T>> for MapSomeFn<F>
	where F: Fn(T) -> U + Sync + Send
	{
		fn call(&self,input:Option<T>) -> Self::Output {
			input.map(|i| self.0(i) )
		}
	}

	/// `.map_some()` にて生成される、 `Option` 型の `Some` の部分の値を写像させたイテレータ。
	pub type MapSome<I,F> = Map<I,MapSomeFn<F>>;
	/// `.map_some()` にて生成される、 `Option` 型の `Some` の部分の値を写像させた並列イテレータ。
	pub type ParallelMapSome<I,F> = ParallelMap<I,MapSomeFn<F>>;

	use std::marker::PhantomData;
	pub struct MapIntoFn<T>(PhantomData<T>);

	impl<T,U> MapFn<T> for MapIntoFn<U>
	where T: Into<U> {
		type Output = U;
		fn call_mut(&mut self,input:T) -> Self::Output { input.into() }
	}

	impl<T,U> ParallelMapFn<T> for MapIntoFn<U>
	where T: Into<U>, U: Sync + Send {
		fn call(&self,input:T) -> Self::Output { input.into() }
	}

	/// `.map_into()` にて生成される、 `Into` トレイトに依拠して型の変換を行うイテレータ。
	pub type MapInto<I,T> = Map<I,MapIntoFn<T>>;
	/// `.map_into()` にて生成される、 `Into` トレイトに依拠して型の変換を行う並列イテレータ。
	pub type ParallelMapInto<I,T> = ParallelMap<I,MapIntoFn<T>>;

	/// イテレータを拡張して、写像関連のイテレータを生成するメソッドを提供するトレイト
	pub trait MapExtension: Iterator + Sized {

		/// `Result<T,E>` 型のイテレータ要素の `T` を `U` に変換して `Result<U,E>` のイテレータにするトレイト
		fn map_ok<F,T,U,E>(self,f:F) -> MapOk<Self,F>
		where
			Self: Iterator<Item=Result<T,E>>,
			F: FnMut(T) -> U
		{ Map { iter: self, map_fn: MapOkFn(f) } }

		/// `Result<T,E>` 型のイテレータ要素の `E` を `G` に変換して `Result<T,G>` のイテレータにするトレイト
		fn map_err<F,T,E,G>(self,f:F) -> MapErr<Self,F>
		where
			Self: Iterator<Item=Result<T,E>>,
			F: FnMut(E) -> G
		{ Map { iter: self, map_fn: MapErrFn(f) } }

		/// `Option<T>` 型のイテレータ要素の `T` を `U` に変換して `Option<U>` のイテレータにするトレイト
		fn map_some<F,T,U>(self,f:F) -> MapSome<Self,F>
		where
			Self: Iterator<Item=Option<T>>,
			F: FnMut(T) -> U
		{ Map { iter: self, map_fn: MapSomeFn(f) } }

		/// `Into` トレイトを使用してイテレータの要素を別の要素に変換するトレイト
		fn map_into<U>(self) -> MapInto<Self,U>
		where Self::Item: Into<U>
		{ Map { iter: self, map_fn: MapIntoFn(PhantomData) } }

	}

	/// 並列イテレータを拡張して、写像関連のイテレータを生成するメソッドを提供するトレイト
	pub trait ParallelMapExtension: ParallelIterator + Sized {

		/// `Result<T,E>` 型のイテレータ要素の `T` を `U` に変換して `Result<U,E>` のイテレータにするトレイト
		fn map_ok<F,T,U,E>(self,f:F) -> ParallelMapOk<Self,F>
		where
			Self: ParallelIterator<Item=Result<T,E>>,
			F: Fn(T) -> U
		{ ParallelMap { parent_iterator: self, map_fn: MapOkFn(f) } }

		/// `Result<T,E>` 型のイテレータ要素の `E` を `G` に変換して `Result<T,G>` のイテレータにするトレイト
		fn map_err<F,T,E,G>(self,f:F) -> ParallelMapErr<Self,F>
		where
			Self: ParallelIterator<Item=Result<T,E>>,
			F: Fn(E) -> G
		{ ParallelMap { parent_iterator: self, map_fn: MapErrFn(f) } }

		/// `Option<T>` 型のイテレータ要素の `T` を `U` に変換して `Option<U>` のイテレータにするトレイト
		fn map_some<F,T,U>(self,f:F) -> ParallelMapSome<Self,F>
		where
			Self: ParallelIterator<Item=Option<T>>,
			F: Fn(T) -> U
		{ ParallelMap { parent_iterator: self, map_fn: MapSomeFn(f) } }

		/// `Into` トレイトを使用してイテレータの要素を別の要素に変換するトレイト
		fn map_into<U>(self) -> ParallelMapInto<Self,U>
		where Self::Item: Into<U>
		{ ParallelMap { parent_iterator: self, map_fn: MapIntoFn(PhantomData) } }

	}

}
pub use specific_cases::{
	MapExtension, ParallelMapExtension
};
