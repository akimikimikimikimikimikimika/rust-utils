//! 複数のイテレータのカーテジアン積をとるトレイトやイテレータをまとめたモジュール

/// イテレータのタプルに関してカーテジアン積をとる関数を含むモジュール
mod for_iters_tuple {

	/// 複数のイテレータのカーテジアン積をとったイテレータ
	pub struct CartesianProduct<I,O,V> {
		pub(crate) iters_tuple: I,
		pub(crate) iters_original_tuple: O,
		pub(crate) current_val_tuple: Option<V>
	}
	use CartesianProduct as Product;

	/// 複数のイテレータのタプルをカーテジアン積をとった単一のイテレータに変換するトレイト
	pub trait IntoIter: Sized {
		type OriginalIters;
		type CurrentValues;
		/// イテレータのタプル `(I1,I2,I3,...)` をカーテジアン積をとったイテレータ `Iterator<Item=(T1,T2,T3,...)` に変換します。各イテレータが `Clone` を実装していなければなりません。
		fn cartesian_product(self) -> Product<Self,Self::OriginalIters,Self::CurrentValues>;
	}

	/// * イテレータの要素数ごとに `CartesianProduct` を実装するマクロ
	/// * `impl_product_iters!( I0 T0 0 I1 T1 1 I2 T2 2 ... I(N-1) T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	/// * `I*` `T*` の異なる型パラメータとタプルのインデクスをこの順で並べていく
	macro_rules! impl_product_iters {
		// マクロのエントリポイント: 全ての実装をモジュールで囲む
		( $( $i:ident $t:ident $n:tt )+ ) => {
			mod impl_product_iters {
				use super::*;
				use ProductForIteratorsTuple as Product;
				use IntoTupleProductIterator as IntoIter;

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

			impl<T,I> IntoIter for (I,)
			where I: Iterator<Item=T>
			{
				type OriginalIters = ();
				type CurrentValues = ();
				fn cartesian_product(self) -> Product<Self,Self::OriginalIters,Self::CurrentValues> {
					Product {
						iters_original_tuple: (),
						current_val_tuple: Some(()),
						iters_tuple: self
					}
				}
			}

			impl<T,I> Iterator for Product<(I,),(),()>
			where I: Iterator<Item=T>
			{
				type Item = (T,);

				fn next(&mut self) -> Option<Self::Item> {
					self.iters_tuple.0.next()
					.map(|v| (v,) )
				}

				fn nth(&mut self,n:usize) -> Option<Self::Item> {
					self.iters_tuple.0.nth(n)
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

			impl<$($ta),+,$($ia),+> IntoIter for ($($ia),+)
			where
				$( $ia: Iterator<Item=$ta> ),+,
				$( $iml: Clone ),+,
				$( $tfm: Clone ),+
			{
				type OriginalIters = ((),$($iml),+);
				type CurrentValues = ($($tfm),+,());
				fn cartesian_product(mut self)
				-> Product<Self,Self::OriginalIters,Self::CurrentValues>
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

}
pub use for_iters_tuple::{
	CartesianProduct as ProductForIteratorsTuple,
	IntoIter as IntoTupleProductIterator
};
pub(crate) use for_iters_tuple::impl_product_iters;



/// イテレータのタプルに関してカーテジアン積をとり、両側からアクセスできるようにした関数を含むモジュール
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
	use CartesianProduct as Product;

	/// 複数のイテレータのタプルをカーテジアン積をとった単一のイテレータに変換するトレイト
	pub trait IntoIter {
		type Iters;
		type OriginalIters;
		type CurrentValues;
		type Length;
		/// イテレータのタプル `(I1,I2,I3,...)` をカーテジアン積をとったイテレータ `Iterator<Item=(T1,T2,T3,...)` に変換します。両方向からのイテレートができます。各イテレータが `Clone` を実装していなければなりません。
		fn cartesian_product_double_ended(self) -> Product<Self::Iters,Self::OriginalIters,Self::CurrentValues,Self::Length>;
	}

	/// * イテレータの要素数ごとに `CartesianProduct` を実装するマクロ
	/// * `impl_product_double_ended_iters!( I0 T0 0 I1 T1 1 I2 T2 2 ... I(N-1) T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	/// * `I*` `T*` の異なる型パラメータとタプルのインデクスをこの順で並べていく
	macro_rules! impl_product_double_ended_iters {
		// マクロのエントリポイント: 全ての実装をモジュールで囲む
		( $( $i:ident $t:ident $n:tt )+ ) => {
			mod impl_product_double_ended_iters {
				use super::*;
				use ProductForDoubleEndedIteratorsTuple as Product;
				use IntoTupleProductDoubleEndedIterator as IntoIter;

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
			impl_product_double_ended_iters! {@process
				first( $i0 $t0 $n0 ) last( )
				mid_forward( ) mid_backward( )
				not_yet( $( $i $t $n )* )
			}
		};
		// 引数を分離するプロセス: 順方向と逆方向でアイテムを並び替える
		(@process
			first( $i_f:ident $t_f:ident $n_f:tt )
			last( $( $i_l:ident $t_l:ident $n_l:tt )? )
			mid_forward( $( $i_mf:ident $t_mf:ident $n_mf:tt )* )
			mid_backward( $( $i_mb:ident $t_mb:ident $n_mb:tt )* )
			not_yet(
				$i_n:ident $t_n:ident $n_n:tt
				$( $others:tt )*
			)
		) => {
			impl_product_double_ended_iters! {@process
				first( $i_f $t_f $n_f ) last( $( $i_l $t_l $n_l )? )
				mid_forward( $( $i_mf $t_mf $n_mf )* )
				mid_backward( $( $i_mb $t_mb $n_mb )* )
				not_yet( )
			}
			impl_product_double_ended_iters! {@process
				first( $i_f $t_f $n_f ) last( $i_n $t_n $n_n )
				mid_forward( $( $i_mf $t_mf $n_mf )* $( $i_l $t_l $n_l )? )
				mid_backward( $( $i_l $t_l $n_l )? $( $i_mb $t_mb $n_mb )* )
				not_yet( $($others)* )
			}
		};
		// 引数を分離するプロセス: impl するアイテムの数が1つの場合 → 後述の one_impl に渡す
		(@process
			first( $i:ident $t:ident $n:tt )
			last() mid_forward() mid_backward() not_yet()
		) => {
			impl_product_double_ended_iters! {@one_impl}
		};
		// 引数を分離するプロセス: impl するアイテムの数が複数ある場合 → 引数を加工した上で後述の many_impl に渡す
		(@process
			first( $i_f:ident $t_f:ident $n_f:tt )
			last( $i_l:ident $t_l:ident $n_l:tt )
			mid_forward( $( $i_mf:ident $t_mf:ident $n_mf:tt )* )
			mid_backward( $( $i_mb:ident $t_mb:ident $n_mb:tt )* )
			not_yet( )
		) => {
			impl_product_double_ended_iters! {@many_impl
				forward_all(
					$i_f $t_f $n_f usize
					$( $i_mf $t_mf $n_mf usize )*
					$i_l $t_l $n_l usize
				)
				forward_first_mid(
					$i_f $t_f $n_f
					$( $i_mf $t_mf $n_mf )*
				)
				forward_mid_last(
					$( $i_mf $t_mf $n_mf )*
					$i_l $t_l $n_l
				)
				forward_mid( $( $n_mf )* )
				backward_mid_last( $n_l $($n_mb)* )
				first_last( $n_f $n_l )
			}
		};

		// イテレータの数が1つの場合の実装
		(@one_impl) => {

			impl<I,T> IntoIter for (I,)
			where
				I: DoubleEndedIterator<Item=T> + ExactSizeIterator
			{
				type Iters = ();
				type OriginalIters = I;
				type CurrentValues = ();
				type Length = ();
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

			impl<I,T> Clone for Product<(),I,(),()>
			where
				I: DoubleEndedIterator<Item=T> + ExactSizeIterator + Clone
			{
				fn clone(&self) -> Self {
					Product {
						forward_iters: (),
						backward_iters: (),
						length: 0,
						length_each: (),
						forward_index: 0,
						backward_index: 0,
						original_iters: self.original_iters.clone(),
						current_val_forward: None,
						current_val_backward: None
					}
				}
			}

		};

		// イテレータが多数の場合の実装
		(@many_impl
			forward_all( $( $i_fa:ident $t_fa:ident $n_fa:tt $ua:ident )+ )
			forward_first_mid( $( $i_ffm:ident $t_ffm:ident $n_ffm:tt )+ )
			forward_mid_last( $( $i_fml:ident $t_fml:ident $n_fml:tt )+ )
			forward_mid( $( $n_fm:tt )* )
			backward_mid_last( $($n_bml:tt)+ )
			first_last( $nf:tt $nl:tt )
		) => {

			impl<$($i_fa),+,$($t_fa),+>
			IntoIter for ($($i_fa),+)
			where
				$( $i_fa: DoubleEndedIterator<Item=$t_fa> + ExactSizeIterator + Clone ),+ ,
				$( $t_fa: Clone ),+
			{
				type Iters = Self;
				type OriginalIters = (() $(,$i_fml)+);
				type CurrentValues = ($($t_ffm,)+ ());
				type Length = ($($ua,)+);
				fn cartesian_product_double_ended(self) -> Product<Self::Iters,Self::OriginalIters,Self::CurrentValues,Self::Length> {
					let l = ( $( self.$n_fa.len(), )+ );
					let lm =
						Some(l.0)
						$( .and_then(|m| m.checked_mul(l.$n_fml) ) )+
						.expect("イテレータの要素数の積が usize 型の上限値を超えるためイテレータが生成できませんでした。");

					let original_iters = (() $(,self.$n_fml.clone())+ );
					let forward_iters = ( $(self.$n_fa.clone(),)+ );
					let backward_iters = self;

					Product {
						length: lm,
						length_each: l,
						forward_index: 0_usize,
						backward_index: lm,
						current_val_forward: None,
						current_val_backward: None,
						original_iters,
						forward_iters,
						backward_iters
					}
				}
			}

			impl<$($i_fa),+,$($t_fa),+> Iterator for Product<($($i_fa),+),(() $(,$i_fml)+),($($t_ffm,)+ ()),($($ua,)+)>
			where
				$( $i_fa: DoubleEndedIterator<Item=$t_fa> + ExactSizeIterator + Clone ),+ ,
				$( $t_fa: Clone ),+
			{
				type Item = ($($t_fa,)+);

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

					// 先頭に位置する場合は別に処理する
					if (*id)==0 {
						let v = ( $( it.$n_fa.next()?, )+ );
						let cv = (
							$( v.$n_ffm.clone(), )+
							()
						);
						*cvo = Some(cv);
						*id += 1;
						return Some(v);
					}

					let cv = cvo.as_mut()?;
					let mut i = *id;
					impl_product_double_ended_iters!{@next next it oi cv i l ($($n_fa)+) }
					*id += 1;
					Some((
						$( cv.$n_ffm.clone(), )+
						it.$nl.next()?
					))
				}

				fn nth(&mut self,by:usize) -> Option<Self::Item> {
					// 末尾に達した場合や、逆方向のイテレートより後に行こうとした場合は None
					let Self {
						forward_index: ref mut fi,
						backward_index: ref bi,
						length: ref l,
						..
					} = self;
					if (*l-*fi) <= by {
						*fi = *l;
						return None;
					}
					if (*bi-*fi) <= by {
						*fi = *bi;
						return None;
					}
					// 先頭に位置する場合は別に処理する
					if (*fi)==0 {
						let i_dst = self.index_each(by);
						let Self {
							forward_iters: ref mut it,
							forward_index: ref mut id,
							current_val_forward: ref mut cvo,
							..
						} = self;
						let v = ( $( it.$n_fa.nth(i_dst.$n_fa)?, )+ );
						let cv = ( $( v.$n_ffm.clone(), )+ () );
						*cvo = Some(cv);
						*id += by + 1;
						return Some(v);
					}

					let i = self.forward_index;
					let i_current = self.index_each(i-1);
					let i_next = self.index_each(i+by);

					let Self {
						forward_iters: ref mut it,
						forward_index: ref mut index,
						original_iters: ref oi,
						current_val_forward: ref mut cvo,
						..
					} = self;
					let cv = cvo.as_mut()?;

					match (i_next.$nf,i_current.$nf) {
						(n,c) if n>c => { cv.$nf = it.$nf.nth(n-c-1)?; },
						_ => {}
					}
					$( match (i_next.$n_fm,i_current.$n_fm) {
						(n,c) if n>c => { cv.$n_fm = it.$n_fm.nth(n-c-1)?; },
						(n,c) if n<c => {
							it.$n_fm = oi.$n_fm.clone();
							cv.$n_fm = it.$n_fm.nth(n)?;
						},
						_ => {}
					} )*
					let v = match (i_next.$nl,i_current.$nl) {
						(n,c) if n>c => it.$nl.nth(n-c-1)?,
						(n,_) => {
							it.$nl = oi.$nl.clone();
							it.$nl.nth(n)?
						}
					};

					*index += by + 1;

					Some( (
						$( cv.$n_ffm.clone(), )+
						v
					) )
				}

				fn size_hint(&self) -> (usize, Option<usize>) {
					let l = length(self);
					(l,Some(l))
				}
			}

			impl<$($i_fa),+,$($t_fa),+> DoubleEndedIterator for Product<($($i_fa),+),(() $(,$i_fml)+),($($t_ffm,)+ ()),($($ua,)+)>
			where
				$( $i_fa: DoubleEndedIterator<Item=$t_fa> + ExactSizeIterator + Clone ),+ ,
				$( $t_fa: Clone ),+
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

					// 末尾に位置する場合は別に処理する
					if (*id)==(*lm) {
						let v = ( $( it.$n_fa.next_back()?, )+ );
						let cv = (
							$( v.$n_ffm.clone(), )+
							()
						);
						*cvo = Some(cv);
						*id -= 1;
						return Some(v);
					}

					let cv = cvo.as_mut()?;
					let mut i = *id;
					impl_product_double_ended_iters!{@next next_back it oi cv i l ($($n_fa)+) }
					*id -= 1;
					Some((
						$( cv.$n_ffm.clone(), )+
						it.$nl.next_back()?
					))
				}

				fn nth_back(&mut self,by:usize) -> Option<Self::Item> {
					// 先頭に達した場合や、順方向のイテレートより前に行こうとした場合は None
					let Self {
						forward_index: ref fi,
						backward_index: ref mut bi,
						length: ref l,
						..
					} = self;
					if (*bi) <= by {
						*bi = 0;
						return None;
					}
					if (*bi-*fi) <= by {
						*bi = *fi;
						return None;
					}
					// 末尾に位置する場合は別に処理する
					if (*bi)==(*l) {
						let i_dst = self.index_each_back(*l-by);
						let Self {
							backward_iters: ref mut it,
							backward_index: ref mut id,
							current_val_backward: ref mut cvo,
							..
						} = self;
						let v = ( $( it.$n_fa.nth_back(i_dst.$n_fa)?, )+ );
						let cv = ( $( v.$n_ffm.clone(), )+ () );
						*cvo = Some(cv);
						*id -= by + 1;
						return Some(v);
					}

					let i = self.backward_index;
					let i_current = self.index_each_back(i+1);
					let i_next = self.index_each_back(i-by);

					let Self {
						backward_iters: ref mut it,
						backward_index: ref mut index,
						original_iters: ref oi,
						current_val_backward: ref mut cvo,
						..
					} = self;
					let cv = cvo.as_mut()?;

					match (i_next.$nf,i_current.$nf) {
						(n,c) if n>c => { cv.$nf = it.$nf.nth_back(n-c-1)?; },
						_ => {}
					}
					$( match (i_next.$n_fm,i_current.$n_fm) {
						(n,c) if n>c => { cv.$n_fm = it.$n_fm.nth_back(n-c-1)?; },
						(n,c) if n<c => {
							it.$n_fm = oi.$n_fm.clone();
							cv.$n_fm = it.$n_fm.nth_back(n)?;
						},
						_ => {}
					} )*
					let v = match (i_next.$nl,i_current.$nl) {
						(n,c) if n>c => it.$nl.nth_back(n-c-1)?,
						(n,_) => {
							it.$nl = oi.$nl.clone();
							it.$nl.nth_back(n)?
						}
					};

					*index -= by + 1;

					Some( (
						$( cv.$n_ffm.clone(), )+
						v
					) )
				}
			}

			impl<$($i_fa),+,$($t_fa),+> ExactSizeIterator for Product<($($i_fa),+),(() $(,$i_fml)+),($($t_ffm,)+ ()),($($ua,)+)>
			where
				$( $i_fa: DoubleEndedIterator<Item=$t_fa> + ExactSizeIterator + Clone ),+ ,
				$( $t_fa: Clone ),+
			{
				fn len(&self) -> usize {
					length(self)
				}
			}

			impl<$($i_fa),+,$($t_fa),+> Clone for Product<($($i_fa),+),(() $(,$i_fml)+),($($t_ffm,)+ ()),($($ua,)+)>
			where
				$( $i_fa: DoubleEndedIterator<Item=$t_fa> + ExactSizeIterator + Clone ),+ ,
				$( $t_fa: Clone ),+
			{
				fn clone(&self) -> Self {
					Product {
						forward_iters: ( $(self.forward_iters.$n_fa.clone(), )+ ),
						backward_iters: ( $(self.backward_iters.$n_fa.clone(), )+ ),
						length: self.length,
						length_each: self.length_each.clone(),
						forward_index: self.forward_index,
						backward_index: self.backward_index,
						original_iters: ( (), $(self.original_iters.$n_fml.clone(), )+ ),
						current_val_forward: self.current_val_forward.clone(),
						current_val_backward: self.current_val_backward.clone()
					}
				}
			}

			impl<$($i_fa),+,$($t_ffm),+> Product<($($i_fa),+),(() $(,$i_fml)+),($($t_ffm,)+ ()),($($ua,)+)>
			{
				fn index_each(&self,mut i:usize) -> ($($ua,)+) {
					let Self {
						length_each: ref le,
						..
					} = self;

					let mut ie = le.clone();
					$(
						ie.$n_bml = i % le.$n_bml;
						i = i / le.$n_bml;
					)+
					ie.$nf = i;
					ie
				}
				fn index_each_back(&self,mut i:usize) -> ($($ua,)+) {
					let Self {
						length: ref l,
						length_each: ref le,
						..
					} = self;

					let mut ie = le.clone();
					i = l - i;
					$(
						ie.$n_bml = i % le.$n_bml;
						i = i / le.$n_bml;
					)+
					ie.$nf = i;
					ie
				}
			}

		};

		// many_impl の next() と next_back() のコンポーネント: 先頭の成分
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
		// many_impl の next() と next_back() のコンポーネント: 先頭以外の残りの成分
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
		// many_impl の next() と next_back() のコンポーネント: 最後の成分が終わった後に呼び出せれ、生成したソースコードをそのまま返す
		(@next
			$next:ident
			$it:ident $oi:ident $cv:ident
			$i:ident $l:ident
			($($nd:tt)*) $np:tt () {$($src:tt)+}
		) => { $($src)+ };
	}
	pub(crate) use impl_product_double_ended_iters;

}
pub use for_double_ended_iters_tuple::{
	CartesianProduct as ProductForDoubleEndedIteratorsTuple,
	IntoIter as IntoTupleProductDoubleEndedIterator
};
pub(crate) use for_double_ended_iters_tuple::impl_product_double_ended_iters;
