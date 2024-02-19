//! ## `special_functions`
//! このモジュールでは、特殊関数などを定義しています

use super::*;
extern crate once_cell;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use FloatCategory as Cat;
use primitive_functions::*;

mod integer_coefficients;
pub use integer_coefficients::*;

mod gamma_functions;
pub use gamma_functions::*;