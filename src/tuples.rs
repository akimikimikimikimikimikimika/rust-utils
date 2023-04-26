mod options {

	pub trait Zip<T> {
		/// 複数の Option 型を含む型を1つの Option 型に変換します。要素のうち1つでも None があれば None になります
		fn zip_options(self) -> Option<T>;
	}

	macro_rules! zip_many {
		( $i0:tt:$t0:ident $($in:tt:$tn:ident)+ ) => {
			zip_many!{ ($i0:$t0) $($in:$tn)+ }
		};
		( ($($ic:tt:$tc:ident)+) $i0:tt:$t0:ident $($in:tt:$tn:ident)* ) => {
			impl<$($tc),+> Zip<($($tc,)+)> for ($(Option<$tc>,)+) {
				fn zip_options(self) -> Option<($($tc,)+)> {
					Some( ( $(self.$ic?,)+ ) )
				}
			}
			zip_many!{ ($($ic:$tc)+ $i0:$t0) $($in:$tn)* }
		};
		( ($($ic:tt:$tc:ident)+) ) => {};
	}
	zip_many!{ 0:T0 1:T1 2:T2 3:T3 4:T4 5:T5 6:T6 7:T7 8:T8 9:T9 10:T10 11:T11 }

	impl<T,const N:usize> Zip<[T;N]> for [Option<T>;N] {
		fn zip_options(self) -> Option<[T;N]> {
			for ov in self.iter() { ov.as_ref()?; }
			Some( self.map(|ov| ov.unwrap() ) )
		}
	}

}
pub use options::*;



mod iterators {

	use std::{
		iter::{Zip,Chain,Iterator},
		marker::Sized
	};

	pub trait ZipIters<A,B> {
		/// 2つのイテレータを同時にイテレートするイテレータを生成します
		fn zip(self) -> Zip<A,B>;
	}
	impl<A,B> ZipIters<A,B> for (A,B)
	where A:Iterator, B:Iterator
	{
		fn zip(self) -> Zip<A,B> {
			std::iter::zip(self.0,self.1)
		}
	}

	pub trait ChainIters<A,B> {
		/// 2つのイテレータを連結したイテレータを生成します
		fn chain(self) -> Chain<A,B>;
	}
	impl<A,B,T> ChainIters<A,B> for (A,B)
	where A:Iterator<Item=T>+Sized, B:Iterator<Item=T>
	{
		fn chain(self) -> Chain<A,B> {
			self.0.into_iter().chain(self.1.into_iter())
		}
	}

}
pub use iterators::*;



mod array {
	use super::*;
	use std::fmt::Debug;

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

	pub trait ZipArrays<T> {
		fn zip(self) -> T;
	}
	impl<A:Debug,B:Debug,const N:usize> ZipArrays<[(A,B);N]> for ([A;N],[B;N]) {
		fn zip(self) -> [(A,B);N] {
			// 標準の zip は unstable なので、カスタム実装する
			(self.0.into_iter(),self.1.into_iter())
			.zip()
			.collect::<Vec<_>>()
			.try_into()
			.unwrap()
		}
	}

}
pub use array::*;
