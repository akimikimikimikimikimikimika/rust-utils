use super::*;



/// 型に関する取り扱いを行うモジュール
mod types {
	use super::*;

	use std::{
		fmt::Display,
		convert::AsRef,
		path::Path
	};

	compose_struct! {
		/// 文字列を受け取るためのジェネリックな型
		pub trait AnyStr = AsRef<str> + Display;
		/// `Path` になりそうな型を受け取るためのジェネリックな型
		pub trait AnyPath = AsRef<Path> + Display;
	}

}
pub use types::*;



/// 簡単に所要時間を測定するタイマーモジュール
#[allow(dead_code)]
mod instant_timer {
	use super::*;

	use std::time::SystemTime;

	pub struct InstantTimer {
		start_time: SystemTime
	}

	impl InstantTimer {

		/// タイマーを開始させる
		pub fn start() -> InstantTimer {
			return InstantTimer {
				start_time: SystemTime::now()
			}
		}

		/// タイマーを終了させ、所要時間を出力する
		pub fn end(self) -> String {

			let end_time = SystemTime::now();

			let dur = end_time.duration_since(self.start_time).unwrap_or_error_in_detail_as("所要時間の計算に失敗しました");

			let mut text = format!("{:.3}秒", dur.as_secs_f64() % 60.0 );

			let sec = dur.as_secs();
			if sec >    60 { text = format!("{}分 "  ,sec/   60%60) + &text; }
			if sec >  3600 { text = format!("{}時間 ",sec/ 3600%24) + &text; }
			if sec > 86400 { text = format!("{}日 "  ,sec/86400   ) + &text; }

			return text;

		}

	}

}
pub use instant_timer::*;



#[cfg(feature="time_description")]
/// 時刻を取得するモジュール
mod time_description {
	extern crate once_cell;
	use once_cell::sync::Lazy;
	extern crate time;
	use time::{
		UtcOffset as Offset,
		OffsetDateTime as DateTime,
		format_description::{
			parse_owned as make_formatter,
			OwnedFormatItem as Formatter
		}
	};

	use super::*;

	static FORMATTER:Lazy<Formatter> = Lazy::new(|| {
		{
			#[cfg(feature="time_older")]
			{ make_formatter("[year]/[month padding:none]/[day padding:none] [hour]:[minute]:[second]") }
			#[cfg(not(feature="time_older"))]
			{ make_formatter::<2>("[year]/[month padding:none]/[day padding:none] [hour]:[minute]:[second]") }
		}
		.unwrap_or_error_in_detail_as("時刻のフォーマッタを生成するのに失敗しました")
	});

	static OFFSET:Lazy<Offset> = Lazy::new(|| {
		Offset::current_local_offset()
		.unwrap_or_error_in_detail_as("時刻のオフセットの取得に失敗しました")
	});

	/// 現在時刻を表すフォーマットされた文字列を返します
	pub fn current_time_description() -> String {
		DateTime::now_utc()
		.to_offset(*OFFSET)
		.format(&*FORMATTER)
		.unwrap_or_error_in_detail_as("現在時刻を取得するのに失敗しました")
	}

	pub trait DescribeTime {
		/// `SystemTime` 型を所定のフォーマットにした文字列を返します
		fn description(&self) -> String;
	}

	impl DescribeTime for std::time::SystemTime {
		fn description(&self) -> String {
			DateTime::from(self.clone())
			.to_offset(*OFFSET)
			.format(&*FORMATTER)
			.unwrap_or_error_in_detail_as("時刻を出力するのに失敗しました")
		}
	}

}
#[cfg(feature="time_description")]
pub use time_description::*;
