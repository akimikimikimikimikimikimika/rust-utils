use super::*;



extern crate ndarray;
use ndarray as nd;

use nd::Axis as NDAxis;
pub use nd::{
	Data as NDData,
	DataMut as NDDataMut
};



/// 場の量の一般的な型を提供するモジュール
mod array_types {
	use super::*;
	use ndarray as nd;

	pub type Dim1 = nd::Dim<[usize;1]>;
	pub type Dim2 = nd::Dim<[usize;2]>;
	pub type Dim3 = nd::Dim<[usize;3]>;

	/// データを配列自身が保持している
	pub type OR = nd::OwnedRepr<f64>;
	/// 配列のビュー (データへの参照を持った配列)
	pub type VR<'a> = nd::ViewRepr<&'a f64>;
	/// 配列のミュータブルなビュー (データへの書き換え可能な参照を持った配列)
	pub type VMR<'a> = nd::ViewRepr<&'a mut f64>;

	compose_struct! {
		/// 配列のデータ保持の仕方のジェネリクス
		pub trait AR = nd::Data<Elem=f64>;
		/// 配列のデータ保持の仕方のジェネリクス (書き換え可能なものに限定)
		pub trait ARM = nd::DataMut<Elem=f64>;
	}

	/// NDArray の基本型
	pub type A<S,D> = nd::ArrayBase<S,D>;
	/// 1次元の NDArray のジェネリックな型
	pub type A1T<S> = A<S,Dim1>;
	/// 2次元の NDArray のジェネリックな型
	pub type A2T<S> = A<S,Dim2>;
	/// 3次元の NDArray のジェネリックな型
	pub type A3T<S> = A<S,Dim3>;

	/// 1次元の NDArray
	pub type A1 = A<OR,Dim1>;
	/// 2次元の NDArray
	pub type A2 = A<OR,Dim2>;
	/// 3次元の NDArray
	pub type A3 = A<OR,Dim3>;

	/// 1次元の NDArray のビュー
	pub type AV1<'a> = A<VR<'a>,Dim1>;
	/// 2次元の NDArray のビュー
	pub type AV2<'a> = A<VR<'a>,Dim2>;
	/// 3次元の NDArray のビュー
	pub type AV3<'a> = A<VR<'a>,Dim3>;

}
pub use array_types::*;



/// `A3` を各成分ごとの `A2` に分離する関数をまとめたモジュール
mod split_a3 {
	use super::*;

	pub trait SplitA3 {
		/// `A3` から `index` で指定したある1つの場を取り出し、ビューにする
		fn a2(&self,index:usize) -> A2T<VR>;
		/// `A3` から `indices` で指定した複数の場を取り出し、ビューにする
		fn split_to_a2<const N:usize>(&self,indices:[usize;N]) -> [A2T<VR>;N];
	}
	impl<S:AR> SplitA3 for A3T<S> {
		fn a2(&self,index:usize) -> A2T<VR> {
			self.index_axis(NDAxis(0),index)
		}
		fn split_to_a2<const N:usize>(&self,indices:[usize;N]) -> [A2T<VR>;N] {
			indices.map(|i| self.index_axis(NDAxis(0),i) )
		}
	}

	pub trait SplitA3Mut {
		/// `A3` から `index` で指定したある1つの場を取り出し、書き換え可能なビューにする
		fn a2_mut(&mut self,index:usize) -> A2T<VMR>;
		/// `A3` から `indices` で指定した複数の場を取り出し、書き換え可能なビューにする
		fn split_to_a2_mut<const N:usize>(&mut self,indices:[usize;N]) -> [A2T<VMR>;N];
		/// * `A3` のビューと書き換え可能なビューを同時生成する
		/// * 2つのビューの同じ要素に同時にアクセスしないこと
		unsafe fn view_and_view_mut(&mut self) -> (A3T<VR>,A3T<VMR>);
	}
	impl<S:ARM> SplitA3Mut for A3T<S> {
		fn a2_mut(&mut self,index:usize) -> A2T<VMR> {
			self.index_axis_mut(NDAxis(0),index)
		}
		fn split_to_a2_mut<const N:usize>(&mut self,indices:[usize;N]) -> [A2T<VMR>;N] {
			let r = self.raw_view_mut();
			indices.map(|i| {
				unsafe {
					r.deref_into_view_mut()
					.index_axis_move(NDAxis(0),i)
				}
			})
		}
		unsafe fn view_and_view_mut(&mut self) -> (A3T<VR>,A3T<VMR>) {
			let r = self.raw_view_mut();
			( r.deref_into_view(), r.deref_into_view_mut() )
		}
	}

}
pub use split_a3::*;



/// CSV 形式で2次元配列の中身を出力する関数群を与えるモジュール
mod debug_output_csv {

	use std::{
		fs::OpenOptions,
		io::prelude::*
	};
	use super::*;

	pub trait DebugNDArray {
		fn debug_output(&self,path:impl AnyPath,format:DebugFormat);
	}
	impl<S:AR> DebugNDArray for A2T<S> {
		fn debug_output(&self,path:impl AnyPath,format:DebugFormat) {

			let src =
			self.rows()
			.into_iter()
			.map(|c| {
				c.iter()
				.map(|v| {
					match format {
						DF::Exp1 => format!("{:.1e}",v),
						DF::Exp3 => format!("{:.3e}",v),
						DF::Exp6 => format!("{:.6e}",v),
						DF::Real => format!("{}",v)
					}
				})
				.collect::<Vec<String>>()
				.join(",")
			})
			.collect::<Vec<String>>()
			.join("\n");

			let mut o = OpenOptions::new()
				.write(true).truncate(true).create(true)
				.open(path)
				.unwrap_or_error_in_detail_as("デバッグ出力を開始できませんでした");

			o.write_all(src.as_bytes())
				.unwrap_or_error_in_detail_as("デバッグ出力できませんでした");

		}
	}

	pub enum DebugFormat {
		Exp1, Exp3, Exp6, Real
	}
	pub type DF = DebugFormat;

}
pub use debug_output_csv::*;
