pub mod numerics;

mod logging;
pub use logging::*;

pub mod tuples;

#[cfg(feature="iterator")]
pub mod iterator;

mod misc;
pub use misc::*;

pub mod macros {
	extern crate macros;
	pub use macros::*;
}

mod macro_expansion;



/// このライブラリで定義された関数や型、トレイト、マクロなどにまとめてアクセスできるモジュール
/// `use utils::prelude::*;` とすることで全てのリソースがインポートされる
pub mod prelude {
	pub use super::{
		numerics::for_prelude::*,
		tuples::for_prelude::*,
		logging::for_prelude::*,
		misc::for_prelude::*,
		macros::*
	};
	#[cfg(feature="iterator")]
	pub use super::iterator::for_prelude::*;
}
/// このクレート内では、クレートで定義されたリソースを展開する
pub(crate) use prelude::*;
