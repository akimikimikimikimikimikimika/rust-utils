use super::*;

mod for_iters_tuple {

	/// 複数のイテレータのカーテジアン積をとったイテレータ
	pub struct CartesianProduct<I,O,V> {
		pub(crate) iters_tuple: I,
		pub(crate) iters_original_tuple: O,
		pub(crate) current_val_tuple: Option<V>
	}
	type Product<I,O,V> = CartesianProduct<I,O,V>;

	/// 複数のイテレータのタプルをカーテジアン積をとった単一のイテレータに変換するトレイト
	pub trait IntoIter<O,V>: Sized {
		/// イテレータのタプル `(I1,I2,I3,...)` をカーテジアン積をとったイテレータ `Iterator<Item=(T1,T2,T3,...)` に変換します。各イテレータが `Clone` を実装していなければなりません。
		fn cartesian_product(self) -> Product<Self,O,V>;
	}

}
pub use for_iters_tuple::{
	CartesianProduct as ProductForIteratorsTuple,
	IntoIter as IntoTupleProductIterator
};

/// * イテレータの要素数ごとに `CartesianProduct` を実装するマクロ
/// * `impl_product_iters!( I0 T0 0 I1 T1 1 I2 T2 2 ... I(N-1) T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
/// * `I*` `T*` の異なる型パラメータとタプルのインデクスをこの順で並べていく
macro_rules! impl_product_iters {
	// マクロのエントリポイント: 全ての実装をモジュールで囲む
	( $( $i:ident $t:ident $n:tt )+ ) => {
		mod impl_product_iters {
			use super::{
				ProductForIteratorsTuple as Product,
				IntoTupleProductIterator as IntoIter,
				*
			};

			type UL = (usize,Option<usize>);
			/// サイズヒントの計算に役立つ積と和の計算
			fn size_hint_mul_add(a:UL,b:UL,c:UL) -> UL {
				(
					a.0 * b.0 + c.0,
					(a.1,b.1,c.1)
					.zip_options()
					.map(|(a,b,c)| a * b + c )
				)
			}

			impl_product_iters! {@process $( $i $t $n )+ }
		}
	};

	// 引数を分離するプロセス: エントリポイント
	(@process
		$i0:ident $t0:ident $n0:tt
		$( $i:ident $t:ident $n:tt )*
	) => {
		impl_product_iters! {@process | $i0 $t0 $n0 | $( $i $t $n )* }
	};
	// 引数を分離するプロセス: `|` により3つに分かれた領域のうち、前の2つを残す場合と、後ろから1つずつ要素をずらした場合に分ける
	// 1つ目の領域: impl するアイテムの末尾以外
	// 2つ目の領域: impl するアイテムの末尾
	// 3つ目の領域: 残りのアイテム
	(@process
		$( $i:ident $t:ident $n:tt )* |
		$il:ident $tl:ident $nl:tt |
		$in:ident $tn:ident $nn:tt
		$( $others:tt )*
	) => {
		impl_product_iters! {@process
			$( $i $t $n )* | $il $tl $nl |
		}
		impl_product_iters! {@process
			$( $i $t $n )* $il $tl $nl |
			$in $tn $nn | $( $others )*
		}
	};
	// 引数を分離するプロセス: impl するアイテムの数が1つの場合 → 後述の one_impl に渡す
	(@process | $i:ident $t:ident $n:tt | ) => {
		impl_product_iters! {@one_impl}
		impl_product_iters! {@additional_impl I }
	};
	// 引数を分離するプロセス: impl するアイテムの数が複数ある場合 → 引数を加工した上で後述の many_impl に渡す
	(@process
		$if:ident $tf:ident $nf:tt
		$( $i:ident $t:ident $n:tt )* |
		$il:ident $tl:ident $nl:tt |
	) => {
		impl_product_iters! {@many_impl
			$if $tf $nf
			$( $i $t $n )*
			$il $tl $nl |
			$if $tf $nf
			$( $i $t $n )* |
			$( $i $t $n )*
			$il $tl $nl |
			$nf $nl
		}
		impl_product_iters! {@additional_impl $if $($i)* $il }
	};

	// イテレータの数が1つの場合の実装
	(@one_impl) => {

		impl<T,I> IntoIter<(),()> for (I,)
		where
			I: Iterator<Item=T>
		{
			fn cartesian_product(self) -> Product<Self,(),()> {
				Product {
					iters_original_tuple: (),
					current_val_tuple: Some(()),
					iters_tuple: self
				}
			}
		}

		impl<T,I> Iterator for Product<(I,),(),()>
		where
			I: Iterator<Item=T>
		{
			type Item = (T,);

			fn next(&mut self) -> Option<Self::Item> {
				self.iters_tuple.0.next()
				.map(|v| (v,) )
			}

			fn size_hint(&self) -> (usize, Option<usize>) {
				self.iters_tuple.0.size_hint()
			}
		}

	};

	// イテレータが多数の場合の実装
	(@many_impl
		$( $ia:ident $ta:ident $na:tt )+ |
		$( $ifm:ident $tfm:ident $nfm:tt )+ |
		$( $iml:ident $tml:ident $nml:tt )+ |
		$nf:tt $nl:tt
	) => {

		impl<$($ta),+,$($ia),+> IntoIter< ((),$($iml),+), ($($tfm),+,()) > for ($($ia),+)
		where
			$( $ia: Iterator<Item=$ta> ),+,
			$( $iml: Clone ),+,
			$( $tfm: Clone ),+
		{
			fn cartesian_product(mut self)
			-> Product<Self,((),$($iml),+),($($tfm),+,())>
			{
				Product {
					iters_original_tuple: ((),$(self.$nml.clone()),+),
					current_val_tuple: ($(self.$nfm.next()),+,Some(()))
					.zip_options(),
					iters_tuple: self
				}
			}
		}

		impl<$($ta),+,$($ia),+> Iterator for Product<($($ia),+),((),$($iml),+),($($tfm),+,())>
		where
			$( $ia: Iterator<Item=$ta> ),+,
			$( $iml: Clone ),+,
			$( $tfm: Clone ),+
		{
			type Item = ($($ta),+);

			fn next(&mut self) -> Option<Self::Item> {
				let Self {
					iters_tuple: ref mut it,
					iters_original_tuple: ref iot,
					current_val_tuple: ref mut cvo,
				} = self;
				let cv = cvo.as_mut()?;

				let last = impl_product_iters!{@next
					it iot cv ($($na)+)
				};

				Some( ( $( cv.$nfm.clone(), )+ last ) )
			}

			fn size_hint(&self) -> (usize, Option<usize>) {
				let Self {
					iters_tuple: ref it,
					iters_original_tuple: ref iot,
					..
				} = self;

				let mut ma = it.$nf.size_hint();
				$( ma = size_hint_mul_add(
					ma,
					iot.$nml.size_hint(),
					it.$nml.size_hint()
				); )+
				ma
			}

		}

	};

	// 関連トレイトの実装
	(@additional_impl $($i:ident)+ ) => {
		impl<$($i),+,O,V> ExactSizeIterator for Product<($($i,)+),O,V>
		where $( $i: ExactSizeIterator ),+, Self: Iterator
		{}

		impl<$($i),+,O,V> FusedIterator for Product<($($i,)+),O,V>
		where $( $i: FusedIterator ),+, Self: Iterator
		{}
	};

	// many_impl の next() のコンポーネント: 先頭の成分
	(@next
		$it:ident $iot:ident $cv:ident
		($nc:tt $($ny:tt)+)
	) => {
		impl_product_iters!{@next
			$it $iot $cv () $nc ($($ny)+)
			{ $it.$nc.next()? }
		}
	};
	// many_impl の next() のコンポーネント: 先頭以外の残りの成分
	(@next
		$it:ident $iot:ident $cv:ident
		($($nd:tt)*) $np:tt ($nc:tt $($ny:tt)*)
		{$($inner:tt)+}
	) => {
		impl_product_iters!{@next
			$it $iot $cv ($($nd)* $np) $nc ($($ny)*)
			{ match $it.$nc.next() {
				Some(v) => v,
				None => {
					$it.$nc = $iot.$nc.clone();
					$cv.$np = $($inner)+;
					$it.$nc.next()?
				}
			} }
		}
	};
	// many_impl の next() のコンポーネント: 最後の成分が終わった後に呼び出せれ、生成したソースコードをそのまま返す
	(@next
		$it:ident $iot:ident $cv:ident
		($($nd:tt)*) $np:tt () {$($src:tt)+}
	) => { $($src)+ };
}
pub(crate) use impl_product_iters;



mod for_double_ended_iters_tuple {

	/// 複数のイテレータのカーテジアン積をとったイテレータで、両方向からのイテレートが可能
	pub struct CartesianProduct<I,O,V,L> {
		pub(crate) forward_iters: I,
		pub(crate) backward_iters: I,
		pub(crate) length: usize,
		pub(crate) length_each: L,
		pub(crate) forward_index: usize,
		pub(crate) backward_index: usize,
		pub(crate) original_iters: O,
		pub(crate) current_val_forward: Option<V>,
		pub(crate) current_val_backward: Option<V>
	}
	type Product<I,O,V,L> = CartesianProduct<I,O,V,L>;

	/// 複数のイテレータのタプルをカーテジアン積をとった単一のイテレータに変換するトレイト
	pub trait IntoIter<I,O,V,L> {
		/// イテレータのタプル `(I1,I2,I3,...)` をカーテジアン積をとったイテレータ `Iterator<Item=(T1,T2,T3,...)` に変換します。両方向からのイテレートができます。各イテレータが `Clone` を実装していなければなりません。
		fn cartesian_product_double_ended(self) -> Product<I,O,V,L>;
	}

	// impl<I1,I2,I3,T1,T2,T3>
	// IntoIter<Self,((),I2,I3),(T1,T2,()),(usize,usize,usize)> for (I1,I2,I3)
	// where
	// 	I1: DoubleEndedIterator<Item=T1> + ExactSizeIterator + Clone,
	// 	I2: DoubleEndedIterator<Item=T2> + ExactSizeIterator + Clone,
	// 	I3: DoubleEndedIterator<Item=T3> + ExactSizeIterator + Clone,
	// 	T1: Clone, T2: Clone, T3: Clone
	// {
	// 	fn cartesian_product(self) -> Product<Self,((),I2,I3),(T1,T2,()),(usize,usize,usize)> {
	// 		let l = (self.0.len(),self.1.len(),self.2.len());
	// 		let lm =
	// 			Some(l.0)
	// 			.and_then(|m| m.checked_mul(l.1) )
	// 			.and_then(|m| m.checked_mul(l.2) )
	// 			.expect("イテレータの要素数の積が usize 型の上限値を超えるためイテレータが生成できませんでした。");

	// 		let original_iters = ((),self.1.clone(),self.2.clone());
	// 		let mut forward_iters = (self.0.clone(),self.1.clone(),self.2.clone());
	// 		let mut backward_iters = self;

	// 		let current_val_forward = (
	// 			forward_iters.0.next(),
	// 			forward_iters.1.next(),
	// 			Some(())
	// 		).zip_options();
	// 		let current_val_backward = (
	// 			backward_iters.0.next_back(),
	// 			backward_iters.1.next_back(),
	// 			Some(())
	// 		).zip_options();

	// 		let forward_index = 0_usize;
	// 		let backward_index = lm.checked_sub(1).unwrap_or(0);

	// 		Product {
	// 			length: lm,
	// 			length_each: l,
	// 			forward_index,
	// 			backward_index,
	// 			current_val_forward,
	// 			current_val_backward,
	// 			original_iters,
	// 			forward_iters,
	// 			backward_iters
	// 		}
	// 	}
	// }

	// impl<I1,I2,I3,T1,T2,T3>
	// Iterator for Product<(I1,I2,I3),((),I2,I3),(T1,T2,()),(usize,usize,usize)>
	// where
	// 	I1: DoubleEndedIterator<Item=T1> + ExactSizeIterator + Clone,
	// 	I2: DoubleEndedIterator<Item=T2> + ExactSizeIterator + Clone,
	// 	I3: DoubleEndedIterator<Item=T3> + ExactSizeIterator + Clone,
	// 	T1: Clone, T2: Clone, T3: Clone
	// {
	// 	type Item = (T1,T2,T3);

	// 	fn next(&mut self) -> Option<Self::Item> {
	// 		// 末尾に達した場合や、逆方向のイテレートより後に行こうとした場合は None
	// 		let Self {
	// 			forward_index: ref fi,
	// 			backward_index: ref bi,
	// 			length: ref l,
	// 			..
	// 		} = self;
	// 		if (*fi+1)>=(*l) { return None }
	// 		else if (*fi+1)>=(*bi) { return None }

	// 		let Self {
	// 			forward_iters: ref mut it,
	// 			current_val_forward: ref mut cvo,
	// 			forward_index: ref mut id,
	// 			original_iters: ref oi,
	// 			length_each: ref l,
	// 			..
	// 		} = self;
	// 		let cv = cvo.as_mut()?;
	// 		*id += 1;
	// 		let mut i = *id;

	// 		if i%l.2==0 {
	// 			i /= l.2;
	// 			if i%l.1==0 {
	// 				i /= l.1;
	// 				if i%l.0==0 {
	// 					return None;
	// 				}
	// 				cv.0 = it.0.next()?;
	// 				it.1 = oi.1.clone();
	// 			}
	// 			cv.1 = it.1.next()?;
	// 			it.2 = oi.2.clone();
	// 		}

	// 		Some((
	// 			cv.0.clone(),
	// 			cv.1.clone(),
	// 			it.2.next()?
	// 		))
	// 	}

	// 	fn size_hint(&self) -> (usize, Option<usize>) {
	// 		let l = length(self);
	// 		(l,Some(l))
	// 	}
	// }
	// impl<I1,I2,I3,T1,T2,T3>
	// DoubleEndedIterator for Product<(I1,I2,I3),((),I2,I3),(T1,T2,()),(usize,usize,usize)>
	// where
	// 	I1: DoubleEndedIterator<Item=T1> + ExactSizeIterator + Clone,
	// 	I2: DoubleEndedIterator<Item=T2> + ExactSizeIterator + Clone,
	// 	I3: DoubleEndedIterator<Item=T3> + ExactSizeIterator + Clone,
	// 	T1: Clone, T2: Clone, T3: Clone
	// {
	// 	fn next_back(&mut self) -> Option<Self::Item> {
	// 		// 先頭に達した場合や、順方向のイテレートより前に行こうとした場合は None
	// 		let Self {
	// 			forward_index: ref fi,
	// 			backward_index: ref bi,
	// 			..
	// 		} = self;
	// 		if (*bi)==0 { return None }
	// 		else if (*bi-1)<=(*fi) { return None }

	// 		let Self {
	// 			backward_iters: ref mut it,
	// 			current_val_backward: ref mut cvo,
	// 			backward_index: ref mut id,
	// 			original_iters: ref oi,
	// 			length_each: ref l,
	// 			..
	// 		} = self;
	// 		let cv = cvo.as_mut()?;
	// 		*id += 1;
	// 		let mut i = *id;

	// 		if i%l.2==0 {
	// 			i /= l.2;
	// 			if i%l.1==0 {
	// 				i /= l.1;
	// 				if i%l.0==0 {
	// 					return None;
	// 				}
	// 				cv.0 = it.0.next_back()?;
	// 				it.1 = oi.1.clone();
	// 			}
	// 			cv.1 = it.1.next_back()?;
	// 			it.2 = oi.2.clone();
	// 		}

	// 		Some((
	// 			cv.0.clone(),
	// 			cv.1.clone(),
	// 			it.2.next_back()?
	// 		))
	// 	}
	// }

	// fn length<I,O,V,L>(p:&Product<I,O,V,L>) -> usize {
	// 	p.backward_index
	// 	.checked_sub(p.forward_index)
	// 	.map_or(0,|d| d+1 )
	// }

}
pub use for_double_ended_iters_tuple::{
	CartesianProduct as ProductForDoubleEndedIteratorsTuple,
	IntoIter as IntoTupleProductDoubleEndedIterator
};

/// * イテレータの要素数ごとに `CartesianProduct` を実装するマクロ
/// * `impl_product_double_ended_iters!( I0 T0 0 I1 T1 1 I2 T2 2 ... I(N-1) T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
/// * `I*` `T*` の異なる型パラメータとタプルのインデクスをこの順で並べていく
macro_rules! impl_product_double_ended_iters {
	// マクロのエントリポイント: 全ての実装をモジュールで囲む
	( $( $i:ident $t:ident $n:tt )+ ) => {
		mod impl_product_double_ended_iters {
			use super::{
				ProductForDoubleEndedIteratorsTuple as Product,
				IntoTupleProductDoubleEndedIterator as IntoIter,
				*
			};

			impl_product_double_ended_iters! {@process $( $i $t $n )+ }

			fn length<I,O,V,L>(p:&Product<I,O,V,L>) -> usize {
				p.backward_index
				.checked_sub(p.forward_index)
				.map_or(0,|d| d+1 )
			}
		}
	};

	// 引数を分離するプロセス: エントリポイント
	(@process
		$i0:ident $t0:ident $n0:tt
		$( $i:ident $t:ident $n:tt )*
	) => {
		impl_product_double_ended_iters! {@process | $i0 $t0 $n0 | $( $i $t $n )* }
	};
	// 引数を分離するプロセス: `|` により3つに分かれた領域のうち、前の2つを残す場合と、後ろから1つずつ要素をずらした場合に分ける
	// 1つ目の領域: impl するアイテムの末尾以外
	// 2つ目の領域: impl するアイテムの末尾
	// 3つ目の領域: 残りのアイテム
	(@process
		$( $i:ident $t:ident $n:tt )* |
		$il:ident $tl:ident $nl:tt |
		$in:ident $tn:ident $nn:tt
		$( $others:tt )*
	) => {
		impl_product_double_ended_iters! {@process
			$( $i $t $n )* | $il $tl $nl |
		}
		impl_product_double_ended_iters! {@process
			$( $i $t $n )* $il $tl $nl |
			$in $tn $nn | $( $others )*
		}
	};
	// 引数を分離するプロセス: impl するアイテムの数が1つの場合 → 後述の one_impl に渡す
	(@process | $i:ident $t:ident $n:tt | ) => {
		impl_product_double_ended_iters! {@one_impl}
	};
	// 引数を分離するプロセス: impl するアイテムの数が複数ある場合 → 引数を加工した上で後述の many_impl に渡す
	(@process
		$if:ident $tf:ident $nf:tt
		$( $i:ident $t:ident $n:tt )* |
		$il:ident $tl:ident $nl:tt |
	) => {
		impl_product_double_ended_iters! {@many_impl
			$if $tf $nf usize
			$( $i $t $n usize )*
			$il $tl $nl usize |
			$if $tf $nf
			$( $i $t $n )* |
			$( $i $t $n )*
			$il $tl $nl |
			$nf $nl
		}
	};

	// イテレータの数が1つの場合の実装
	(@one_impl) => {

		impl<I,T> IntoIter<(),I,(),()> for (I,)
		where
			I: DoubleEndedIterator<Item=T> + ExactSizeIterator
		{
			fn cartesian_product_double_ended(self) -> Product<(),I,(),()> {
				Product {
					forward_iters: (),
					backward_iters: (),
					length: 0,
					length_each: (),
					forward_index: 0,
					backward_index: 0,
					original_iters: self.0,
					current_val_forward: None,
					current_val_backward: None
				}
			}
		}

		impl<I,T> Iterator for Product<(),I,(),()>
		where
			I: DoubleEndedIterator<Item=T> + ExactSizeIterator
		{
			type Item = (T,);
			fn next(&mut self) -> Option<Self::Item>
			{ self.original_iters.next().map(|v| (v,) ) }
			fn nth(&mut self,n:usize) -> Option<Self::Item> { self.original_iters.nth(n).map(|v| (v,) ) }
			fn size_hint(&self) -> (usize, Option<usize>)
			{ self.original_iters.size_hint() }
		}

		impl<I,T> DoubleEndedIterator for Product<(),I,(),()>
		where
			I: DoubleEndedIterator<Item=T> + ExactSizeIterator
		{
			fn next_back(&mut self) -> Option<Self::Item>
			{ self.original_iters.next_back().map(|v| (v,) ) }
			fn nth_back(&mut self, n: usize) -> Option<Self::Item>
			{ self.original_iters.nth_back(n).map(|v| (v,) ) }
		}

		impl<I,T> ExactSizeIterator for Product<(),I,(),()>
		where
			I: DoubleEndedIterator<Item=T> + ExactSizeIterator
		{
			fn len(&self) -> usize { self.original_iters.len() }
		}

	};

	// イテレータが多数の場合の実装
	(@many_impl
		$( $ia:ident $ta:ident $na:tt $ua:ident )+ |
		$( $ifm:ident $tfm:ident $nfm:tt )+ |
		$( $iml:ident $tml:ident $nml:tt )+ |
		$nf:tt $nl:tt
	) => {

		impl<$($ia),+,$($ta),+>
		IntoIter<Self,(() $(,$iml)+),($($tfm,)+ ()),($($ua,)+)> for ($($ia),+)
		where
			$( $ia: DoubleEndedIterator<Item=$ta> + ExactSizeIterator + Clone ),+ ,
			$( $ta: Clone, )+
		{
			fn cartesian_product_double_ended(self) -> Product<Self,(() $(,$iml)+),($($tfm,)+ ()),($($ua,)+)> {
				let l = ( $( self.$na.len(), )+ );
				let lm =
					Some(l.0)
					$( .and_then(|m| m.checked_mul(l.$nml) ) )+
					.expect("イテレータの要素数の積が usize 型の上限値を超えるためイテレータが生成できませんでした。");

				let original_iters = (() $(,self.$nml.clone())+ );
				let mut forward_iters = ( $(self.$na.clone(),)+ );
				let mut backward_iters = self;

				let current_val_forward = (
					$( forward_iters.$nfm.next(), )+
					Some(())
				).zip_options();
				let current_val_backward = (
					$( backward_iters.$nfm.next_back(), )+
					Some(())
				).zip_options();

				Product {
					length: lm,
					length_each: l,
					forward_index: 0_usize,
					backward_index: lm,
					current_val_forward,
					current_val_backward,
					original_iters,
					forward_iters,
					backward_iters
				}
			}
		}

		impl<$($ia),+,$($ta),+> Iterator for Product<($($ia),+),(() $(,$iml)+),($($tfm,)+ ()),($($ua,)+)>
		where
			$( $ia: DoubleEndedIterator<Item=$ta> + ExactSizeIterator + Clone ),+ ,
			$( $ta: Clone, )+
		{
			type Item = ($($ta,)+);

			fn next(&mut self) -> Option<Self::Item> {
				// 末尾に達した場合や、逆方向のイテレートより後に行こうとした場合は None
				let Self {
					forward_index: ref fi,
					backward_index: ref bi,
					length: ref l,
					..
				} = self;
				if (*fi)>=(*l) || (*fi)>=(*bi) { return None; }

				let Self {
					forward_iters: ref mut it,
					current_val_forward: ref mut cvo,
					forward_index: ref mut id,
					original_iters: ref oi,
					length_each: ref l,
					..
				} = self;
				let cv = cvo.as_mut()?;

				if (*id) > 0 {
					let mut i = *id;
					impl_product_double_ended_iters!{@next next it oi cv i l ($($na)+) }
				}

				*id += 1;
				Some((
					$( cv.$nfm.clone(), )+
					it.$nl.next()?
				))
			}

			fn size_hint(&self) -> (usize, Option<usize>) {
				let l = length(self);
				(l,Some(l))
			}
		}

		impl<$($ia),+,$($ta),+> DoubleEndedIterator for Product<($($ia),+),(() $(,$iml)+),($($tfm,)+ ()),($($ua,)+)>
		where
			$( $ia: DoubleEndedIterator<Item=$ta> + ExactSizeIterator + Clone ),+ ,
			$( $ta: Clone, )+
		{
			fn next_back(&mut self) -> Option<Self::Item> {
				// 先頭に達した場合や、順方向のイテレートより前に行こうとした場合は None
				let Self {
					forward_index: ref fi,
					backward_index: ref bi,
					..
				} = self;
				if (*bi)==0 || (*bi)<=(*fi) { return None; }

				let Self {
					backward_iters: ref mut it,
					current_val_backward: ref mut cvo,
					backward_index: ref mut id,
					original_iters: ref oi,
					length_each: ref l,
					length: ref lm,
					..
				} = self;
				let cv = cvo.as_mut()?;

				if (*id) < (*lm) {
					let mut i = *id;
					impl_product_double_ended_iters!{@next next_back it oi cv i l ($($na)+) }
				}

				*id -= 1;
				Some((
					$( cv.$nfm.clone(), )+
					it.$nl.next_back()?
				))
			}
		}

		impl<$($ia),+,$($ta),+> ExactSizeIterator for Product<($($ia),+),(() $(,$iml)+),($($tfm,)+ ()),($($ua,)+)>
		where
			$( $ia: DoubleEndedIterator<Item=$ta> + ExactSizeIterator + Clone ),+ ,
			$( $ta: Clone, )+
		{
			fn len(&self) -> usize {
				length(self)
			}
		}

	};

	// many_impl の next() のコンポーネント: 先頭の成分
	(@next
		$next:ident
		$it:ident $oi:ident $cv:ident
		$i:ident $l:ident
		($nc:tt $($ny:tt)+)
	) => {
		impl_product_double_ended_iters!{@next
			$next $it $oi $cv $i $l
			() $nc ($($ny)+)
			{ if $i % $l.$nc == 0 {
				return None;
			} }
		}
	};
	// many_impl の next() のコンポーネント: 先頭以外の残りの成分
	(@next
		$next:ident
		$it:ident $oi:ident $cv:ident
		$i:ident $l:ident
		($($nd:tt)*) $np:tt ($nc:tt $($ny:tt)*)
		{$($inner:tt)+}
	) => {
		impl_product_double_ended_iters!{@next
			$next $it $oi $cv $i $l
			($($nd)* $np) $nc ($($ny)*)
			{ if $i % $l.$nc == 0 {
				$i /= $l.$nc;
				$($inner)+
				$cv.$np = $it.$np.$next()?;
				$it.$nc = $oi.$nc.clone();
			} }
		}
	};
	// many_impl の next() のコンポーネント: 最後の成分が終わった後に呼び出せれ、生成したソースコードをそのまま返す
	(@next
		$next:ident
		$it:ident $oi:ident $cv:ident
		$i:ident $l:ident
		($($nd:tt)*) $np:tt () {$($src:tt)+}
	) => { $($src)+ };
}
pub(crate) use impl_product_double_ended_iters;
