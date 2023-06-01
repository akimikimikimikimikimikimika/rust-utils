use super::*;

use std::{
	backtrace::Backtrace,
	process::exit
};

/// Result 型や Option 型をアンラップして、エラーをログとして出力する
pub mod unwrap_result_option {
	use super::*;

	#[cfg(feature="logging")]
	extern crate log;

	pub trait UnwrapOrError<O> {
		/// アンラップし、失敗したらエラーメッセージを出して終了する
		fn unwrap_or_error_as(self,message:impl AnyStr) -> O;
	}
	pub trait UnwrapOrErrorInDetail<O,E> {
		/// アンラップし、失敗したらエラーメッセージを出して終了する (エラー型の内容とバックトレースも表示する)
		fn unwrap_or_error_in_detail_as(self,message:impl AnyStr) -> O;
	}
	pub trait UnwrapOrWarnForSameType<T> {
		/// アンラップし、失敗したら警告メッセージを出す (エラー型と成功型が同一の場合のみ)
		fn unwrap_or_warn_as(self,message:impl AnyStr) -> T;
	}

	/// トレートの実装
	impl<O,E> UnwrapOrError<O> for Result<O,E> where E: ToString {
		fn unwrap_or_error_as(self,message:impl AnyStr) -> O {
			self.unwrap_or_else(|_| {
				#[cfg(feature="logging")]
				log::error!("{}",message);
				#[cfg(not(feature="logging"))]
				eprintln!("ERROR: {}",message);
				exit(1);
			})
		}
	}
	impl<O> UnwrapOrError<O> for Option<O> {
		fn unwrap_or_error_as(self,message:impl AnyStr) -> O {
			self.unwrap_or_else(|| {
				#[cfg(feature="logging")]
				log::error!("{}",message);
				#[cfg(not(feature="logging"))]
				eprintln!("ERROR: {}",message);
				exit(1);
			})
		}
	}
	impl<O,E> UnwrapOrErrorInDetail<O,E> for Result<O,E> where E: ToString {
		fn unwrap_or_error_in_detail_as(self,message:impl AnyStr) -> O {
			self.unwrap_or_else(|e| {
				let b = Backtrace::force_capture();
				#[cfg(feature="logging")]
				log::error!(
					"{}: {}\nバックトレース:\n{}",
					message,e.to_string(), b
				);
				#[cfg(not(feature="logging"))]
				eprintln!(
					"エラー: {} ({})\nバックトレース:\n{}",
					message,e.to_string(), b
				);
				exit(1);
			})
		}
	}
	impl<T> UnwrapOrWarnForSameType<T> for Result<T,T> {
		fn unwrap_or_warn_as(self,message:impl AnyStr) -> T {
			self.unwrap_or_else(|v| {
				#[cfg(feature="logging")]
				log::warn!("{}",message);
				#[cfg(not(feature="logging"))]
				eprintln!("WARNING: {}",message);
				v
			})
		}
	}

}



/// エラー出力により終了するモジュール
pub mod fatal_error {
	use super::*;

	#[cfg(feature="logging")]
	extern crate log;

	/// エラーの出力
	pub fn fatal_error(message:impl AnyStr) -> ! {
		let b = Backtrace::force_capture();
		#[cfg(feature="logging")]
		log::error!(
			"{}\nバックトレース:{}",
			message, b
		);
		#[cfg(not(feature="logging"))]
		eprintln!(
			"{}\nバックトレース:{}",
			message, b
		);
		exit(1);
	}

	#[allow(unused_macros)]
	/// エラーをマクロ形式で展開する
	macro_rules! fatal_error {
		($($arg:tt)+) => {
			fatal_error(&format!($($arg)+));
		};
	}
	pub use fatal_error as fatal_error_fmt;

}



/// Result 型の結果を集約する
pub mod collect_result {
	use super::*;
	use std::{
		error::Error,
		backtrace::Backtrace,
		process::exit
	};

	pub type PropagatedError = (String,Backtrace);

	/// `Result` が `Err` の場合、それをバックトレース付きで伝播させる
	pub trait ResultPropagator<T> {
		/// エラーを伝播させる。 `catch_error` で囲む
		fn propagate(self) -> Result<T,PropagatedError>;
	}
	impl<T,E> ResultPropagator<T> for Result<T,E> where E: Error {
		fn propagate(self) -> Result<T,PropagatedError> {
			let b = Backtrace::force_capture();
			self.map_err(|e| (e.to_string(),b) )
		}
	}

	pub fn catch_error<T>(mut f:impl FnMut()->Result<T,PropagatedError>,error_msg:impl AnyStr) -> T {
		match f() {
			Ok(t) => t,
			Err((e,b)) => {
				eprintln!("エラー: {}\nエラー内容: {}\nバックトレース:\n{}",error_msg,e,b);
				exit(1);
			}
		}
	}

}



/// このモジュールからクレートの `prelude` でアクセスできるようにするアイテムをまとめたもの
pub(crate) mod for_prelude {
	pub use super::{
		unwrap_result_option::*,
		fatal_error::*,
		collect_result::*
	};
}
