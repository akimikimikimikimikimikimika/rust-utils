pub mod options {

	pub trait ZipOptions<T> {
		/// 複数の Option 型を含む型を1つの Option 型に変換します。要素のうち1つでも None があれば None になります
		fn zip_options(self) -> Option<T>;
	}

	/// * `(Option<T1>,Option<T2>,...)` を `Option<(T1,T2,...)>` に変換するトレイト `ZipOptions` の実装をまとめて行うマクロ
	/// * `implement!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! implement {
		( $( $t:ident $n:tt )+ ) => {
			mod impl_zip_options {
				use super::*;
				use crate::tuples::options::*;

				implement!{@each | $( $t $n )+ }
			}
		};
		(@each $( $t:ident $n:tt )* | $tn:ident $nn:tt $( $others:tt )* ) => {
			implement! {@each $( $t $n )* | }
			implement! {@each $( $t $n )* $tn $nn | $( $others )* }
		};
		(@each $( $t:ident $n:tt )+ | ) => {
			impl<$($t),+> ZipOptions<($($t,)+)> for ($(Option<$t>,)+) {
				fn zip_options(self) -> Option<($($t,)+)> {
					Some( ( $(self.$n?,)+ ) )
				}
			}
		};
		(@each | ) => {};
	}
	pub(crate) use implement;

	impl<T,const N:usize> ZipOptions<[T;N]> for [Option<T>;N] {
		fn zip_options(self) -> Option<[T;N]> {
			// 予め全要素の `None` チェックを行う
			for ov in self.iter() { ov.as_ref()?; }
			// 全て `Some(..)` なので、安全にアンラップできる
			Some( self.map(|ov| ov.unwrap() ) )
		}
	}

}



/// 同一要素からなるタプル型を配列に変換するモジュール
pub mod tuple_to_array {

	/// タプルを配列に変換します
	pub trait TupleToArray<T,const N:usize> {
		/// タプルを配列に変換します
		fn to_array(self) -> [T;N];
	}

	/// * タプルを配列に変換するトレイト `TupleToArray` の実装をまとめて行うマクロ
	/// * `implement!(indices: 0 1 2 ... N )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! implement {
		(indices: $i0:tt $($i:tt)+ ) => {
			mod impl_tuple_to_array {
				use super::*;
				use crate::tuples::tuple_to_array::*;

				implement! {@each T T $i0 | $($i),+ }
			}
		};
		(@each $t:ident $($tx:ident $x:tt),+ | $y0:tt $(,$y:tt)* ) => {
			impl<$t> TupleToArray<$t,$y0> for ($($tx,)+) {
				fn to_array(self) -> [$t;$y0] {
					[ $(self.$x),+ ]
				}
			}

			implement! {@each $t $($tx $x,)+ $t $y0 | $($y),* }
		};
		(@each $t:ident $($tx:ident $x:tt),+ | ) => {};
	}
	pub(crate) use implement;

}



pub mod array {

	/// インデクス付き配列を生成するトレイト
	pub trait WithIndex<T,const N:usize> {
		/// 固定長配列にインデクスを付けたものを返します
		fn with_index(self) -> [(usize,T);N];
	}

	impl<T,const N:usize> WithIndex<T,N> for [T;N] {
		fn with_index(self) -> [(usize,T);N] {
			let mut index = 0_usize;
			self.map(|v| {
				let t = (index,v);
				index += 1;
				t
			})
		}
	}

	#[cfg(feature="iterator")]
	/// 配列のタプルからタプルの配列を生成するトレイト
	pub trait ZipArrays<T> {
		/// 同じ長さの固定長配列のタプル `([T1;N],[T2;N],...)` からタプルの配列 `[(T1,T2,...);N]` を生成します
		fn zip(self) -> T;
	}

	#[cfg(feature="iterator")]
	/// * 配列のタプルからタプルの配列を生成するトレイト `ZipArrays` の実装をまとめて行うマクロ
	/// * `implement!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! implement {
		( $( $t:ident $n:tt )+ ) => {
			mod impl_zip_arrays {
				use super::*;
				use crate::tuples::array::*;

				implement! {@each | $( $t $n )+ }
			}
		};
		(@each $( $t:ident $n:tt )* | $tn:ident $nn:tt $( $others:tt )* ) => {
			implement! {@each $( $t $n )* | }
			implement! {@each $( $t $n )* $tn $nn | $( $others )* }
		};
		(@each $( $t:ident $n:tt )+ | ) => {
			impl<$($t),+,const N:usize> ZipArrays<[($($t,)+);N]> for ($([$t;N],)+) where $($t: std::fmt::Debug),+ {
				fn zip(self) -> [($($t,)+);N] {
					( $( self.$n.into_iter() ,)+ )
					.into_zipped_iter()
					.collect::<Vec<_>>()
					.try_into()
					.unwrap() // アンラップのために Debug トレイトが必要
				}
			}
		};
		(@each | ) => {};
	}
	#[cfg(feature="iterator")]
	pub(crate) use implement;

}



/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::{
		options::ZipOptions,
		tuple_to_array::TupleToArray,
		array::{ WithIndex, ZipArrays }
	};
}
