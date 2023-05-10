mod options {

	pub trait ZipOptions<T> {
		/// 複数の Option 型を含む型を1つの Option 型に変換します。要素のうち1つでも None があれば None になります
		fn zip_options(self) -> Option<T>;
	}

	/// * `(Option<T1>,Option<T2>,...)` を `Option<(T1,T2,...)>` に変換するトレイト `ZipOptions` の実装をまとめて行うマクロ
	/// * `impl_zip_options!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! impl_zip_options {
		( $( $t:ident $n:tt )+ ) => {
			mod impl_zip_options {
				use super::*;

				impl_zip_options!{@each | $( $t $n )+ }
			}
		};
		(@each $( $t:ident $n:tt )* | $tn:ident $nn:tt $( $others:tt )* ) => {
			impl_zip_options! {@each $( $t $n )* | }
			impl_zip_options! {@each $( $t $n )* $tn $nn | $( $others )* }
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
	pub(crate) use impl_zip_options;

	impl<T,const N:usize> ZipOptions<[T;N]> for [Option<T>;N] {
		fn zip_options(self) -> Option<[T;N]> {
			// 予め全要素の `None` チェックを行う
			for ov in self.iter() { ov.as_ref()?; }
			// 全て `Some(..)` なので、安全にアンラップできる
			Some( self.map(|ov| ov.unwrap() ) )
		}
	}

}
pub use options::*;



/// 同一要素からなるタプル型を配列に変換するモジュール
mod tuple_to_array {

	/// タプルを配列に変換します
	pub trait TupleToArray<T,const N:usize> {
		/// タプルを配列に変換します
		fn to_array(self) -> [T;N];
	}

	/// * タプルを配列に変換するトレイト `TupleToArray` の実装をまとめて行うマクロ
	/// * `impl_tuple_to_array!(indices: 0 1 2 ... N )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! impl_tuple_to_array {
		(indices: $i0:tt $($i:tt)+ ) => {
			mod impl_tuple_to_array {
				use super::*;

				impl_tuple_to_array! {@each T T $i0 | $($i),+ }
			}
		};
		(@each $t:ident $($tx:ident $x:tt),+ | $y0:tt $(,$y:tt)* ) => {
			impl<$t> TupleToArray<$t,$y0> for ($($tx,)+) {
				fn to_array(self) -> [$t;$y0] {
					[ $(self.$x),+ ]
				}
			}

			impl_tuple_to_array! {@each $t $($tx $x,)+ $t $y0 | $($y),* }
		};
		(@each $t:ident $($tx:ident $x:tt),+ | ) => {};
	}
	pub(crate) use impl_tuple_to_array;

}
pub use tuple_to_array::*;



mod array {
	use super::*;

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

	/// 配列のタプルからタプルの配列を生成するトレイト
	pub trait ZipArrays<T> {
		/// 同じ長さの固定長配列のタプル `([T1;N],[T2;N],...)` からタプルの配列 `[(T1,T2,...);N]` を生成します
		fn zip(self) -> T;
	}

	/// * 配列のタプルからタプルの配列を生成するトレイト `ZipArrays` の実装をまとめて行うマクロ
	/// * `impl_zip_arrays!( T0 0 T1 1 T2 2 ... T(N-1) (N-1) )` と指定すれば、 `N` 個の要素まで対応する
	macro_rules! impl_zip_arrays {
		( $( $t:ident $n:tt )+ ) => {
			mod impl_zip_arrays {
				use super::*;

				impl_zip_arrays! {@each | $( $t $n )+ }
			}
		};
		(@each $( $t:ident $n:tt )* | $tn:ident $nn:tt $( $others:tt )* ) => {
			impl_zip_arrays! {@each $( $t $n )* | }
			impl_zip_arrays! {@each $( $t $n )* $tn $nn | $( $others )* }
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
	pub(crate) use impl_zip_arrays;

}
pub use array::*;
