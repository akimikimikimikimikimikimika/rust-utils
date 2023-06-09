/// イテレータのタプルに対してチェーンを定義するモジュール
pub mod for_iters_tuple {

	/// 複数のイテレータのタプルをチェーンしたイテレータに変換するトレイト
	pub trait IntoChain: Sized {
		/// イテレータのタプル `(I1,I2,I3,...)` を `I1`→`I2`→`I3` という順に連結した1つのイテレータに変換します
		fn into_chained_iter(self) -> Chain<Self>;
		/// イテレータのタプル `(I1,I2,I3,...)` を `I1`→`I2`→`I3` という順に連結した1つのイテレータに変換します
		fn chain(self) -> Chain<Self> { self.into_chained_iter() }
	}

	/// 複数のイテレータをチェーンする (連続に繋げる) イテレータです
	pub struct Chain<T> {
		pub(crate) iters_tuple: T,
		pub(crate) current: usize,
		pub(crate) current_back: usize
	}

	/// * 複数のイテレータに対する `Chain` トレイトを実装するマクロ
	/// * `implement!( I0 0 I1 1 I2 2 ... I(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! implement {
		( $( $i:ident $n:tt )+ ) => {
			mod impl_chain_iters {
				use super::*;
				use crate::iterator::chain::for_iters_tuple::*;

				implement! {@each T | $( $i $n )+ }
			}
		};
		(@each $t:ident $( $i:ident $n:tt )* | $in:ident $nn:tt $( $others:tt )* ) => {
			implement! {@each $t $( $i $n )* | }
			implement! {@each $t $( $i $n )* $in $nn | $($others)* }
		};
		(@each $t:ident $( $i:ident $n:tt )+ | ) => {

			impl<$t,$($i),+> IntoChain for ($($i,)+)
			where $( $i: Iterator<Item=$t> ),+
			{
				fn into_chained_iter(self) -> Chain<Self> {
					Chain { iters_tuple: self, current: 0, current_back: 0 }
				}
			}

			impl<$t,$($i),+> Iterator for Chain<($($i,)+)>
			where $( $i: Iterator<Item=$t> ),+
			{
				type Item = $t;

				fn next(&mut self) -> Option<Self::Item> {
					$( if self.current==$n {
						if let s @ Some(_) = self.iters_tuple.$n.next() { return s; }
						self.current += 1;
					} )+
					None
				}

				fn size_hint(&self) -> (usize, Option<usize>) {
					let size_hint = ( $( self.iters_tuple.$n.size_hint(),)+ );
					let l = 0 $(+ size_hint.$n.0 )+;
					let u = ( $(size_hint.$n.1,)+ )
					.zip_options()
					.map(|t| 0 $(+ t.$n)+ );
					(l,u)
				}
			}

			implement! {@backward $t 0 | $( $i $n )+ }

			impl<$t,$($i),+> ExactSizeIterator for Chain<($($i,)+)>
			where $( $i: ExactSizeIterator<Item=$t> ),+ {}

			impl<$t,$($i),+> FusedIterator for Chain<($($i,)+)>
			where $( $i: FusedIterator<Item=$t> ),+ {}

		};
		(@each $t:ident | ) => {};
		(@backward
			$t:ident $n_largest:tt
			$( $i:ident $n:tt )* |
			$in:ident $nn:tt $( $others:tt )*
		) => {
			implement! {@backward $t $nn $in $nn $( $i $n )* | $($others)* }
		};
		(@backward
			$t:ident $n_largest:tt
			$( $i:ident $n:tt )+ |
		) => {
			impl<$t,$($i),+> DoubleEndedIterator for Chain<($($i,)+)>
			where $( $i: DoubleEndedIterator<Item=$t> + ExactSizeIterator ),+
			{
				fn next_back(&mut self) -> Option<Self::Item> {
					$( if self.current_back==($n_largest-$n) {
						if let s @ Some(_) = self.iters_tuple.$n.next_back() { return s; }
						self.current_back += 1;
					} )+
					None
				}
			}
		};
	}
	pub(crate) use implement;

}



/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::for_iters_tuple::IntoChain as IntoChainedIteratorForIteratorsTuple;
}
