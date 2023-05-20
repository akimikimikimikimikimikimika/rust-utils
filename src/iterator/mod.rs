use super::*;
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



mod zip;
pub use zip::*;

mod product;
pub use product::*;

mod chain;
pub use chain::*;

mod extended_map;
pub use extended_map::*;

#[cfg(feature="iterator")]
mod misc;
#[cfg(feature="iterator")]
pub use misc::*;
