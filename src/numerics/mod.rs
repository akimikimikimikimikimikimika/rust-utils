use super::*;

pub mod basic_operations;

#[cfg(feature="numerics")]
pub mod primitive_functions;

#[cfg(feature="numerics")]
pub mod primitive_function_extensions;

#[cfg(feature="numerics")]
pub mod float;



/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::basic_operations::for_prelude::*;
	#[cfg(feature="numerics")]
	pub use super::{
		primitive_functions::for_prelude::*,
		primitive_function_extensions::for_prelude::*,
		float::for_prelude::*
	};

	#[cfg(feature="numerics")]
	extern crate num;
	#[cfg(feature="numerics")]
	pub use num::*;
}
