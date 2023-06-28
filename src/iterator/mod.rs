use super::*;

pub mod zip;

pub mod product;

pub mod chain;

pub mod extended_map;

pub mod misc;



/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::{
		extended_map::for_prelude::*,
		zip::for_prelude::*,
		product::for_prelude::*,
		chain::for_prelude::*,
		misc::for_prelude::*
	};

	pub(crate) use std::iter::{
		Iterator,
		ExactSizeIterator,
		DoubleEndedIterator,
		FusedIterator
	};
	#[cfg(feature="parallel")]
	pub(crate) use rayon::iter::{
		plumbing as rayon_plumbing,
		ParallelIterator,
		IndexedParallelIterator,
		IntoParallelIterator
	};
}
