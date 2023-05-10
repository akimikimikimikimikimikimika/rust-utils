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

				impl_product_iters! {@next
					it iot cv
					( $($na)+ )
					( $($nfm)+ ) $nl
				}

				None
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

	// many_impl の next() のコンポーネント: 末尾の2成分
	(@next
		$it:ident $iot:ident $cv:ident
		( $n0:tt $n1:tt )
		($( $nfm:tt )+) $nl:tt
	) => {

		if let Some(v) = $it.$n1.next() {
			return Some( (
				$( $cv.$nfm.clone(), )+ v
			) )
		}

		$it.$n1 = $iot.$n1.clone();
		if let Some(v) = $it.$n0.next() {
			$cv.$n0 = v;
			return Some( (
				$ ($cv.$nfm.clone(), )+
				$it.$nl.next()?
			) )
		}

	};
	// many_impl の next() のコンポーネント: 先頭から末尾の手前まで (末尾に辿り着くまで繰り返し呼び出される)
	(@next
		$it:ident $iot:ident $cv:ident
		( $n0:tt $n1:tt $($nr:tt)+ )
		($( $nfm:tt )+) $nl:tt
	) => {
		impl_product_iters! {@next
			$it $iot $cv
			( $n1 $($nr)+ )
			($($nfm)+) $nl
		}

		$it.$n1 = $iot.$n1.clone();
		if let Some(v) = $it.$n0.next() {
			$cv.$n0 = v;
			$cv.$n1 = $it.$n1.next()?;
			return Some( (
				$ ($cv.$nfm.clone(), )+
				$it.$nl.next()?
			) )
		}
	};
}
pub(crate) use impl_product_iters;
