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



#[cfg(feature="iterator")]
pub mod zip;
#[cfg(feature="iterator")]
pub mod product;

#[cfg(feature="iterator")]
pub mod chain;

#[cfg(feature="iterator")]
pub mod extended_map;

#[cfg(feature="iterator")]
pub mod misc;
