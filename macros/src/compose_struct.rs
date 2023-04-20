use proc_macro::TokenStream as TS1;
use proc_macro2::{
	TokenStream as TS, TokenTree as TT,
	Ident, Literal, Delimiter, Span
};
use quote::{quote,ToTokens};
use std::mem::swap;

macro_rules! compose_struct_interface { ()=>{

	#[proc_macro]
	pub fn compose_struct(item:TokenStream) -> TokenStream {
		//! ## `compose_struct`
		//! 簡単な表式で複合的な `struct` や `enum` を生成できます。また `type` による型エイリアス、 `trait` によるトレイトエイリアスを定義できます。
		//!
		//! ### 特徴
		//!
		//! #### 構造体/列挙体の内部に別の構造体/列挙体を定義可能
		//! * 視覚的に構造体間の関係を捉えやすくなります
		//!
		//! ```rust
		//! struct Coord {
		//! 	z:f64,
		//! 	xy = enum XYPlane {
		//! 		Orthogonal { x:f64, y:f64 },
		//! 		Polar(
		//! 			struct XYPolarCoord { r:f64, θ:f64 }
		//! 		)
		//! 	}
		//! }
		//! ```
		//!
		//! * `derive`, `allow`, `cfg` などのアトリビュートは依存関係がありうるので、親の構造体で指定されると、子の構造体にも再帰的にアトリビュートが付加されます
		//!
		//! ```rust
		//! #[derive(Clone,Copy)]
		//! struct Coord {
		//! 	z:f64,
		//! 	#[allow(dead_code)]
		//! 	xy = enum XYPlane { // #[derive(Clone,Copy)] が継承される
		//! 		Orthogonal { x:f64, y:f64 },
		//! 		Polar(
		//! 			// #[derive(Clone,Copy)] が継承される
		//! 			struct XYPolarCoord { r:f64, θ:f64 }
		//! 		)
		//! 	}
		//! }
		//! ```
		//!
		//! #### デフォルト値を指定できる
		//! * 別途 `impl Default` を用意しなくても対にして定義できます
		//!
		//! ```rust
		//! struct Coord {
		//! 	x:f64 = 0.0,
		//! 	y:f64 = 0.0,
		//! 	z:f64 = 0.0
		//! }
		//! ```
		//!
		//! * 列挙体については `= default` 或いは `#[default]` アトリビュートによりバリアントをデフォルト値を指定できます
		//! * フィールド付きのバリアントについては、フィールドにデフォルト値が与えられれば、そのバリアントはデフォルト値になります
		//! * 内包する構造体や列挙体には、バリアントがデフォルト値かどうかに関係なくデフォルト値を定めることは可能です
		//!
		//! ```rust
		//! enum Coordinate {
		//! 	Orthogonal = default, // これがデフォルト値
		//! 	Polar( struct PolarCoord { r:f64 = 0.0, θ:f64 = 0.0 } )
		//! }
		//! ```
		//!
		//! * `#[default]` アトリビュートを付すと、型に合わせたデフォルト値を自動的に定めます
		//!
		//! ```rust
		//! struct Coord {
		//! 	#[default]
		//! 	z:f64, // f64 のデフォルト値 0.0 が与えられます
		//! 	x:f64 = 1.0,
		//! 	y:f64 = 1.0
		//! }
		//! ```
		//!
		//! #### 型やトレイトのエイリアスを指定可能
		//! * 通常通り型の定義ができるのはもちろんのこと、 stable でないトレイトのエイリアスも用意できます。
		//!
		//! ```rust
		//! compose_struct! {
		//! 	/// `None` を許容する実数型
		//! 	pub type NullableFloat = Option<f64>;
		//! 	/// 任意の文字列
		//! 	pub trait AnyStr = std::convert::Into<String> + std::fmt::Display;
		//! 	/// クローン可能な `u8` 型イテレータ
		//! 	trait IntIter = Iterator<Item=u8> + Clone;
		//! }
		//! ```
		//!
		//! * 構造体や列挙体の内部で型エイリアスを定義することもできます。フィールドの近くに配置できるので関係性が視覚的にわかりやすくなります。
		//!
		//! ```rust
		//! compose_struct! {
		//! 	struct Shape {
		//! 		kind = enum ShapeKind {
		//! 			Rect, Oval
		//! 		},
		//!
		//! 		type Coord = (f64,f64);
		//! 		coord_left_top: Coord,
		//! 		coord_right_bottom: Coord
		//! 	}
		//! }
		//! ```
		//!
		//! ### 例
		//! ```rust
		//! compose_struct! {
		//!
		//! 	#[derive(Clone,Copy)]
		//! 	/// 座標を表す
		//! 	struct Coord {
		//! 		z:f64 = 0.0,
		//! 		#[allow(dead_code)]
		//! 		/// XY平面の座標を表す
		//! 		xy = enum XYPlane {
		//! 			Unspecified = default,
		//! 			Orthogonal {
		//! 				x:f64,
		//! 				y:f64
		//! 			},
		//! 			Polar( struct XYPolarCoord {
		//! 				r:f64 = 0.0,
		//! 				θ:f64
		//! 			} )
		//! 		}
		//! 	}
		//!
		//! }
		//! ```
		compose_struct_impl(item)
	}

}; }
pub(crate) use compose_struct_interface;

/// `compose_struct!` マクロの実装のエントリポイント
pub fn compose_struct_impl(ts:TS1) -> TS1 {
	let mut root = parse(TS::from(ts.clone()));
	modify(&mut root);
	let generated = compose(root);
	TS1::from(generated)
}



/// データ構造を定義するモジュール
mod typedef {
	use super::*;

	/// パースしたデータのルート
	pub struct Root {
		/// デバッグ出力を有効にする
		pub debug: bool,
		/// データ型のリスト
		pub datum: Vec<Data>,
		/// 元のソースコード
		pub src: String
	}

	/// 構造体、列挙体、エイリアスを抽象化したデータ型
	pub enum Data {
		/// 構造体
		Struct(Struct),
		/// 列挙体
		Enum(Enum),
		/// 型エイリアス
		Type(TypeAlias),
		/// トレイトエイリアス
		Trait(TraitAlias),
		/// デバッグフラグ
		Debug
	}

	/// 構造体を表す型
	pub struct Struct {
		/// 構造体の名前
		pub name: Ident,
		/// 構造体の名前に付されたジェネリクスパラメータ
		pub generics: TS,
		/// 構造体に付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// `pub` などの構造性 (列挙体にアクセス可能な範囲) の情報
		pub visibility: TS,
		/// `where` によるジェネリクスの拘束条件
		pub where_condition: TS,
		/// 構造体のフィールドのリスト
		pub fields: Vec<StructField>,
		/// 内包する別のデータ型
		pub enclosed: Vec<Data>,
		/// 元のソースコード
		pub src: String
	}

	/// 構造体のフィールドを表す
	pub struct StructField {
		/// フィールドに付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// `pub` などの可視性 (フィールドにアクセス可能な範囲) の情報
		pub visibility: TS,
		/// フィールドの名前
		pub name: Ident,
		/// フィールドの値
		pub value: FieldValue,
		/// 元のソースコード
		pub src: String
	}

	/// 列挙体を表す型
	pub struct Enum {
		/// 列挙体の名前
		pub name: Ident,
		/// 列挙体の名前に付されたジェネリクスパラメータ
		pub generics: TS,
		/// 列挙体に付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// `pub` などの可視性 (列挙体にアクセス可能な範囲) の情報
		pub visibility: TS,
		/// `where` によるジェネリクスの拘束条件
		pub where_condition: TS,
		/// 列挙体の要素のリスト
		pub variants: Vec<EnumVariant>,
		/// 内包する別のデータ型
		pub enclosed: Vec<Data>,
		/// 元のソースコード
		pub src: String
	}

	/// 列挙体の要素を表す
	pub struct EnumVariant {
		/// 要素に付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// 要素名
		pub name: Ident,
		/// 要素に付されたフィールドの情報
		pub fields: EnumFields,
		/// この要素が列挙体のデフォルト値になっているか
		pub is_default: bool,
		/// 元のソースコード
		pub src: String
	}

	/// 列挙体のフィールド (ある場合、ない場合の双方) を表す
	pub enum EnumFields {
		/// フィールドがない (単位要素) の場合
		Unit,
		/// フィールド名のないフィールドの場合 (フィールドのリスト)
		Unnamed(Vec<EnumUnnamedField>),
		/// フィールド名のあるフィールドの場合
		Named(Vec<EnumNamedField>)
	}

	/// 列挙体のフィールド名のないフィールドを表す
	pub struct EnumUnnamedField {
		/// フィールドに付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// フィールドの値
		pub value: FieldValue,
		/// 元のソースコード
		pub src: String
	}

	/// 列挙体のフィールド名のあるフィールドを表す
	pub struct EnumNamedField {
		/// フィールドに付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// フィールドの名前
		pub name: Ident,
		/// フィールドの値
		pub value: FieldValue,
		/// 元のソースコード
		pub src: String
	}

	/// 構造体や列挙体のフィールドの値を表す
	pub enum FieldValue {
		/// 任意の型の値である場合
		Type {
			/// 型の種類
			name: TS,
			/// デフォルト値 (ある場合)
			default: Option<TS>
		},
		/// 別の構造体や列挙体を含む場合
		Data(Data)
	}

	/// `type A = B` の型エイリアスを表す
	pub struct TypeAlias {
		/// エイリアス名
		pub name:TS,
		/// エイリアスの実態
		pub artifact:TS,
		/// エイリアスに付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// `pub` などの可視性 (エイリアスにアクセス可能な範囲) の情報
		pub visibility: TS,
		/// 元のソースコード
		pub src: String
	}

	/// `trait A = B` のトレイトエイリアスを表す
	pub struct TraitAlias {
		/// エイリアス名
		pub name:Ident,
		/// エイリアス名に含まれるジェネリクスパラメータ
		pub generics:TS,
		/// エイリアスの実態
		pub artifact:TS,
		/// エイリアスに付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// `pub` などの可視性 (エイリアスにアクセス可能な範囲) の情報
		pub visibility: TS,
		/// 元のソースコード
		pub src: String
	}

	#[derive(Clone)]
	/// アトリビュートを表す。アトリビュートの種類ごとに分けている
	pub enum Attr {
		/// `#[derive(..)]` アトリビュート
		Derive(Vec<Ident>),
		/// `#[allow(..)]` アトリビュート
		Allow(Vec<Ident>),
		/// `#[cfg(..)]` アトリビュート
		Cfg(TS),
		/// `#[doc=".."]` アトリビュート
		Doc(Literal),
		/// `#[default]` アトリビュート
		Default,
		/// `#[pub_all]` アトリビュート
		PubAll,
		/// その他の全てのアトリビュート
		Other(TS)
	}

	/// 構造体や列挙体のヘッダーをパースした結果。それぞれの場合でさらに `body`の内容をパースして `Struct` や `Enum` を使用する
	pub struct ParsingResult {
		/// 付されたアトリビュートのリスト
		pub attr: Vec<Attr>,
		/// `pub` などのアクセス可能な範囲の情報
		pub vis: TS,
		/// 構造体/列挙体の名前
		pub name: Ident,
		/// ジェネリクスのパラメータ
		pub generics: TS,
		/// `where` によるジェネリクスの拘束条件
		pub wh: TS,
		/// `{ ... }` の中身
		pub body: TS,
		/// 元のソースコード
		pub src: String
	}

}
use typedef::*;

/// ユーティリティ群
mod utils {
	use super::*;

	pub trait Join {
		/// 要素をコンマで繋いだトークンストリームを生成します。
		fn comma_join(&self) -> TS;
	}
	impl<T> Join for Vec<T> where T: ToTokens {
		fn comma_join(&self) -> TS {
			if self.len()==0 { return quote!(); }
			let first = &self[0];
			let mut joined = quote!( #first );
			for item in self.iter().skip(1) {
				joined = quote!( #joined , #item );
			}
			joined
		}
	}

	use std::{
		process::exit,
		fmt::Display,
		convert::Into,
		iter::Peekable
	};

	pub type PI<I> = Peekable<I>;

	/// トレイトの別名を定義するマクロ
	macro_rules! trait_def {
		( $(
			$( #[doc=$doc:literal] )?
			$alias:ident { $($im:tt)+ }
		)+ ) => { $(
			$( #[doc=$doc] )?
			pub trait $alias: $($im)+ {}
			impl<T> $alias for T where T: $($im)+ {}
		)+ };
	}

	trait_def! {
		/// 文字列を受け取るためのジェネリックな型
		AnyStr { Into<String> + Display }
		/// クローン可能なトークンツリーのイテレータ
		TI { Iterator<Item=TT> + Clone }
	}

	/// エラーで終了するモジュール
	pub fn error(msg:impl AnyStr,src:Option<&str>) -> ! {
		let output = format!(
			"エラー: compose_struct! のパースに失敗しました\n内容: {}{}",
			msg,
			match src {
				Some(s) => format!("\n該当箇所:\n{}",s),
				None => String::new()
			}
		);
		eprintln!("{}",output);
		exit(1)
	}

}
use utils::*;



/// トークンストリームのパーサーを含むモジュール
mod parser {
	use super::*;

	/// 生データのパーサを含むモジュール。 `{ ... }` の外側をパースする。
	mod data_parser {
		use super::*;

		/// 生の入力データをパースする
		/// * 含まれる構造体/列挙体のリストを返す
		/// * 外の部分だけパースし、 `{ ... }` の内部は構造体/列挙体のパーサーにそれぞれ渡す。
		pub fn parse(ts:TS) -> Root {
			let src = ts.to_string();

			let mut datum:Vec<Data> = vec![];
			let mut debug = false;
			let mut iter = ts.into_iter().peekable();

			while let Some(d) = parse_each(&mut iter) {
				datum.push(d);
			}

			datum = datum.into_iter()
			.filter(|d| {
				if matches!(d,Data::Debug) {
					debug = true;
					false
				}
				else { true }
			})
			.collect::<Vec<_>>();

			if datum.is_empty() {
				error("構造体や列挙体などが1つも見つかりませんでした",None);
			}

			Root { datum, debug, src }
		}

		/// データをパースする
		/// * ここでは正確に1つのデータのみを含む場合をパースする。複数ある場合はエラーを返す。
		/// * 構造体/列挙体に含まれる別の構造体/列挙体をパースする際に使う
		pub fn parse_one(ts:TS) -> Data {
			let mut iter = ts.clone().into_iter().peekable();
			let first = parse_each(&mut iter);
			let second = parse_each(&mut iter);

			match (first,second) {
				(Some(d),None) => d,
				(None,None)|(Some(_),Some(_)) => {
					error(
						"複数のデータを受け取りました",
						Some(&ts.to_string())
					)
				},
				(None,Some(_)) => { unreachable!(); }
			}
		}

		/// トークンストリームのイテレータを進めて、データを1つだけ解析したら返す
		fn parse_each(iter:&mut PI<impl TI>) -> Option<Data> {
			let src = TS::from_iter(iter.clone()).to_string();

			let mut phase = PP::Beginning;
			let mut attr:Vec<Attr> = vec![];
			let mut vis = TS::new();
			let mut ty = Type::Unknown;
			let mut name:Option<Ident> = None;
			let mut generics = TS::new();
			let mut generics_enclosure_count = 0_u8;
			let mut wh = TS::new();
			let mut body = TS::new();
			let mut whole = TS::new();

			loop {
				let tt = match iter.next() {
					Some(t) => t,
					None => { break }
				};
				let s = tt.to_string();

				match (&phase,&s[..],tt.clone(),&ty) {
					(PP::Beginning,"debug",_,Type::Unknown) => {
						phase = PP::GotType;
						ty = Type::Debug;
						if iter.peek().map_or(
							false,
							|t| t.to_string()==";"
						) {
							let _ = iter.next();
						}
						break
					},
					(PP::Beginning|PP::GotAttrBody,"#",_,Type::Unknown) => {
						phase = PP::GotAttrHash;
					},
					(PP::GotAttrHash,_,TT::Group(g),Type::Unknown) => {
						attr.push( parse_attr(g.stream()) );
						phase = PP::GotAttrBody;
					}
					(PP::Beginning|PP::GotAttrBody,"pub",_,Type::Unknown) => {
						vis = quote!(pub);
						phase = PP::GotPub;
					},
					(PP::GotPub,_,TT::Group(g),Type::Unknown) => {
						match g.delimiter() {
							Delimiter::Parenthesis => {
								let t = TT::Group(g);
								vis = quote!( #vis #t );
								phase = PP::GotVisiblity;
							},
							_ => error(
								"予期しない括弧にマッチしました",
								Some(&src)
							)
						}
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotPub|PP::GotVisiblity,"struct",_,Type::Unknown) => {
						ty = Type::Struct;
						phase = PP::GotType;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotPub|PP::GotVisiblity,"enum",_,Type::Unknown) => {
						ty = Type::Enum;
						phase = PP::GotType;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotPub|PP::GotVisiblity,"type",_,Type::Unknown) => {
						ty = Type::TypeAlias;
						phase = PP::GotType;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotPub|PP::GotVisiblity,"trait",_,Type::Unknown) => {
						ty = Type::TraitAlias;
						phase = PP::GotType;
					},
					(PP::GotType,_,TT::Ident(i),_) => {
						name = Some(i);
						phase = PP::GotName;
					},
					(PP::GotName,"<",_,_) => {
						generics_enclosure_count += 1;
						phase = PP::GotGenericsBegin;
					},
					(PP::GotGenericsBegin|PP::GotGenerics,"<",t,_) => {
						generics = quote!(#generics #t);
						generics_enclosure_count += 1;
						phase = PP::GotGenerics;
					},
					(PP::GotGenericsBegin|PP::GotGenerics,">",t,_) => {
						generics_enclosure_count -= 1;
						if generics_enclosure_count>0 {
							generics = quote!(#generics #t);
							phase = PP::GotGenerics;
						}
						else {
							phase = PP::GotGenericsEnd;
						}
					},
					(PP::GotGenericsBegin|PP::GotGenerics,_,t,_) => {
						generics = quote!(#generics #t);
						phase = PP::GotGenerics;
					},
					(PP::GotName|PP::GotGenericsEnd,"where",_,Type::Struct|Type::Enum) => {
						if generics_enclosure_count!=0 {
							error(
								format!("予期しないトークン {} が含まれています",s),
								Some(&src)
							);
						}
						phase = PP::GotWhere;
					},
					(PP::GotName|PP::GotGenericsEnd,_,TT::Group(g),Type::Struct|Type::Enum) => {
						match g.delimiter() {
							Delimiter::Brace => {
								body = g.stream();
								phase = PP::GotBody;
								break;
							},
							_ => error(
								"予期しない括弧にマッチしました",
								Some(&src)
							)
						}
					},
					(PP::GotWhereItem,_,TT::Group(g),Type::Struct|Type::Enum) => {
						match g.delimiter() {
							Delimiter::Brace => {
								body = g.stream();
								phase = PP::GotBody;
								break;
							},
							_ => {
								wh = quote!( #wh #g );
							}
						}
					},
					(PP::GotWhere|PP::GotWhereItem,_,t,Type::Struct|Type::Enum) => {
						wh = quote!(#wh #t);
						phase = PP::GotWhereItem;
					},
					(PP::GotName|PP::GotGenericsEnd,"=",_,Type::TypeAlias|Type::TraitAlias) => {
						phase = PP::GotEqual;
					},
					(PP::GotArtifact,";",_,Type::TypeAlias|Type::TraitAlias) => {
						phase = PP::GotSemicolon;
						break;
					},
					(PP::GotEqual|PP::GotArtifact,_,t,Type::TypeAlias|Type::TraitAlias) => {
						body = quote!( #body #t );
						phase = PP::GotArtifact;
					},
					_ => error(
						format!("予期しないトークン {} が含まれています",s),
						Some(&src)
					)
				}

				whole = quote!( #whole #tt );
			}

			match (&ty,phase) {
				(Type::Struct|Type::Enum,PP::GotBody)|(Type::TypeAlias|Type::TraitAlias,PP::GotSemicolon) => {},
				(Type::Debug,PP::GotType) => { return Some(D::Debug); },
				(Type::Unknown,PP::Beginning) => { return None; },
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}

			let pr = ParsingResult {
				attr, vis,
				name: name.unwrap(),
				generics, wh, body,
				src: whole.to_string()
			};

			Some( match &ty {
				Type::Struct => D::Struct(parse_struct(pr)),
				Type::Enum => D::Enum(parse_enum(pr)),
				Type::TypeAlias => D::Type(parse_type_alias(pr)),
				Type::TraitAlias => D::Trait(parse_trait_alias(pr)),
				_ => { unreachable!(); }
			} )
		}

		/// 新しいデータの種類を表す
		enum Type {
			/// 構造体
			Struct,
			/// 列挙体
			Enum,
			/// 型エイリアス
			TypeAlias,
			/// トレイトエイリアス
			TraitAlias,
			/// デバッグフラグ
			Debug,
			/// まだ定まっていない
			Unknown
		}

		/// 現在のパースの過程を表す
		enum ParsingPhase {
			Beginning, GotAttrHash, GotAttrBody,
			GotPub, GotVisiblity,
			GotType, GotName,
			GotGenericsBegin, GotGenerics, GotGenericsEnd,
			GotWhere, GotWhereItem, GotBody,
			GotEqual, GotArtifact, GotSemicolon
		}
		type PP = ParsingPhase;

		/// 型エイリアスをパースする
		fn parse_type_alias(pr:ParsingResult) -> TypeAlias {
			let ParsingResult {
				name, mut generics, body, attr, vis, src, ..
			} = pr;
			if generics.is_empty() {
				generics = quote!( <#generics> );
			}

			TypeAlias {
				name: quote!( #name #generics ),
				artifact: body,
				attributes: attr,
				visibility: vis,
				src
			}
		}

		/// トレイトエイリアスをパースする
		fn parse_trait_alias(pr:ParsingResult) -> TraitAlias {
			let ParsingResult {
				name, generics, body, attr, vis, src, ..
			} = pr;

			TraitAlias {
				name, generics,
				artifact: body,
				attributes: attr,
				visibility: vis,
				src
			}
		}

		type D = Data;
	}
	use data_parser::*;
	pub use data_parser::parse;

	/// 構造体のパーサ
	mod struct_parser {
		use super::*;

		/// 構造体をパースする
		pub fn parse_struct(pr:PPR) -> S {
			let mut fields:Vec<SF> = vec![];
			let mut enclosed:Vec<Data> = vec![];
			let mut iter = pr.body.into_iter();

			loop {
				match parse_field(&mut iter) {
					PR::Field(f) => fields.push(f),
					PR::Data(d) => enclosed.push(d),
					PR::None => break
				}
			}

			if fields.is_empty() {
				error(
					"フィールドの数を 0 にすることはできません",
					Some(&pr.src)
				);
			}

			S {
				name: pr.name,
				generics: pr.generics,
				attributes: pr.attr,
				visibility: pr.vis,
				where_condition: pr.wh,
				fields, enclosed,
				src: pr.src
			}
		}

		/// 構造体のそれぞれのフィールドをパースする
		fn parse_field(iter:&mut impl TI) -> PR {
			let src = TS::from_iter(iter.clone()).to_string();

			let mut phase = PP::Beginning;
			let mut enclosed = false;
			let mut attr:Vec<Attr> = vec![];
			let mut vis = TS::new();
			let mut name:Option<Ident> = None;
			let mut ty = TS::new();
			let mut generics_count = 0_u8;
			let mut default = TS::new();
			let mut is_subtype = false;
			let mut whole = TS::new();

			loop {
				let tt = match iter.next() {
					Some(t) => t,
					None => { break }
				};
				let s = tt.to_string();

				match (&phase,&s[..],tt.clone()) {
					(PP::Beginning|PP::GotAttrBody,"#",_) => {
						phase = PP::GotAttrHash;
					},
					(PP::GotAttrHash,_,TT::Group(g)) => {
						attr.push( parse_attr(g.stream()) );
						phase = PP::GotAttrBody;
					}
					(PP::Beginning|PP::GotAttrBody,"pub",_) => {
						vis = quote!(pub);
						phase = PP::GotPub;
					},
					(PP::GotPub,_,TT::Group(g)) => {
						match g.delimiter() {
							Delimiter::Parenthesis => {
								vis = quote!( #vis #g );
								phase = PP::GotVisiblity;
							},
							_ => error(
								"予期しない括弧にマッチしました",
								Some(&src)
							)
						}
					},
					(PP::Beginning|PP::GotPub|PP::GotVisiblity|PP::GotAttrBody,"struct"|"enum"|"type"|"trait",_) => {
						phase = PP::GotEnclosedType;
						enclosed = true;
					},
					(PP::Beginning|PP::GotPub|PP::GotVisiblity|PP::GotAttrBody,_,TT::Ident(i)) => {
						name = Some(i);
						phase = PP::GotName;
					},
					(PP::GotName,":",_) => {
						phase = PP::GotColon;
					},
					(PP::GotName,"=",_) => {
						phase = PP::GotEqual;
					},
					(PP::GotType,"<",t) => {
						generics_count += 1;
						ty = quote!( #ty #t );
					},
					(PP::GotType,">",t) => {
						generics_count -= 1;
						ty = quote!( #ty #t );
					},
					(PP::GotType,"=",t) => {
						if generics_count!=0 {
							ty = quote!( #ty #t );
						}
						else { phase = PP::GotEqual; }
					},
					(PP::GotType,",",t) => {
						if generics_count!=0 {
							ty = quote!( #ty #t );
						}
						else {
							phase = PP::GotComma;
							break;
						}
					},
					(PP::GotColon|PP::GotType,_,t) => {
						ty = quote!( #ty #t );
						phase = PP::GotType;
					},
					(PP::GotEqual,"struct"|"enum",t) => {
						is_subtype = true;
						default = quote!(#t);
						phase = PP::GotSubValType;
					},
					(PP::GotSubValHeader,_,TT::Group(g)) => {
						if matches!(g.delimiter(),Delimiter::Brace) {
							phase = PP::GotSubValBody;
						}
						default = quote!( #default #g );
					},
					(PP::GotSubValType|PP::GotSubValHeader,_,t) => {
						default = quote!( #default #t );
						phase = PP::GotSubValHeader;
					},
					(PP::GotSubValBody|PP::GotDefaultVal,",",_) => {
						phase = PP::GotComma;
						break;
					},
					(PP::GotEqual|PP::GotDefaultVal,_,t) => {
						default = quote!( #default #t );
						phase = PP::GotDefaultVal;
					},
					(PP::GotEnclosedHeader,_,TT::Group(g)) => {
						if matches!(g.delimiter(),Delimiter::Brace) {
							phase = PP::GotEnclosedBody;
							whole = quote!( #whole #tt );
							break;
						}
					},
					(PP::GotEnclosedHeader,";",_) => {
						phase = PP::GotSemicolon;
						whole = quote!( #whole #tt );
						break;
					},
					(PP::GotEnclosedType|PP::GotEnclosedHeader,_,_) => {
						phase = PP::GotEnclosedHeader;
					},
					_ => error(
						format!("予期しないトークン {} が含まれています",s),
						Some(&src)
					)
				}

				whole = quote!( #whole #tt );
			}

			match (phase,enclosed) {
				(PP::GotComma|PP::GotType|PP::GotDefaultVal|PP::GotSubValBody,false) => {
					PR::Field( SF {
						attributes: attr,
						visibility: vis,
						name: name.unwrap(),
						value: match (default.is_empty(),is_subtype) {
							(true,false) => FV::Type {
								name: ty,
								default: None
							},
							(false,false) => FV::Type {
								name: ty,
								default: Some(default)
							},
							(false,true) => FV::Data(parse_one(default)),
							(true,true) => { unreachable!() }
						},
						src: whole.to_string()
					} )
				},
				(PP::GotComma|PP::GotSemicolon|PP::GotEnclosedBody,true) => {
					PR::Data( parse_one(whole) )
				},
				(PP::Beginning,_) => { return PR::None; },
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}
		}

		/// 現在のパースの過程を表す
		enum ParsingPhase {
			Beginning, GotAttrHash, GotAttrBody,
			GotPub, GotVisiblity,
			GotName, GotColon, GotType,
			GotEqual, GotDefaultVal,
			GotSubValType, GotSubValHeader, GotSubValBody,
			GotEnclosedType, GotEnclosedHeader, GotEnclosedBody,
			GotComma, GotSemicolon
		}
		type PP = ParsingPhase;

		enum ParsingResult {
			Field(SF),
			Data(Data),
			None
		}
		type PR = ParsingResult;

		type PPR = super::ParsingResult;
		type S = Struct;
		type SF = StructField;
		type FV = FieldValue;

	}
	use struct_parser::*;

	/// 列挙体のパーサ
	mod enum_parser {
		use super::*;

		/// 列挙体をパースする
		pub fn parse_enum(pr:PPR) -> E {
			let mut variants:Vec<EV> = vec![];
			let mut enclosed:Vec<Data> = vec![];
			let mut iter = pr.body.into_iter();

			loop {
				match parse_variant(&mut iter) {
					PR::Variant(v) => variants.push(v),
					PR::Data(d) => enclosed.push(d),
					PR::None => break
				}
			}

			if variants.is_empty() {
				error(
					"バリアントの数を 0 にすることはできません",
					Some(&pr.src)
				);
			}

			E {
				name: pr.name,
				generics: pr.generics,
				attributes: pr.attr,
				visibility: pr.vis,
				where_condition: pr.wh,
				variants, enclosed,
				src: pr.src
			}
		}

		/// 列挙体のそれぞれのバリアントをパースする
		fn parse_variant(iter:&mut impl TI) -> PR {
			let src = TS::from_iter(iter.clone()).to_string();

			let mut phase = PP::Beginning;
			let mut enclosed = false;
			let mut attr:Vec<Attr> = vec![];
			let mut name:Option<Ident> = None;
			let mut fields = EFS::Unit;
			let mut is_default = false;
			let mut whole = TS::new();

			loop {
				let tt = match iter.next() {
					Some(t) => t,
					None => { break }
				};
				let s = tt.to_string();

				match (&phase,&s[..],tt.clone()) {
					(PP::Beginning|PP::GotAttrBody,"#",_) => {
						phase = PP::GotAttrHash;
					},
					(PP::GotAttrHash,_,TT::Group(g)) => {
						attr.push( parse_attr(g.stream()) );
						phase = PP::GotAttrBody;
					},
					(PP::Beginning|PP::GotAttrBody,"struct"|"enum"|"type"|"trait",_) => {
						phase = PP::GotEnclosedType;
						enclosed = true;
					},
					(PP::Beginning|PP::GotAttrBody,_,TT::Ident(i)) => {
						name = Some(i);
						phase = PP::GotFieldName;
					},
					(PP::GotFieldName,_,TT::Group(g)) => {
						match g.delimiter() {
							Delimiter::Parenthesis => {
								fields = parse_enum_unnamed_fields(g.stream());
								phase = PP::GotFieldValue;
							},
							Delimiter::Brace => {
								fields = parse_enum_named_fields(g.stream());
								phase = PP::GotFieldValue;
							},
							_ => error(
								"予期しない括弧にマッチしました",
								Some(&src)
							)
						}
					},
					(PP::GotFieldName|PP::GotFieldValue,"=",_) => {
						phase = PP::GotEqual;
					},
					(PP::GotEqual,"default",_) => {
						is_default = true;
						phase = PP::GotDefault;
					},
					(PP::GotFieldName|PP::GotDefault|PP::GotFieldValue,",",_) => {
						phase = PP::GotComma;
						break;
					},
					(PP::GotEnclosedHeader,_,TT::Group(g)) => {
						if matches!(g.delimiter(),Delimiter::Brace) {
							phase = PP::GotEnclosedBody;
							whole = quote!( #whole #tt );
							break;
						}
					},
					(PP::GotEnclosedHeader,";",_) => {
						phase = PP::GotSemicolon;
						whole = quote!( #whole #tt );
						break;
					},
					(PP::GotEnclosedType|PP::GotEnclosedHeader,_,_) => {
						phase = PP::GotEnclosedHeader;
					},
					_ => error(
						format!("予期しないトークン {} が含まれています",s),
						Some(&src)
					)
				}

				whole = quote!( #whole #tt );
			}

			match (phase,enclosed) {
				(PP::GotFieldName|PP::GotDefault|PP::GotFieldValue|PP::GotComma,false) => {
					PR::Variant( EV {
						attributes: attr,
						name: name.unwrap(),
						fields, is_default,
						src: whole.to_string(),
					} )
				},
				(PP::GotEnclosedBody|PP::GotSemicolon,true) => {
					PR::Data( parse_one(whole) )
				},
				(PP::Beginning,_) => { return PR::None; },
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}
		}

		/// 現在のパースの過程を表す
		enum ParsingPhase {
			Beginning, GotAttrHash, GotAttrBody,
			GotFieldName, GotFieldValue,
			GotEqual, GotDefault,
			GotEnclosedType, GotEnclosedHeader, GotEnclosedBody,
			GotComma, GotSemicolon
		}
		type PP = ParsingPhase;

		enum ParsingResult {
			Variant(EV),
			Data(Data),
			None
		}
		type PR = ParsingResult;

		type PPR = super::ParsingResult;
		type E = Enum;
		type EV = EnumVariant;
		type EFS = EnumFields;
	}
	use enum_parser::*;

	/// 列挙体の名前なしフィールドのパーサ
	mod enum_unnamed_fields_parser {
		use super::*;

		/// 列挙体の名前なしフィールドをパースする
		pub fn parse_enum_unnamed_fields(ts:TS) -> FS {
			let mut fields:Vec<F> = vec![];
			let mut iter = ts.into_iter();

			while let Some(f) = parse_field(&mut iter) {
				fields.push(f);
			}

			FS::Unnamed(fields)
		}

		/// それぞれのフィールドをパースする
		fn parse_field(iter:&mut impl TI) -> Option<F> {
			let src = TS::from_iter(iter.clone()).to_string();

			let mut phase = PP::Beginning;
			let mut attr:Vec<Attr> = vec![];
			let mut is_subtype = false;
			let mut ty = TS::new();
			let mut type_generics_count = 0_u8;
			let mut default = TS::new();
			let mut whole = TS::new();

			loop {
				let tt = match iter.next() {
					Some(t) => t,
					None => { break }
				};
				let s = tt.to_string();

				match (&phase,&s[..],tt.clone()) {
					(PP::Beginning|PP::GotAttrBody|PP::GotComma,"#",_) => {
						phase = PP::GotAttrHash;
					},
					(PP::GotAttrHash,_,TT::Group(g)) => {
						attr.push( parse_attr(g.stream()) );
						phase = PP::GotAttrBody;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotComma,"struct"|"enum",t) => {
						is_subtype = true;
						default = quote!(#t);
						phase = PP::GotSubValType;
					},
					(PP::GotSubValHeader,_,TT::Group(g)) => {
						if matches!(g.delimiter(),Delimiter::Brace) {
							phase = PP::GotSubValBody;
						}
						default = quote!( #default #g );
					},
					(PP::GotSubValType|PP::GotSubValHeader,_,t) => {
						default = quote!( #default #t );
						phase = PP::GotSubValHeader;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotComma,_,TT::Ident(i)) => {
						ty = quote!(#i);
						phase = PP::GotType;
					},
					(PP::GotType,"<",t) => {
						type_generics_count += 1;
						ty = quote!( #ty #t );
					},
					(PP::GotType,">",t) => {
						type_generics_count -= 1;
						ty = quote!( #ty #t );
					},
					(PP::GotType,"=",t) => {
						if type_generics_count!=0 {
							ty = quote!( #ty #t );
						}
						else {
							phase = PP::GotEqual;
						}
					},
					(PP::GotType|PP::GotDefaultVal|PP::GotSubValBody,",",_) => {
						phase = PP::GotComma;
						break;
					},
					(PP::GotType,_,t) => {
						ty = quote!( #ty #t );
					},
					(PP::GotEqual|PP::GotDefaultVal,_,t) => {
						default = quote!( #default #t );
						phase = PP::GotDefaultVal;
					},
					_ => error(
						format!("予期しないトークン {} が含まれています",s),
						Some(&src)
					)
				}

				whole = quote!( #whole #tt );
			}

			match phase {
				PP::GotComma|PP::GotType|PP::GotDefaultVal|PP::GotSubValBody => {},
				PP::Beginning => { return None; },
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}

			let fv = match is_subtype {
				true => FV::Data(parse_one(default)),
				false => FV::Type {
					name: ty,
					default: match default.is_empty() {
						true => None,
						false => Some(default)
					}
				}
			};

			Some( F {
				attributes: attr,
				value: fv,
				src: whole.to_string()
			} )
		}

		/// 現在のパースの過程を表す
		enum ParsingPhase {
			Beginning, GotAttrHash, GotAttrBody,
			GotType, GotEqual, GotDefaultVal,
			GotSubValType, GotSubValHeader, GotSubValBody,
			GotComma
		}
		type PP = ParsingPhase;

		type F = EnumUnnamedField;
		type FV = FieldValue;
		type FS = EnumFields;
	}
	use enum_unnamed_fields_parser::*;

	/// 列挙体の名前ありフィールドのパーサ
	mod enum_named_fields_parser {
		use super::*;

		/// 列挙体の名前ありフィールドをパースする
		pub fn parse_enum_named_fields(ts:TS) -> FS {
			let mut fields:Vec<F> = vec![];
			let mut iter = ts.into_iter();

			while let Some(f) = parse_field(&mut iter) {
				fields.push(f);
			}

			FS::Named(fields)
		}

		/// それぞれのフィールドをパースする
		fn parse_field(iter:&mut impl TI) -> Option<F> {
			let src = TS::from_iter(iter.clone()).to_string();

			let mut phase = PP::Beginning;
			let mut attr:Vec<Attr> = vec![];
			let mut name:Option<Ident> = None;
			let mut is_subtype = false;
			let mut ty = TS::new();
			let mut type_generics_count = 0_u8;
			let mut default = TS::new();
			let mut whole = TS::new();

			loop {
				let tt = match iter.next() {
					Some(t) => t,
					None => { break }
				};
				let s = tt.to_string();

				match (&phase,&s[..],tt.clone()) {
					(PP::Beginning|PP::GotAttrBody|PP::GotComma,"#",_) => {
						phase = PP::GotAttrHash;
					},
					(PP::GotAttrHash,_,TT::Group(g)) => {
						attr.push( parse_attr(g.stream()) );
						phase = PP::GotAttrBody;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotComma,_,TT::Ident(i)) => {
						name = Some(i);
						phase = PP::GotName;
					},
					(PP::GotName,":",_) => {
						phase = PP::GotColon;
					},
					(PP::GotType,"<",t) => {
						type_generics_count += 1;
						ty = quote!( #ty #t );
					},
					(PP::GotType,">",t) => {
						type_generics_count -= 1;
						ty = quote!( #ty #t );
					},
					(PP::GotType,"=",t) => {
						if type_generics_count!=0 {
							ty = quote!( #ty #t );
						}
						else { phase = PP::GotEqual; }
					},
					(PP::GotType,",",t) => {
						if type_generics_count!=0 {
							ty = quote!( #ty #t );
						}
						else {
							phase = PP::GotComma;
							break;
						}
					},
					(PP::GotColon|PP::GotType,_,t) => {
						ty = quote!( #ty #t );
						phase = PP::GotType;
					},
					(PP::GotEqual,"struct"|"enum",t) => {
						is_subtype = true;
						default = quote!(#t);
						phase = PP::GotSubValType;
					},
					(PP::GotSubValHeader,_,TT::Group(g)) => {
						if matches!(g.delimiter(),Delimiter::Brace) {
							phase = PP::GotSubValBody;
						}
						default = quote!( #default #g );
					},
					(PP::GotSubValType|PP::GotSubValHeader,_,t) => {
						default = quote!( #default #t );
						phase = PP::GotSubValHeader;
					},
					(PP::GotSubValBody|PP::GotDefaultVal,",",_) => {
						phase = PP::GotComma;
						break;
					},
					(PP::GotEqual|PP::GotDefaultVal,_,t) => {
						default = quote!( #default #t );
						phase = PP::GotDefaultVal;
					},
					_ => error(
						format!("予期しないトークン {} が含まれています",s),
						Some(&src)
					)
				}

				whole = quote!( #whole #tt );
			}

			match phase {
				PP::GotComma|PP::GotType|PP::GotDefaultVal|PP::GotSubValBody => {},
				PP::Beginning => { return None; },
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}

			let fv = match is_subtype {
				true => FV::Data(parse_one(default)),
				false => FV::Type {
					name: ty,
					default: match default.is_empty() {
						true => None,
						false => Some(default)
					}
				}
			};

			Some( F {
				attributes: attr,
				name: name.unwrap(),
				value: fv,
				src: whole.to_string()
			} )
		}

		/// 現在のパースの過程を表す
		enum ParsingPhase {
			Beginning, GotAttrHash, GotAttrBody,
			GotName, GotColon, GotType,
			GotEqual, GotDefaultVal,
			GotSubValType, GotSubValHeader, GotSubValBody,
			GotComma
		}
		type PP = ParsingPhase;

		type F = EnumNamedField;
		type FV = FieldValue;
		type FS = EnumFields;
	}
	use enum_named_fields_parser::*;

	/// アトリビュートのパーサ
	mod attr_parser {
		use super::*;

		/// アトリビュートをパースする
		pub fn parse_attr(ts:TS) -> A {
			let mut iter = ts.clone().into_iter();
			let kind = match iter.next() {
				Some(TT::Ident(i)) => i.to_string(),
				t => error(
					format!("予期しないトークン {} が含まれています",t.into_token_stream()),
					Some(&ts.to_string())
				)
			};
			let mut a = match &kind[..] {
				"default" => A::Default,
				"pub_all" => A::PubAll,
				_ => A::Other(ts.clone())
			};
			let mut phase = PP::Beginning;

			for tt in iter {
				let s = tt.to_string();
				match (&phase,&kind[..],&a,&s[..],tt) {
					(PP::Beginning,"derive",A::Other(_),_,TT::Group(g)) => {
						if let Some(v) = parse_group(g.stream()) {
							a = A::Derive(v);
						}
						phase = PP::GotGroup;
					},
					(PP::Beginning,"allow",A::Other(_),_,TT::Group(g)) => {
						if let Some(v) = parse_group(g.stream()) {
							a = A::Allow(v);
						}
						phase = PP::GotGroup;
					},
					(PP::Beginning,"cfg",A::Other(_),_,TT::Group(g)) => {
						a = A::Cfg(g.stream());
						phase = PP::GotGroup;
					},
					(PP::Beginning,"doc",A::Other(_),"=",_) => {
						phase = PP::GotEqual;
					},
					(PP::GotEqual,"doc",A::Other(_),_,TT::Literal(l)) => {
						a = A::Doc(l);
						phase = PP::GotLiteral;
					},
					(PP::Beginning,_,A::Other(_),_,_) => {},
					_ => { a = A::Other(ts.clone()); }
				}
			}

			a
		}

		/// アトリビュートに含まれるカンマ区切りトークンのパースを試みる
		fn parse_group(ts:TS) -> Option<Vec<Ident>> {
			let mut items:Vec<Ident> = vec![];
			let mut phase = PP::Beginning;
			for tt in ts {
				let s = tt.to_string();
				match (phase,&s[..],tt) {
					(PP::Beginning|PP::GotComma,_,TT::Ident(i)) => {
						items.push(i);
						phase = PP::GotName;
					},
					(PP::GotName,",",_) => {
						phase = PP::GotComma;
					},
					_ => { return None; }
				}
			}
			Some(items)
		}

		/// 現在のパースの過程を表す
		enum ParsingPhase {
			Beginning,
			GotEqual, GotGroup, GotLiteral,
			GotName, GotComma
		}
		type PP = ParsingPhase;

		type A = Attr;
	}
	use attr_parser::*;

}
use parser::*;



/// 生成する前に適当にパースした情報を書き換えるモジュール
mod modification {
	use super::*;

	/// データのリストを受け取って書き換えるインターフェイス
	pub fn modify(root:&mut Root) {
		let datum = &mut root.datum;
		for d in datum.iter_mut() {
			d.modify();
		}
	}

	/// 書き換えるメソッド `modify` を与えるトレイト
	trait Modify {
		/// このオブジェクトを適当に書き換える。子要素の `modify` も再帰的にび出される。
		fn modify(&mut self);
	}

	impl Modify for Data {
		fn modify(&mut self) {
			match self {
				Self::Struct(s) => s.modify(),
				Self::Enum(e) => e.modify(),
				_ => {}
			}
		}
	}

	impl Modify for Struct {
		fn modify(&mut self) {
			self.check_pub_all();
			self.check_default();

			let Self {
				ref mut attributes,
				ref mut fields,
				ref mut enclosed,
				ref visibility,
				..
			} = self;

			let mut st = fields.collect_subtype();
			st.extend(enclosed.iter_mut());
			copy_attr_to_subtype(&*attributes,&mut st);

			for d in enclosed.iter_mut() {
				inherit_visibility(visibility, d);
			}

			remove_duplicate(attributes);

			fields.iter_mut()
			.for_each(|f| f.modify() );
			enclosed.iter_mut()
			.for_each(|d| d.modify() );
		}
	}

	impl Modify for StructField {
		fn modify(&mut self) {
			self.check_pub_all();
			self.check_default();

			let Self {
				ref mut attributes,
				ref mut value,
				ref visibility,
				..
			} = self;

			if let Some(d) = value.get_subtype() {
				inherit_visibility(visibility,d);
			}
			move_field_attrs_to_subtype(attributes,value);
			value.modify();
		}
	}

	impl Modify for Enum {
		fn modify(&mut self) {
			self.check_pub_all();
			self.check_default();

			let Self {
				ref mut variants,
				ref visibility,
				ref mut attributes,
				ref mut enclosed,
				..
			} = self;

			let mut st = variants.collect_subtype();
			st.extend(enclosed.iter_mut());

			copy_attr_to_subtype(&*attributes,&mut st);

			remove_duplicate(attributes);

			st.iter_mut()
			.for_each(|d| {
				inherit_visibility(visibility,*d);
			});

			variants.iter_mut()
			.for_each(|v| {
				v.modify();
			});
			enclosed.iter_mut()
			.for_each(|d| d.modify() );
		}
	}

	impl Modify for EnumVariant {
		fn modify(&mut self) {
			self.check_default();

			self.fields.modify();
		}
	}

	impl Modify for EnumFields {
		fn modify(&mut self) {
			match self {
				Self::Unit => {},
				Self::Unnamed(v) => {
					for f in v.iter_mut() {
						f.modify();
					}
				},
				Self::Named(v) => {
					for f in v.iter_mut() {
						f.modify();
					}
				}
			}
		}
	}

	impl Modify for EnumUnnamedField {
		fn modify(&mut self) {
			self.check_pub_all();
			self.check_default();

			let Self {
				ref mut attributes,
				ref mut value,
				..
			} = self;

			move_field_attrs_to_subtype(attributes,value);
			value.modify();
		}
	}

	impl Modify for EnumNamedField {
		fn modify(&mut self) {
			self.check_pub_all();
			self.check_default();

			let Self {
				ref mut attributes,
				ref mut value,
				..
			} = self;

			move_field_attrs_to_subtype(attributes,value);
			value.modify();
		}
	}

	impl Modify for FieldValue {
		fn modify(&mut self) {
			match self {
				Self::Type{..} => {},
				Self::Data(d) => { d.modify(); }
			}
		}
	}

	trait CollectSubType {
		/// このオブジェクトに含まれるサブ構造体/列挙体のリストを返す
		fn collect_subtype(&mut self) -> Vec<&mut Data>;
	}
	impl CollectSubType for Vec<StructField> {
		fn collect_subtype(&mut self) -> Vec<&mut Data> {
			self.iter_mut()
			.filter_map(|f| f.value.get_subtype() )
			.collect()
		}
	}
	impl CollectSubType for Vec<EnumVariant> {
		fn collect_subtype(&mut self) -> Vec<&mut Data> {
			self.iter_mut()
			.map( |v| v.fields.collect_subtype() )
			.flatten()
			.collect()
		}
	}
	impl CollectSubType for EnumFields {
		fn collect_subtype(&mut self) -> Vec<&mut Data> {
			match self {
				Self::Unit{..} => vec![],
				Self::Unnamed(v) => {
					v.iter_mut()
					.filter_map(|f| f.value.get_subtype() )
					.collect()
				},
				Self::Named(v) => {
					v.iter_mut()
					.filter_map(|f| f.value.get_subtype() )
					.collect()
				}
			}
		}
	}
	impl FieldValue {
		/// フィールドの値が構造体/列挙体を包含するものであれば、それを返す
		fn get_subtype(&mut self) -> Option<&mut Data> {
			match self {
				Self::Type{..} => None,
				Self::Data(d) => Some(d)
			}
		}
	}

	trait PubAll {
		/// このオブジェクトに `#[pub_all]` アトリビュートが含まれているか確認し、含まれていたら `pub_all()` を実行する
		fn check_pub_all(&mut self);
		/// このオブジェクトや含まれる全てのサブ構造体/列挙体、フィールドに `pub` を付す
		fn pub_all(&mut self);
	}
	impl PubAll for Data {
		fn check_pub_all(&mut self) {
			match self {
				Self::Struct(s) => s.check_pub_all(),
				Self::Enum(e) => e.check_pub_all(),
				_ => {}
			}
		}
		fn pub_all(&mut self) {
			match self {
				Self::Struct(s) => s.pub_all(),
				Self::Enum(e) => e.pub_all(),
				_ => {}
			}
		}
	}
	impl PubAll for Struct {
		fn check_pub_all(&mut self) {
			if let Some(_) = check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::PubAll)
			) { self.pub_all(); }
		}
		fn pub_all(&mut self) {
			self.visibility = quote!(pub);
			self.fields.iter_mut()
			.for_each(|f| {
				f.visibility = quote!(pub);
			});
			self.fields.collect_subtype()
			.iter_mut()
			.for_each(|d| d.pub_all() );
		}
	}
	impl PubAll for StructField {
		fn check_pub_all(&mut self) {
			if let Some(_) = check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::PubAll)
			) { self.pub_all(); }
		}
		fn pub_all(&mut self) {
			self.visibility = quote!(pub);
			self.value.get_subtype()
			.map(|d| d.pub_all() );
		}
	}
	impl PubAll for Enum {
		fn check_pub_all(&mut self) {
			if let Some(_) = check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::PubAll)
			) { self.pub_all(); }
		}
		fn pub_all(&mut self) {
			self.variants.collect_subtype()
			.iter_mut()
			.for_each(|d| d.pub_all() );
		}
	}
	impl PubAll for EnumUnnamedField {
		fn check_pub_all(&mut self) {
			if let Some(_) = check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::PubAll)
			) { self.pub_all(); }
		}
		fn pub_all(&mut self) {
			if let Some(d) = self.value.get_subtype() {
				d.pub_all();
			}
		}
	}
	impl PubAll for EnumNamedField {
		fn check_pub_all(&mut self) {
			if let Some(_) = check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::PubAll)
			) { self.pub_all(); }
		}
		fn pub_all(&mut self) {
			if let Some(d) = self.value.get_subtype() {
				d.pub_all();
			}
		}
	}

	trait SetDefault {
		/// このオブジェクトに `#[default]` アトリビュートが含まれているか確認し、含まれていたら `set_default()` を実行する
		fn check_default(&mut self);
		/// デフォルト値が定義されていない場合、デフォルト値を定義する
		fn set_default(&mut self);
	}
	impl SetDefault for Data {
		fn check_default(&mut self) {
			match self {
				Self::Struct(s) => s.check_default(),
				Self::Enum(e) => e.check_default(),
				_ => {}
			}
		}
		fn set_default(&mut self) {
			match self {
				Self::Struct(s) => s.set_default(),
				Self::Enum(e) => e.set_default(),
				_ => {}
			}
		}
	}
	impl SetDefault for Struct {
		fn check_default(&mut self) {
			if check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::Default)
			).is_some() { self.set_default(); }
		}
		fn set_default(&mut self) {
			for f in self.fields.iter_mut() {
				f.set_default();
			}
		}
	}
	impl SetDefault for StructField {
		fn check_default(&mut self) {
			if check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::Default)
			).is_some() { self.set_default(); }
		}
		fn set_default(&mut self) {
			self.value.set_default();
		}
	}
	impl SetDefault for Enum {
		fn check_default(&mut self) {
			if check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::Default)
			).is_some() { self.set_default(); }
		}
		fn set_default(&mut self) {
			type B = QuadBool;
			if !matches!(self.has_default(),B::False) { return; }
			self.variants[0].set_default();
		}
	}
	impl SetDefault for EnumVariant {
		fn check_default(&mut self) {
			let contains = check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::Default)
			);
			if contains.is_some()||self.is_default { self.set_default(); }
		}
		fn set_default(&mut self) {
			self.is_default = true;
			type F = EnumFields;
			match &mut self.fields {
				F::Unit => {},
				F::Named(v) => {
					for f in v.iter_mut() {
						f.set_default();
					}
				},
				F::Unnamed(v) => {
					for f in v.iter_mut() {
						f.set_default();
					}
				}
			}
		}
	}
	impl SetDefault for EnumUnnamedField {
		fn check_default(&mut self) {
			if check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::Default)
			).is_some() { self.set_default(); }
		}
		fn set_default(&mut self) {
			self.value.set_default()
		}
	}
	impl SetDefault for EnumNamedField {
		fn check_default(&mut self) {
			if check_attr_flag(
				&mut self.attributes,
				|a| matches!(a,Attr::Default)
			).is_some() { self.set_default(); }
		}
		fn set_default(&mut self) {
			self.value.set_default()
		}
	}
	impl FieldValue {
		/// デフォルト値が定義されていない場合、デフォルト値を定義する
		fn set_default(&mut self) {
			match self {
				Self::Data(d) => { d.set_default(); },
				Self::Type { default: d @ None, .. } => {
					*d = Some( quote!( std::default::Default::default() ) );
				},
				Self::Type { default: Some(_), .. } => {}
			}

		}
	}

	/// 構造体のフィールドに与えられた可視性を、値となるサブ構造体/列挙体にコピーする
	fn inherit_visibility(vis:&TS,d:&mut Data) {
		let vis_child = match d {
			Data::Struct(s) => &mut s.visibility,
			Data::Enum(e) => &mut e.visibility,
			Data::Type(t) => &mut t.visibility,
			Data::Trait(t) => &mut t.visibility,
			_ => { unreachable!(); }
		};
		*vis_child = vis.clone();
	}

	/// 構造体のフィールドに付されたアトリビュートを、値となるサブ構造体/列挙体に移動或いはコピーする
	fn move_field_attrs_to_subtype(pal:&mut Vec<Attr>,value:&mut FieldValue) {
		let mut tmp = pal.iter()
		.filter_map(|a| {
			match a {
				Attr::Doc(_)|Attr::Cfg(_) => Some(a.clone()),
				_ => None,
			}
		})
		.collect::<Vec<_>>();
		swap(pal,&mut tmp);
		let cal = match value.get_subtype() {
			None => { return },
			Some(Data::Struct(s)) => &mut s.attributes,
			Some(Data::Enum(e)) => &mut e.attributes,
			_ => { unreachable!(); }
		};
		cal.extend(tmp);
	}

	/// 構造体/列挙体に付されたアトリビュートの一部を、サブ構造体/列挙体にコピーする
	fn copy_attr_to_subtype(pal:&Vec<Attr>,dl:&mut Vec<&mut Data>) {
		if dl.len()==0 { return; }

		let mut copied_derive:Vec<Ident> = vec![];
		let mut copied_allow:Vec<Ident> = vec![];
		let mut copied_cfg:Vec<TS> = vec![];
		for a in pal.iter() {
			match a {
				Attr::Allow(v) => {
					copied_allow.extend(v.clone());
				},
				Attr::Cfg(c) => {
					copied_cfg.push(c.clone());
				}
				Attr::Derive(v) => {
					let nv = v.iter()
					.filter_map(|i| {
						match &i.to_string()[..] {
							"Clone"|"Copy"|"Debug"|"PartialEq"|"Eq"|"PartialOrd"|"Ord"|"Serialize"|"Deserialize" => Some(i.clone()),
							_ => None
						}
					})
					.collect::<Vec<_>>();
					copied_derive.extend(nv);
				},
				_ => {}
			}
		}

		for d in dl.iter_mut() {
			let (ca,will_copy_derive) = match d {
				Data::Struct(s) => (&mut s.attributes,true),
				Data::Enum(e) => (&mut e.attributes,true),
				Data::Type(t) => (&mut t.attributes,false),
				Data::Trait(t) => (&mut t.attributes,false),
				Data::Debug => { unreachable!(); }
			};

			if will_copy_derive && copied_derive.len()>0 {
				ca.push(
					Attr::Derive(copied_derive.clone())
				);
			}
			if copied_allow.len()>0 {
				ca.push(
					Attr::Allow(copied_allow.clone())
				);
			}
			for c in copied_cfg.iter() {
				ca.push(
					Attr::Cfg(c.clone())
				);
			}
		}
	}

	/// アトリビュートで重複した項目があれば、1つに絞る
	fn remove_duplicate(attr:&mut Vec<Attr>) {
		remove_duplicate_impl(
			attr,
			|a| matches!(a,Attr::Derive(_)),
			|a| {
				match a {
					Attr::Derive(v) => Ok(v),
					a => Err(a)
				}
			},
			|v| Attr::Derive(v)
		);
		remove_duplicate_impl(
			attr,
			|a| matches!(a,Attr::Allow(_)),
			|a| {
				match a {
					Attr::Allow(v) => Ok(v),
					a => Err(a)
				}
			},
			|v| Attr::Allow(v)
		);
	}

	/// `derive` と `allow` のための `remove_duplicate` の実装 (共通部分をここにまとめた)
	fn remove_duplicate_impl(
		attr:&mut Vec<Attr>,
		is_matched:impl Fn(&&Attr) -> bool,
		to_list:impl Fn(Attr) -> Result<Vec<Ident>,Attr>,
		to_attr:impl Fn(Vec<Ident>) -> Attr
	) {
		let num = attr.iter()
		.filter(is_matched)
		.count();
		if num<2 { return; }

		let mut items:Vec<Ident> = vec![];

		let mut attr_tmp:Vec<Attr> = vec![];
		swap(attr,&mut attr_tmp);
		*attr = attr_tmp.into_iter()
		.filter_map(|a| {
			match to_list(a) {
				Ok(mut v) => {
					items.append(&mut v);
					None
				},
				Err(a) => Some(a)
			}
		})
		.collect::<Vec<_>>();

		items.sort_by(|i1,i2| {
			i1.to_string().cmp(&i2.to_string())
		});
		items.dedup_by(|i1,i2| i1.to_string()==i2.to_string() );

		if items.len()>0 {
			let a = to_attr(items);
			if attr.is_empty() { attr.push(a); }
			else {
				let mut attr_tmp = vec![a];
				attr_tmp.append(attr);
				swap(attr,&mut attr_tmp);
			}
		}
	}

	/// 付されたアトリビュートのうち特定の条件を満たすものがあれば返し、アトリビュートのリストから取り除く
	fn check_attr_flag(attr:&mut Vec<Attr>,predicate:impl FnMut(&Attr) -> bool) -> Option<Attr> {
		attr.iter()
		.position(predicate)
		.map(|i| attr.remove(i) )
	}

}
use modification::*;



/// 実際にオブジェクトを生成するモジュール
mod compose {
	use super::*;

	/// オブジェクト生成のエントリポイント
	pub fn compose(root:Root) -> TS {
		let Root { debug, datum, src } = root;

		let mut ts = TS::new();

		for d in datum {
			let _ = d.compose(&mut ts);
		}

		if debug {
			let out = ts.to_string();
			let output = format!(
				"The macro code\ncompose_struct! {{\n\n{}\n\n}}\nwill be converted to\n\n{}\n\n\n",
				src, out
			);
			eprintln!("{}",output);
		}

		ts
	}

	/// 各々のオブジェクト生成を行うトレイト
	trait Compose {
		/// オブジェクトに対応するパーツを生成
		fn compose(&self,global:&mut TS) -> TS;
		/// オブジェクトがデフォルト値を生成する際に担うパーツを生成
		fn compose_default(&self,global:&mut TS) -> TS;
	}

	impl Compose for Data {
		fn compose(&self,global:&mut TS) -> TS {
			match self {
				Self::Struct(s) => s.compose(global),
				Self::Enum(e) => e.compose(global),
				Self::Type(t) => t.compose(global),
				Self::Trait(t) => t.compose(global),
				Self::Debug => { unreachable!(); }
			}
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			match self {
				Self::Struct(s) => s.compose_default(global),
				Self::Enum(e) => e.compose_default(global),
				Self::Type(t) => t.compose_default(global),
				Self::Trait(t) => t.compose_default(global),
				Self::Debug => { unreachable!(); }
			}
		}
	}

	impl Compose for Struct {
		fn compose(&self,global:&mut TS) -> TS {
			let n = &self.name;
			let g = match self.generics.is_empty() {
				true => TS::new(),
				false => {
					let g = &self.generics;
					quote!( <#g> )
				}
			};

			{
				let a = self.attributes.compose(global);
				let v = &self.visibility;
				let w = add_where(&self.where_condition.clone());
				let mut body = TS::new();
				for f in self.fields.iter() {
					let ft = f.compose(global);
					body = quote!( #body #ft, );
				}
				let this = quote!(
					#a #v struct #n #g #w { #body }
				);
				*global = quote!( #global #this );
			}

			match self.has_default() {
				QuadBool::NotAllowed => error(
					"一部の値にはデフォルト値が指定されていますが、他の値には指定されていません",
					Some(&self.src)
				),
				QuadBool::TrueOptional => {
					let a = self.attributes.compose_default(global);
					let mut body = TS::new();
					for f in self.fields.iter() {
						let ft = f.compose_default(global);
						body = quote!( #body #ft, );
					}
					let this = quote!(
						#a impl #g std::default::Default for #n #g {
							fn default() -> Self {
								Self { #body }
							}
						}
					);
					*global = quote!( #global #this );
				},
				_ => {}
			}

			for d in self.enclosed.iter() {
				d.compose(global);
			}

			quote!( #n #g )
		}
		fn compose_default(&self,_:&mut TS) -> TS {
			quote!( Default::default() )
		}
	}

	impl Compose for StructField {
		fn compose(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose(global);
			let v = &self.visibility;
			let n = &self.name;
			let t = self.value.compose(global);
			quote!( #a #v #n: #t )
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose_default(global);
			let n = &self.name;
			let v = self.value.compose_default(global);
			quote!( #a #n: #v )
		}
	}

	impl Compose for Enum {
		fn compose(&self,global:&mut TS) -> TS {
			let n = &self.name;
			let g = match self.generics.is_empty() {
				true => TS::new(),
				false => {
					let g = &self.generics;
					quote!( <#g> )
				}
			};

			{
				let a = self.attributes.compose(global);
				let v = &self.visibility;
				let w = add_where(&self.where_condition);
				let mut body = TS::new();
				for var in self.variants.iter() {
					let v = var.compose(global);
					body = quote!( #body #v, );
				}
				let this = quote!(
					#a #v enum #n #g #w { #body }
				);
				*global = quote!( #global #this );
			}

			if let Some(var_default) = self.variants.iter().find_map(|v| {
				match v.has_default() {
					QuadBool::TrueRequired => Some(v.compose_default(global)),
					QuadBool::NotAllowed => error(
						"デフォルト値が複数指定されているか、サブフィールドのデフォルト値の指定の仕方が正しくない可能性があります",
						Some(&self.src)
					),
					_ => None
				}
			}) {
				let a = self.attributes.compose_default(global);
				let w = add_where(&self.where_condition);
				let this = quote!(
					#a impl #g std::default::Default for #n #g #w {
						fn default() -> Self {
							Self::#var_default
						}
					}
				);
				*global = quote!( #global #this );
			}

			for d in self.enclosed.iter() {
				d.compose(global);
			}

			quote!( #n #g )
		}
		fn compose_default(&self,_:&mut TS) -> TS {
			quote!( Default::default() )
		}
	}

	impl Compose for EnumVariant {
		fn compose(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose(global);
			let n = &self.name;
			match &self.fields {
				EnumFields::Unit => quote!( #a #n ),
				EnumFields::Unnamed(v) => {
					let mut fst = TS::new();
					for f in v.iter() {
						let ft = f.compose(global);
						fst = quote!( #fst #ft, );
					}
					quote!( #a #n ( #fst ) )
				},
				EnumFields::Named(v) => {
					let mut fst = TS::new();
					for f in v.iter() {
						let ft = f.compose(global);
						fst = quote!( #fst #ft, );
					}
					quote!( #a #n { #fst } )
				}
			}
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose_default(global);
			let n = &self.name;
			match &self.fields {
				EnumFields::Unit => quote!( #a #n ),
				EnumFields::Unnamed(v) => {
					let mut fst = TS::new();
					for f in v.iter() {
						let ft = f.compose_default(global);
						fst = quote!( #fst #ft, );
					}
					quote!( #a #n ( #fst ) )
				},
				EnumFields::Named(v) => {
					let mut fst = TS::new();
					for f in v.iter() {
						let ft = f.compose_default(global);
						fst = quote!( #fst #ft, );
					}
					quote!( #a #n { #fst } )
				}
			}

		}
	}

	impl Compose for EnumUnnamedField {
		fn compose(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose(global);
			let v = self.value.compose(global);
			quote!( #a #v )
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose_default(global);
			let v = self.value.compose_default(global);
			quote!( #a #v )
		}
	}

	impl Compose for EnumNamedField {
		fn compose(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose(global);
			let n = &self.name;
			let v = self.value.compose(global);
			quote!( #a #n: #v )
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose_default(global);
			let n = &self.name;
			let v = self.value.compose_default(global);
			quote!( #a #n: #v )
		}
	}

	impl Compose for FieldValue {
		fn compose(&self,global:&mut TS) -> TS {
			match self {
				Self::Type {name,..} => name.clone(),
				Self::Data(d) => d.compose(global)
			}
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			match self {
				Self::Type {default:Some(d),..} => d.clone(),
				Self::Type {default:None,..} => {
					quote!( std::default::Default::default() )
				},
				Self::Data(d) => d.compose_default(global)
			}
		}
	}

	impl Compose for TypeAlias {
		fn compose(&self,global:&mut TS) -> TS {
			let Self {
				ref name,
				ref artifact,
				ref attributes,
				ref visibility,
				..
			} = self;
			let attr = attributes.compose(global);
			let this = quote!( #attr #visibility type #name = #artifact; );
			*global = quote!( #global #this );
			TS::new()
		}
		fn compose_default(&self,_global:&mut TS) -> TS { unreachable!(); }
	}

	impl Compose for TraitAlias {
		fn compose(&self,global:&mut TS) -> TS {
			let Self {
				ref name,
				ref generics,
				ref artifact,
				ref attributes,
				ref visibility,
				..
			} = self;
			let attr = attributes.compose(global);
			let t = Ident::new(&format!("GenericTypeFor{}",name),Span::call_site());
			let g = match generics.is_empty() {
				true => t.to_token_stream(),
				false => quote!( #t,#generics )
			};
			let this = quote!(
				#attr #visibility trait #name: #artifact {}
				impl<#g> #name for #t where #t: #artifact {}
			);
			*global = quote!( #global #this );
			TS::new()
		}
		fn compose_default(&self,_global:&mut TS) -> TS { unreachable!(); }
	}

	impl Compose for Vec<Attr> {
		fn compose(&self,_:&mut TS) -> TS {
			let mut ts = TS::new();
			for a in self.iter() {
				let at = a.compose();
				ts = quote!( #ts #at );
			}
			ts
		}
		fn compose_default(&self,_:&mut TS) -> TS {
			let mut ts = TS::new();
			for a in self.iter() {
				if !matches!(a,Attr::Cfg(_)) { continue; }
				let at = a.compose();
				ts = quote!( #ts #at );
			}
			ts
		}
	}
	impl Attr {
		/// アトリビュートを生成
		fn compose(&self) -> TS {
			match self {
				Self::Derive(v) => {
					let items = v.comma_join();
					quote!( #[derive(#items)] )
				},
				Self::Allow(v) => {
					let items = v.comma_join();
					quote!( #[allow(#items)] )
				},
				Self::Cfg(ts) => {
					quote!( #[cfg(#ts)] )
				},
				Self::Doc(doc) => {
					quote!( #[doc=#doc] )
				},
				Self::Default|Self::PubAll => TS::new(),
				Self::Other(ts) => {
					quote!( #[#ts] )
				}
			}
		}
	}

	/// `Attr` に `Display` トレイトを付けるモジュール
	mod attr_display {
		use super::*;
		use std::fmt::{Display,Debug,Formatter,Result};

		impl Display for Attr {
			fn fmt(&self, f: &mut Formatter<'_>) -> Result {
				write!(f,"{}",self.compose())
			}
		}
		impl Debug for Attr {
			fn fmt(&self, f: &mut Formatter<'_>) -> Result {
				write!(f,"{}",self.compose())
			}
		}
	}
	pub use attr_display::*;

	/// `where` 節を生成
	fn add_where(w:&TS) -> TS {
		if w.is_empty() { TS::new() }
		else { quote!( where #w ) }
	}

}
use compose::*;



/// デフォルト値を持つべきか構造を再帰的に探索して決定するモジュール
mod has_default {
	use super::*;

	pub trait HasDefault {
		/// このオブジェクトの情報からデフォルト値を構成すべきか判定する
		fn has_default(&self) -> B;
	}

	impl HasDefault for Data {
		fn has_default(&self) -> B {
			match self {
				Self::Struct(s) => s.has_default(),
				Self::Enum(e) => e.has_default(),
				_ => B::TrueOptional
			}
		}
	}

	impl HasDefault for Struct {
		fn has_default(&self) -> B {
			self.fields.iter()
			.map(|f| f.value.has_default() )
			.merge_struct()
		}
	}

	impl HasDefault for Enum {
		fn has_default(&self) -> B {
			self.variants.iter()
			.map(|v| v.has_default() )
			.merge_enum()
		}
	}

	impl HasDefault for EnumVariant {
		fn has_default(&self) -> B {
			type F = EnumFields;
			let bool_fields = match &self.fields {
				F::Unit => {
					return match self.is_default {
						true => B::TrueRequired,
						false => B::False
					};
				},
				F::Unnamed(v) => {
					v.iter()
					.map(|f| f.value.has_default() )
					.merge_enum_fields()
				},
				F::Named(v) => {
					v.iter()
					.map(|f| f.value.has_default() )
					.merge_enum_fields()
				}
			};
			match (self.is_default,bool_fields) {
				(true,B::TrueRequired|B::TrueOptional) => B::TrueRequired,
				(true,B::False|B::NotAllowed) => B::NotAllowed,
				(false,b) => b,
			}
		}
	}

	impl HasDefault for FieldValue {
		fn has_default(&self) -> B {
			match self {
				Self::Type{default: Some(_),..} => B::TrueRequired,
				Self::Type{default: None,..} => B::False,
				Self::Data(d) => d.has_default()
			}
		}
	}

	#[derive(Clone,Copy)]
	/// `has_default` で用いられるブール値の拡張
	pub enum QuadBool {
		/// 真。この値の場合は必ずデフォルト値を構成しなければならない
		TrueRequired,
		/// 真。この値が定義されていても必ずしもデフォルト値を構成する必要はない
		TrueOptional,
		/// 偽。この値はデフォルト値を定義していないことを表す
		False,
		/// 判別不能。これは列挙体において複数のバリアントがデフォルト値に指定されている場合に該当する。
		NotAllowed
	}
	type B = QuadBool;

	/// `QuadBool` の値を縮約するモジュール
	trait Merge {
		/// 構造体の場合のルールに従って `QuadBool` を縮約する
		fn merge_struct(self) -> B;
		/// 列挙体の場合のルールに従って `QuadBool` を縮約する
		fn merge_enum(self) -> B;
		/// 列挙体フィールドの場合のルールに従って `QuadBool` を縮約する
		fn merge_enum_fields(self) -> B;
	}
	impl<I> Merge for I where I: Iterator<Item=B> {
		fn merge_struct(self) -> B {
			match self.merge_enum_fields() {
				B::NotAllowed => B::NotAllowed,
				B::TrueRequired|B::TrueOptional => B::TrueOptional,
				B::False => B::False
			}
		}
		fn merge_enum(self) -> B {
			let raw = self.reduce(|b1,b2| {
				match (b1,b2) {
					(B::NotAllowed,_)|(_,B::NotAllowed) => B::NotAllowed,
					(B::TrueRequired,B::TrueRequired) => B::NotAllowed,
					(B::TrueOptional,B::TrueOptional) => B::TrueOptional,
					(B::TrueRequired,_)|(_,B::TrueRequired) => B::TrueRequired,
					(B::False,_)|(_,B::False) => B::False
				}
			})
			.unwrap_or(B::NotAllowed);
			match raw {
				B::NotAllowed => B::NotAllowed,
				B::TrueRequired|B::TrueOptional => B::TrueOptional,
				B::False => B::False
			}
		}
		fn merge_enum_fields(self) -> B {
			self.reduce(|b1,b2| {
				match (b1,b2) {
					(B::NotAllowed,B::NotAllowed) => B::False,
					(B::NotAllowed,b)|(b,B::NotAllowed) => b,
					(B::TrueOptional,B::TrueOptional) => B::TrueOptional,
					(B::TrueRequired,_)|(_,B::TrueRequired) => B::TrueRequired,
					(B::False,_)|(_,B::False) => B::False
				}
			})
			.unwrap_or(B::False)
		}
	}

}
use has_default::*;
