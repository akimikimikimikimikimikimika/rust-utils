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
		//! 	pub trait AnyStr = std::convert::AsRef<str> + std::fmt::Display;
		//! 	/// クローン可能な `u8` 型イテレータ
		//! 	trait IntIter = Iterator<Item=u8> + Clone;
		//! 	/// `Iterator` に変換可能な型
		//! 	trait II<T> = IntoIterator<Item=T> where T: ?Sized;
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
		/// 構造体のフィールド
		pub fields: Fields,
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
		pub fields: Fields,
		/// この要素が列挙体のデフォルト値になっているか
		pub is_default: bool,
		/// 元のソースコード
		pub src: String
	}

	/// フィールドリスト (ある場合、ない場合の双方) を表す
	pub enum Fields {
		/// フィールドがない (単位要素) の場合
		Unit,
		/// フィールド名のないフィールドリスト
		Unnamed(UnnamedFields),
		/// フィールド名のあるフィールドリスト
		Named(NamedFields)
	}
	/// `Fields` にカプセル化されるフィールド型
	pub struct CapsuledFields<F> {
		/// フィールドのリスト
		pub fields: Vec<F>,
		/// 内包するデータ型
		pub enclosed: Vec<Data>
	}
	pub type UnnamedFields = CapsuledFields<UnnamedField>;
	pub type NamedFields = CapsuledFields<NamedField>;

	/// フィールド名のないフィールドを表す
	pub struct UnnamedField {
		/// フィールドに付されたアトリビュートのリスト
		pub attributes: Vec<Attr>,
		/// `pub` などの可視性 (フィールドにアクセス可能な範囲) の情報
		pub visibility: TS,
		/// フィールドの値
		pub value: FieldValue,
		/// 元のソースコード
		pub src: String
	}

	/// フィールド名のあるフィールドを表す
	pub struct NamedField {
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
		Data {
			/// データ型が明示的に与えられた場合は、型の種類を与える (省略した場合は構造体/列挙体の型名を使う)
			ty: Option<TS>,
			/// 別の構造体や列挙体の定義
			data: Data
		}
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
		/// `where` によるジェネリクスの拘束条件
		pub where_condition: TS,
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
		convert::AsRef,
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
		AnyStr { AsRef<str> + Display }
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

	/// 型 `T` からデータ型をパースするトレイト
	trait ParseFrom<S,R> {
		fn parse_from(src:S) -> R;
	}

	/// `impl ParseFrom for Data` で構造体や列挙体のヘッダーをパースした結果。それぞれ `impl ParseFrom for Struct` や `impl ParseFrom for Enum` に渡すことでさらに `body` の内容をパースして `Struct` や `Enum` を構成する。
	struct ParsingResult {
		/// データ型の種類
		pub kind: ParsedKind,
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
		/// `{ ... }` や `( ... )` の中身
		pub body: TS,
		/// 元のソースコード
		pub src: String
	}

	#[derive(Clone,Copy)]
	/// `impl ParseFrom for Data` でパースした結果として、そのデータ型の種別を表す。
	enum ParsedKind {
		/// フィールドのない構造体
		StructUnit,
		/// 名前なしフィールドのある構造体
		StructUnnamed,
		/// 名前ありフィールドのある構造体
		StructNamed,
		/// 列挙体
		Enum,
		/// 型エイリアス
		TypeAlias,
		/// トレイトエイリアス
		TraitAlias,
		/// デバッグフラグ
		Debug,
		/// 定まっていない
		Unknown
	}

	/// グループに含まれる項目を返す型。通常のバリアント/フィールド、表記上内包する `Data` 型、内側に定義されたアトリビュートを返せる
	enum GroupItem<I> {
		/// バリアント/フィールドなど、通常の値の場合
		Item(I),
		/// 内包する `Data` 型の場合
		Data(Data),
		/// 内側で定義されるアトリビュート
		InnerAttr(Attr),
		/// 末尾でいづれでもない場合
		None
	}

	/// フィールドと合わせてアトリビュートを受け取る
	struct FieldsWithAttr<F> {
		/// フィールド
		fields: F,
		/// アトリビュート
		attrs: Vec<Attr>
	}

	#[inline]
	/// 生の入力データをパースする
	pub fn parse(ts:TS) -> Root {
		Root::parse_from(ts)
	}

	// 入力値を全てパース
	impl ParseFrom<TS,Self> for Root {
		fn parse_from(ts:TS) -> Self {
			let src = ts.to_string();

			let mut datum:Vec<Data> = vec![];
			let mut debug = false;
			let mut iter = ts.into_iter().peekable();

			type OD = Option<Data>;
			while let Some(d) = OD::parse_from(&mut iter) {
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
	}

	// `Data` 型の値を正確に1つだけパース (複数あるとエラーになる)
	impl ParseFrom<TS,Self> for Data {
		fn parse_from(ts:TS) -> Self {
			type OD = Option<Data>;

			let mut iter = ts.clone().into_iter().peekable();
			let first = OD::parse_from(&mut iter);
			let second = OD::parse_from(&mut iter);

			match (first,second) {
				(Some(d),None) => d,
				(None,None) => {
					error(
						"データが見つかりませんでした",
						Some(&ts.to_string())
					)
				},
				(Some(_),Some(_)) => {
					error(
						"複数のデータを受け取りました",
						Some(&ts.to_string())
					)
				},
				(None,Some(_)) => { unreachable!(); }
			}
		}
	}

	// `Data` 型の1つをパース
	impl<I: TI> ParseFrom<&mut PI<I>,Self> for Option<Data> {
		// 外の部分だけパースし、 `{ ... }` の内部は構造体/列挙体のパーサーにそれぞれ渡す。
		fn parse_from(iter:&mut PI<I>) -> Self {
			let src = TS::from_iter(iter.clone()).to_string();

			/// 現在のパースの過程を表す型
			enum ParsingPhase {
				Beginning, GotAttrHash, GotAttrBody,
				GotPub, GotVisibility,
				GotType, GotName,
				GotGenericsBegin, GotGenerics, GotGenericsEnd,
				GotWhere, GotWhereItem, GotBody,
				GotEqual, GotArtifact, GotSemicolon
			}
			type PP = ParsingPhase;
			type K = ParsedKind;

			let mut phase = PP::Beginning;
			let mut attr:Vec<Attr> = vec![];
			let mut vis = TS::new();
			let mut kind = K::Unknown;
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

				match (&phase,&s[..],tt.clone(),&kind) {
					(PP::Beginning,"debug",_,K::Unknown) => {
						phase = PP::GotType;
						kind = K::Debug;
						if iter.peek().map_or(
							false,
							|t| t.to_string()==";"
						) {
							let _ = iter.next();
						}
						break
					},
					(PP::Beginning|PP::GotAttrBody,"#",_,K::Unknown) => {
						phase = PP::GotAttrHash;
					},
					(PP::GotAttrHash,_,TT::Group(g),K::Unknown) => {
						attr.push( Attr::parse_from(g.stream()) );
						phase = PP::GotAttrBody;
					}
					(PP::Beginning|PP::GotAttrBody,"pub",_,K::Unknown) => {
						vis = quote!(pub);
						phase = PP::GotPub;
					},
					(PP::GotPub,_,TT::Group(g),K::Unknown) => {
						match g.delimiter() {
							Delimiter::Parenthesis => {
								let t = TT::Group(g);
								vis = quote!( #vis #t );
								phase = PP::GotVisibility;
							},
							_ => error(
								"予期しない括弧にマッチしました",
								Some(&src)
							)
						}
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotPub|PP::GotVisibility,"struct",_,K::Unknown) => {
						kind = K::StructUnit;
						phase = PP::GotType;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotPub|PP::GotVisibility,"enum",_,K::Unknown) => {
						kind = K::Enum;
						phase = PP::GotType;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotPub|PP::GotVisibility,"type",_,K::Unknown) => {
						kind = K::TypeAlias;
						phase = PP::GotType;
					},
					(PP::Beginning|PP::GotAttrBody|PP::GotPub|PP::GotVisibility,"trait",_,K::Unknown) => {
						kind = K::TraitAlias;
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
					(PP::GotName|PP::GotGenericsEnd,"where",_,K::StructUnit|K::Enum) => {
						if generics_enclosure_count!=0 {
							error(
								format!("予期しないトークン {} が含まれています",s),
								Some(&src)
							);
						}
						phase = PP::GotWhere;
					},
					(PP::GotName|PP::GotGenericsEnd,_,TT::Group(g),K::Enum) => {
						if matches!(g.delimiter(),Delimiter::Brace) {
							body = g.stream();
							phase = PP::GotBody;
							break;
						}
						else {
							error(
								"予期しない括弧にマッチしました",
								Some(&src)
							);
						}
					},
					(PP::GotName|PP::GotGenericsEnd,_,TT::Group(g),K::StructUnit) => {
						match g.delimiter() {
							Delimiter::Brace => {
								kind = K::StructNamed;
								body = g.stream();
								phase = PP::GotBody;
								break;
							},
							Delimiter::Parenthesis => {
								kind = K::StructUnnamed;
								body = g.stream();
								phase = PP::GotBody;
							},
							_ => {
								error(
									"予期しない括弧にマッチしました",
									Some(&src)
								);
							}
						}
					},
					(PP::GotName|PP::GotGenericsEnd,";",_,K::StructUnit) => {
						phase = PP::GotSemicolon;
						break;
					},
					(PP::GotWhereItem,_,TT::Group(g),K::Enum) => {
						if generics_enclosure_count==0 && matches!(g.delimiter(),Delimiter::Brace) {
							body = g.stream();
							phase = PP::GotBody;
							break;
						}
						else {
							wh = quote!( #wh #g );
						}
					},
					(PP::GotWhereItem,_,TT::Group(g),K::StructUnit) => {
						match (generics_enclosure_count,g.delimiter()) {
							(0,Delimiter::Brace) => {
								kind = K::StructNamed;
								phase = PP::GotBody;
								break;
							},
							(0,Delimiter::Parenthesis) => {
								kind = K::StructUnnamed;
								phase = PP::GotBody;
							},
							_ => {
								wh = quote!( #wh #g );
							}
						}
					},
					(PP::GotBody,";",_,K::StructUnnamed) => {
						phase = PP::GotSemicolon;
						break;
					},
					(PP::GotName|PP::GotGenericsEnd,"=",_,K::TypeAlias|K::TraitAlias) => {
						phase = PP::GotEqual;
					},
					(PP::GotArtifact|PP::GotWhereItem,";",t,K::TypeAlias|K::TraitAlias) => {
						if generics_enclosure_count==0 {
							phase = PP::GotSemicolon;
							break;
						}
						else {
							wh = quote!( #wh #t );
						}
					},
					(PP::GotArtifact,"where",_,K::TraitAlias) => {
						phase = PP::GotWhere;
					},
					(PP::GotEqual|PP::GotArtifact,_,t,K::TypeAlias|K::TraitAlias) => {
						body = quote!( #body #t );
						phase = PP::GotArtifact;
					},
					(PP::GotWhere|PP::GotWhereItem,"<",t,_) => {
						generics_enclosure_count += 1;
						wh = quote!( #wh #t );
					},
					(PP::GotWhere|PP::GotWhereItem,">",t,_) => {
						generics_enclosure_count -= 1;
						wh = quote!( #wh #t );
					},
					(PP::GotWhere|PP::GotWhereItem,_,t,_) => {
						wh = quote!( #wh #t );
						phase = PP::GotWhereItem;
					},
					_ => error(
						format!("予期しないトークン {} が含まれています",s),
						Some(&src)
					)
				}

				whole = quote!( #whole #tt );
			}

			match (&kind,phase) {
				(K::StructNamed|K::Enum,PP::GotBody)|(K::StructUnnamed|K::StructUnit|K::TypeAlias|K::TraitAlias,PP::GotSemicolon) => {},
				(K::Debug,PP::GotType) => { return Some(Data::Debug); },
				(K::Unknown,PP::Beginning) => { return None; },
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}

			let pr = ParsingResult {
				kind,
				attr, vis,
				name: name.unwrap(),
				generics, wh, body,
				src: whole.to_string()
			};

			Some( match &kind {
				K::StructUnit|K::StructUnnamed|K::StructNamed => Data::Struct(Struct::parse_from(pr)),
				K::Enum => Data::Enum(Enum::parse_from(pr)),
				K::TypeAlias => Data::Type(TypeAlias::parse_from(pr)),
				K::TraitAlias => Data::Trait(TraitAlias::parse_from(pr)),
				_ => { unreachable!(); }
			} )
		}
	}

	// 構造体をパース
	impl ParseFrom<ParsingResult,Self> for Struct {
		fn parse_from(pr:ParsingResult) -> Self {
			type F = Fields;
			type K = ParsedKind;

			let mut attributes = pr.attr;
			let fields = match pr.kind {
				K::StructUnit => F::Unit,
				K::StructUnnamed => {
					let FieldsWithAttr { fields: f, attrs: a } = UnnamedFields::parse_from(pr.body);
					attributes.extend(a);
					F::Unnamed(f)
				},
				K::StructNamed => {
					let FieldsWithAttr { fields: f, attrs: a } = NamedFields::parse_from(pr.body);
					attributes.extend(a);
					F::Named(f)
				},
				_ => { unreachable!(); }
			};

			Self {
				name: pr.name,
				generics: pr.generics,
				attributes,
				visibility: pr.vis,
				where_condition: pr.wh,
				fields,
				src: pr.src
			}
		}
	}

	// 列挙体をパース
	impl ParseFrom<ParsingResult,Self> for Enum {
		fn parse_from(pr:ParsingResult) -> Self {
			let mut variants:Vec<EnumVariant> = vec![];
			let mut enclosed:Vec<Data> = vec![];
			let mut attributes = pr.attr;
			let mut iter = pr.body.into_iter();

			loop {
				type GI = GroupItem<EnumVariant>;
				match EnumVariant::parse_from(&mut iter) {
					GI::Item(v) => variants.push(v),
					GI::Data(d) => enclosed.push(d),
					GI::InnerAttr(a) => attributes.push(a),
					GI::None => break
				}
			}

			if variants.is_empty() {
				error(
					"バリアントの数を 0 にすることはできません",
					Some(&pr.src)
				);
			}

			Self {
				name: pr.name,
				generics: pr.generics,
				attributes,
				visibility: pr.vis,
				where_condition: pr.wh,
				variants, enclosed,
				src: pr.src
			}
		}
	}

	// 列挙体のバリアントをパース
	impl<I: TI> ParseFrom<&mut I,GroupItem<Self>> for EnumVariant {
		fn parse_from(iter:&mut I) -> GroupItem<Self> {
			let src = TS::from_iter(iter.clone()).to_string();

			/// 現在のパースの過程を表す型
			enum ParsingPhase {
				Beginning,
				GotAttrHash, GotAttrExclamation, GotAttrBody,
				GotFieldName, GotFieldValue,
				GotEqual, GotDefault,
				GotEnclosedType, GotEnclosedHeader, GotEnclosedBody,
				GotComma, GotSemicolon
			}
			type PP = ParsingPhase;

			/// パースして得られたアイテムの種類
			enum ParsedType {
				Variant, Enclosed, InnerAttr
			}
			type PT = ParsedType;

			type F = Fields;

			let mut phase = PP::Beginning;
			let mut pt = PT::Variant;
			let mut attr:Vec<Attr> = vec![];
			let mut name:Option<Ident> = None;
			let mut fields = F::Unit;
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
					(PP::GotAttrHash,"!",_) => {
						pt = PT::InnerAttr;
						phase = PP::GotAttrExclamation;
					},
					(PP::GotAttrHash|PP::GotAttrExclamation,_,TT::Group(g)) => {
						attr.push( Attr::parse_from(g.stream()) );
						phase = PP::GotAttrBody;
						if matches!(pt,PT::InnerAttr) { break }
					},
					(PP::Beginning|PP::GotAttrBody,"struct"|"enum"|"type"|"trait",_) => {
						pt = PT::Enclosed;
						phase = PP::GotEnclosedType;
					},
					(PP::Beginning|PP::GotAttrBody,_,TT::Ident(i)) => {
						name = Some(i);
						phase = PP::GotFieldName;
					},
					(PP::GotFieldName,_,TT::Group(g)) => {
						match g.delimiter() {
							Delimiter::Parenthesis => {
								let FieldsWithAttr { fields: f, attrs: a } = UnnamedFields::parse_from(g.stream());
								fields = F::Unnamed(f);
								attr.extend(a);
								phase = PP::GotFieldValue;
							},
							Delimiter::Brace => {
								let FieldsWithAttr { fields: f, attrs: a } = NamedFields::parse_from(g.stream());
								fields = F::Named(f);
								attr.extend(a);
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

			type GI = GroupItem<EnumVariant>;
			match (phase,pt) {
				(PP::GotFieldName|PP::GotDefault|PP::GotFieldValue|PP::GotComma,PT::Variant) => {
					GI::Item( Self {
						attributes: attr,
						name: name.unwrap(),
						fields, is_default,
						src: whole.to_string(),
					} )
				},
				(PP::GotEnclosedBody|PP::GotSemicolon,PT::Enclosed) => {
					GI::Data( Data::parse_from(whole) )
				},
				(PP::GotAttrBody,PT::InnerAttr) => {
					GI::InnerAttr( attr.pop().unwrap() )
				},
				(PP::Beginning,_) => GI::None,
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}
		}
	}

	// 名前なしフィールドのフィールドリストをパース
	impl ParseFrom<TS,FieldsWithAttr<Self>> for UnnamedFields {
		fn parse_from(ts:TS) -> FieldsWithAttr<Self> {
			let src = quote!( (#ts) ).to_string();
			let mut fields: Vec<UnnamedField> = vec![];
			let mut enclosed: Vec<Data> = vec![];
			let mut attrs: Vec<Attr> = vec![];
			let mut iter = ts.into_iter();

			loop {
				type GI = GroupItem<UnnamedField>;
				match UnnamedField::parse_from(&mut iter) {
					GI::Item(f) => fields.push(f),
					GI::Data(d) => enclosed.push(d),
					GI::InnerAttr(a) => attrs.push(a),
					GI::None => break
				}
			}

			if fields.is_empty() {
				error(
					"フィールドの数を 0 にすることはできません",
					Some(&src)
				);
			}

			FieldsWithAttr {
				fields: Self { fields, enclosed },
				attrs
			}
		}
	}

	// 単一の名前なしフィールドをパース
	impl<I: TI> ParseFrom<&mut I,GroupItem<Self>> for UnnamedField {
		fn parse_from(iter:&mut I) -> GroupItem<Self> {
			let src = TS::from_iter(iter.clone()).to_string();

			/// 現在のパースの過程を表す型
			enum ParsingPhase {
				Beginning,
				GotAttrHash, GotAttrExclamation, GotAttrBody,
				GotPub, GotVisibility,
				GotType, GotEqual, GotDefaultVal,
				GotSubValType, GotSubValHeader, GotSubValBody,
				GotEnclosedType, GotEnclosedHeader,
				GotComma, GotSemicolon
			}
			type PP = ParsingPhase;

			/// パースして得られたアイテムの種類
			enum ParsedType {
				Field, SubtypeField, Enclosed, InnerAttribute
			}
			type PT = ParsedType;

			let mut phase = PP::Beginning;
			let mut pt = PT::Field;
			let mut can_be_enclosed = false;
			let mut attr:Vec<Attr> = vec![];
			let mut vis = TS::new();
			let mut ty = TS::new();
			let mut generics_count = 0_u8;
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
					(PP::GotAttrHash,"!",_) => {
						pt = PT::InnerAttribute;
						phase = PP::GotAttrExclamation;
					},
					(PP::GotAttrHash|PP::GotAttrExclamation,_,TT::Group(g)) => {
						attr.push( Attr::parse_from(g.stream()) );
						phase = PP::GotAttrBody;
						if matches!(pt,PT::InnerAttribute) { break }
					},
					(PP::Beginning|PP::GotAttrBody,"pub",_) => {
						vis = quote!(pub);
						phase = PP::GotPub;
					},
					(PP::GotPub,_,TT::Group(g)) => {
						if matches!(g.delimiter(),Delimiter::Parenthesis) {
							vis = quote!( #vis #g );
							phase = PP::GotVisibility;
						}
						else {
							error(
								"予期しない括弧にマッチしました",
								Some(&src)
							);
						}
					},
					(PP::Beginning|PP::GotPub|PP::GotVisibility|PP::GotAttrBody,"struct"|"enum",t) => {
						pt = PT::SubtypeField;
						can_be_enclosed = true;
						default = quote!(#t);
						phase = PP::GotSubValType;
					},
					(PP::Beginning|PP::GotPub|PP::GotVisibility|PP::GotAttrBody,"type"|"trait",_) => {
						pt = PT::Enclosed;
						phase = PP::GotEnclosedType;
					},
					(PP::Beginning|PP::GotPub|PP::GotVisibility|PP::GotAttrBody|PP::GotComma,_,TT::Ident(i)) => {
						ty = quote!(#i);
						phase = PP::GotType;
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
					(PP::GotType,_,t) => {
						ty = quote!( #ty #t );
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
					(PP::GotDefaultVal|PP::GotSubValBody,",",_) => {
						phase = PP::GotComma;
						break;
					},
					(PP::GotEqual,"struct"|"enum",t) => {
						pt = PT::SubtypeField;
						default = quote!(#t);
						phase = PP::GotSubValType;
					},
					(PP::GotEqual|PP::GotDefaultVal,_,t) => {
						default = quote!( #default #t );
						phase = PP::GotDefaultVal;
					},
					// struct { }; や enum { }; のように、末尾にセミコロンを付けると、内包型として認識するようにした
					// struct や enum といったキーワードを認識した際に can_be_enclosed フラグが付いていない場合はこのように認識されない
					(PP::GotSubValBody,";",_) => {
						if !can_be_enclosed {
							error("予期しないトークン ; が含まれています",Some(&src));
						}
						pt = PT::Enclosed;
						phase = PP::GotSemicolon;
						break;
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

			type FV = FieldValue;
			type GI = GroupItem<UnnamedField>;
			match (phase,pt) {
				(PP::GotComma|PP::GotType|PP::GotDefaultVal|PP::GotSubValBody,PT::Field) => {
					GI::Item( Self {
						attributes: attr,
						visibility: vis,
						value: match default.is_empty() {
							true => FV::Type {
								name: ty,
								default: None
							},
							false => FV::Type {
								name: ty,
								default: Some(default)
							}
						},
						src: whole.to_string()
					} )
				},
				(PP::GotComma|PP::GotType|PP::GotDefaultVal|PP::GotSubValBody,PT::SubtypeField) => {
					GI::Item( Self {
						attributes: attr,
						visibility: vis,
						value: FV::Data {
							ty: (!ty.is_empty()).then_some(ty),
							data: Data::parse_from(default)
						},
						src: whole.to_string()
					} )
				},
				(PP::GotSemicolon,PT::Enclosed) => {
					GI::Data( Data::parse_from(whole) )
				},
				(PP::GotAttrBody,PT::InnerAttribute) => {
					GI::InnerAttr( attr.pop().unwrap() )
				},
				(PP::Beginning,_) => GI::None,
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}
		}
	}

	// 名前ありフィールドのフィールドリストをパース
	impl ParseFrom<TS,FieldsWithAttr<Self>> for NamedFields {
		fn parse_from(ts:TS) -> FieldsWithAttr<Self> {
			let src = quote!( {#ts} ).to_string();
			let mut fields: Vec<NamedField> = vec![];
			let mut enclosed: Vec<Data> = vec![];
			let mut attrs: Vec<Attr> = vec![];
			let mut iter = ts.into_iter();

			loop {
				type GI = GroupItem<NamedField>;
				match NamedField::parse_from(&mut iter) {
					GI::Item(f) => fields.push(f),
					GI::Data(d) => enclosed.push(d),
					GI::InnerAttr(a) => attrs.push(a),
					GI::None => break
				}
			}

			if fields.is_empty() {
				error(
					"フィールドの数を 0 にすることはできません",
					Some(&src)
				);
			}

			FieldsWithAttr {
				fields: Self { fields, enclosed },
				attrs
			}

		}
	}

	// 単一の名前ありフィールドをパース
	impl<I: TI> ParseFrom<&mut I,GroupItem<Self>> for NamedField {
		fn parse_from(iter:&mut I) -> GroupItem<Self> {
			let src = TS::from_iter(iter.clone()).to_string();

			/// 現在のパースの過程を表す型
			enum ParsingPhase {
				Beginning,
				GotAttrHash, GotAttrExclamation, GotAttrBody,
				GotPub, GotVisibility,
				GotName, GotColon, GotType,
				GotEqual, GotDefaultVal,
				GotSubValType, GotSubValHeader, GotSubValBody,
				GotEnclosedType, GotEnclosedHeader, GotEnclosedBody,
				GotComma, GotSemicolon
			}
			type PP = ParsingPhase;

			/// パースして得られたアイテムの種類
			enum ParsedType {
				Field, SubtypeField, Enclosed, InnerAttribute
			}
			type PT = ParsedType;

			let mut phase = PP::Beginning;
			let mut pt = PT::Field;
			let mut attr:Vec<Attr> = vec![];
			let mut vis = TS::new();
			let mut name:Option<Ident> = None;
			let mut ty = TS::new();
			let mut generics_count = 0_u8;
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
					(PP::GotAttrHash,"!",_) => {
						pt = PT::InnerAttribute;
						phase = PP::GotAttrExclamation;
					},
					(PP::GotAttrHash|PP::GotAttrExclamation,_,TT::Group(g)) => {
						attr.push( Attr::parse_from(g.stream()) );
						phase = PP::GotAttrBody;
						if matches!(pt,PT::InnerAttribute) { break }
					},
					(PP::Beginning|PP::GotAttrBody,"pub",_) => {
						vis = quote!(pub);
						phase = PP::GotPub;
					},
					(PP::GotPub,_,TT::Group(g)) => {
						if matches!(g.delimiter(),Delimiter::Parenthesis) {
							vis = quote!( #vis #g );
							phase = PP::GotVisibility;
						}
						else {
							error(
								"予期しない括弧にマッチしました",
								Some(&src)
							);
						}
					},
					(PP::Beginning|PP::GotPub|PP::GotVisibility|PP::GotAttrBody,"struct"|"enum"|"type"|"trait",_) => {
						pt = PT::Enclosed;
						phase = PP::GotEnclosedType;
					},
					(PP::Beginning|PP::GotPub|PP::GotVisibility|PP::GotAttrBody,_,TT::Ident(i)) => {
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
						pt = PT::SubtypeField;
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

			type FV = FieldValue;
			type GI = GroupItem<NamedField>;
			match (phase,pt) {
				(PP::GotComma|PP::GotType|PP::GotDefaultVal|PP::GotSubValBody,PT::Field) => {
					GI::Item( Self {
						attributes: attr,
						visibility: vis,
						name: name.unwrap(),
						value: match default.is_empty() {
							true => FV::Type {
								name: ty,
								default: None
							},
							false => FV::Type {
								name: ty,
								default: Some(default)
							}
						},
						src: whole.to_string()
					} )
				},
				(PP::GotComma|PP::GotType|PP::GotDefaultVal|PP::GotSubValBody,PT::SubtypeField) => {
					GI::Item( Self {
						attributes: attr,
						visibility: vis,
						name: name.unwrap(),
						value: FV::Data {
							ty: (!ty.is_empty()).then_some(ty),
							data: Data::parse_from(default)
						},
						src: whole.to_string()
					} )
				},
				(PP::GotSemicolon|PP::GotEnclosedBody,PT::Enclosed) => {
					GI::Data( Data::parse_from(whole) )
				},
				(PP::GotAttrBody,PT::InnerAttribute) => {
					GI::InnerAttr( attr.pop().unwrap() )
				},
				(PP::Beginning,_) => GI::None,
				_ => {
					error("終わり方が正しくありません",Some(&src));
				}
			}
		}
	}

	// 型エイリアスをパース
	impl ParseFrom<ParsingResult,Self> for TypeAlias {
		fn parse_from(pr:ParsingResult) -> Self {
			let ParsingResult {
				name, mut generics, body, attr, vis, src, ..
			} = pr;
			if !generics.is_empty() {
				generics = quote!( <#generics> );
			}

			Self {
				name: quote!( #name #generics ),
				artifact: body,
				attributes: attr,
				visibility: vis,
				src
			}
		}
	}

	// トレイトエイリアスをパース
	impl ParseFrom<ParsingResult,Self> for TraitAlias {
		fn parse_from(pr:ParsingResult) -> Self {
			let ParsingResult {
				name, generics, body, attr, vis, wh, src, ..
			} = pr;

			Self {
				name, generics,
				artifact: body,
				attributes: attr,
				visibility: vis,
				where_condition: wh,
				src
			}
		}
	}

	// アトリビュートをパース
	impl ParseFrom<TS,Self> for Attr {
		fn parse_from(ts:TS) -> Self {
			let mut iter = ts.clone().into_iter();

			let kind = match iter.next() {
				Some(TT::Ident(i)) => i.to_string(),
				Some(t) => error(
					format!("予期しないトークン {} が含まれています",t.into_token_stream()),
					Some(&ts.to_string())
				),
				None => error(
					"アトリビュートが空です",
					Some(&ts.to_string())
				)
			};
			let mut a = match &kind[..] {
				"default" => Self::Default,
				"pub_all" => Self::PubAll,
				_ => Self::Other(ts.clone())
			};

			/// 現在のパースの過程を表す型
			enum ParsingPhase {
				Beginning,
				GotEqual, GotGroup, GotLiteral
			}
			type PP = ParsingPhase;
			let mut phase = PP::Beginning;

			type VI = Vec<Ident>;
			for tt in iter {
				let s = tt.to_string();
				match (&phase,&kind[..],&a,&s[..],tt) {
					(PP::Beginning,"derive",Self::Other(_),_,TT::Group(g)) => {
						if let Some(v) = VI::parse_from(g.stream()) {
							a = Self::Derive(v);
						}
						phase = PP::GotGroup;
					},
					(PP::Beginning,"allow",Self::Other(_),_,TT::Group(g)) => {
						if let Some(v) = VI::parse_from(g.stream()) {
							a = Self::Allow(v);
						}
						phase = PP::GotGroup;
					},
					(PP::Beginning,"cfg",Self::Other(_),_,TT::Group(g)) => {
						a = Self::Cfg(g.stream());
						phase = PP::GotGroup;
					},
					(PP::Beginning,"doc",Self::Other(_),"=",_) => {
						phase = PP::GotEqual;
					},
					(PP::GotEqual,"doc",Self::Other(_),_,TT::Literal(l)) => {
						a = Self::Doc(l);
						phase = PP::GotLiteral;
					},
					(PP::Beginning,_,Self::Other(_),_,_) => {},
					_ => { a = Self::Other(ts.clone()); }
				}
			}

			a
		}
	}

	// アトリビュートに含まれるコンマ区切りの Ident のパースを試みる
	impl ParseFrom<TS,Option<Self>> for Vec<Ident> {
		fn parse_from(ts:TS) -> Option<Self> {
			let mut items:Vec<Ident> = vec![];

			/// 現在のパースの過程を表す型
			enum ParsingPhase {
				Beginning, GotName, GotComma
			}
			type PP = ParsingPhase;
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
	}

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

			// フィールドの種類によらない抽象化
			impl Struct {
				fn exchange_with_fields<F: GetSubtype>(&mut self,fields:&mut CapsuledFields<F>) {
					let Self {
						ref mut attributes,
						ref visibility,
						..
					} = self;

					let mut st = fields.collect_subtype();
					copy_attr_to_subtype(&*attributes, &mut st);

					for d in fields.enclosed.iter_mut() {
						inherit_visibility(visibility,d);
					}
				}
			}

			let (st,fields) = unsafe {
				let p = self as *mut Struct;
				( (&mut *p), &mut (&mut *p).fields )
			};

			type F = Fields;
			match fields {
				F::Unit => {},
				F::Unnamed(f) => st.exchange_with_fields(f),
				F::Named(f) => st.exchange_with_fields(f)
			}

			remove_duplicate(&mut self.attributes);

			fields.modify();
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

	impl Modify for Fields {
		fn modify(&mut self) {
			match self {
				Self::Unit => {},
				Self::Unnamed(f) => { f.modify(); },
				Self::Named(f) => { f.modify(); }
			}
		}
	}

	impl<F: Modify> Modify for CapsuledFields<F> {
		fn modify(&mut self) {
			for f in self.fields.iter_mut() {
				f.modify();
			}
			for d in self.enclosed.iter_mut() {
				d.modify();
			}
		}
	}

	impl Modify for UnnamedField {
		fn modify(&mut self) {
			self.check_pub_all();
			self.check_default();

			let Self {
				ref attributes,
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

	impl Modify for NamedField {
		fn modify(&mut self) {
			self.check_pub_all();
			self.check_default();

			let Self {
				ref attributes,
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

	impl Modify for FieldValue {
		fn modify(&mut self) {
			match self {
				Self::Type{..} => {},
				Self::Data{data,..} => { data.modify(); }
			}
		}
	}

	trait CollectSubType {
		/// このオブジェクトに含まれるサブ構造体/列挙体のリストを返す
		fn collect_subtype(&mut self) -> Vec<&mut Data>;
	}
	impl CollectSubType for Vec<EnumVariant> {
		fn collect_subtype(&mut self) -> Vec<&mut Data> {
			self.iter_mut()
			.map( |v| v.fields.collect_subtype() )
			.flatten()
			.collect()
		}
	}
	impl CollectSubType for Fields {
		fn collect_subtype(&mut self) -> Vec<&mut Data> {
			match self {
				Self::Unit{..} => vec![],
				Self::Unnamed(f) => f.collect_subtype(),
				Self::Named(f) => f.collect_subtype()
			}
		}
	}
	impl<F: GetSubtype> CollectSubType for CapsuledFields<F> {
		fn collect_subtype(&mut self) -> Vec<&mut Data> {
			let Self {
				ref mut fields,
				ref mut enclosed,
				..
			} = self;

			fields.iter_mut()
			.filter_map(|f| f.get_subtype() )
			.chain( enclosed.iter_mut() )
			.collect::<Vec<_>>()
		}
	}
	trait GetSubtype {
		/// フィールドの値が構造体/列挙体を包含するものであれば、それを返す
		fn get_subtype(&mut self) -> Option<&mut Data>;
	}
	impl GetSubtype for UnnamedField {
		fn get_subtype(&mut self) -> Option<&mut Data> {
			self.value.get_subtype()
		}
	}
	impl GetSubtype for NamedField {
		fn get_subtype(&mut self) -> Option<&mut Data> {
			self.value.get_subtype()
		}
	}
	impl GetSubtype for FieldValue {
		fn get_subtype(&mut self) -> Option<&mut Data> {
			match self {
				Self::Type{..} => None,
				Self::Data {data,..} => Some(data)
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
			type F = Fields;
			match self.fields {
				F::Unit => {},
				F::Unnamed( UnnamedFields {
					ref mut fields,
					ref mut enclosed,
					..
				} ) => {
					for f in fields.iter_mut() {
						f.visibility = quote!(pub);
					}
					for d in enclosed.iter_mut() {
						d.pub_all();
					}
				},
				F::Named( NamedFields {
					ref mut fields,
					ref mut enclosed,
					..
				} ) => {
					for f in fields.iter_mut() {
						f.visibility = quote!(pub);
					}
					for d in enclosed.iter_mut() {
						d.pub_all();
					}
				}
			}
			self.fields.collect_subtype()
			.iter_mut()
			.for_each(|d| d.pub_all() );
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
			let Self {
				ref mut variants,
				ref mut enclosed,
				..
			} = self;
			variants.collect_subtype()
			.into_iter()
			.chain( enclosed.iter_mut() )
			.for_each(|d| d.pub_all() );
		}
	}
	impl PubAll for UnnamedField {
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
	impl PubAll for NamedField {
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
			self.fields.set_default();
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
			self.fields.set_default();
		}
	}
	impl SetDefault for Fields {
		fn check_default(&mut self) {}
		fn set_default(&mut self) {
			match self {
				Self::Unit => {},
				Self::Named( NamedFields {
					ref mut fields,
					..
				} ) => {
					for f in fields.iter_mut() {
						f.set_default();
					}
				},
				Self::Unnamed( UnnamedFields {
					ref mut fields,
					..
				} ) => {
					for f in fields.iter_mut() {
						f.set_default();
					}
				}
			}
		}
	}
	impl SetDefault for UnnamedField {
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
	impl SetDefault for NamedField {
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
				Self::Data { data, .. } => { data.set_default(); },
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

	/// 構造体のフィールドに付されたアトリビュート (`doc`/`cfg`) を、値となるサブ構造体/列挙体に移動或いはコピーする
	fn move_field_attrs_to_subtype(pal:&Vec<Attr>,value:&mut FieldValue) {
		let tmp = pal.iter()
		.filter_map(|a| {
			match a {
				Attr::Doc(_)|Attr::Cfg(_) => Some(a.clone()),
				_ => None,
			}
		})
		.collect::<Vec<_>>();
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

		// それぞれの `Data` に対して処理を行う
		for d in dl.iter_mut() {
			let (ca,will_copy_derive) = match d {
				Data::Struct(s) => (&mut s.attributes,true),
				Data::Enum(e) => (&mut e.attributes,true),
				Data::Type(t) => (&mut t.attributes,false),
				Data::Trait(t) => (&mut t.attributes,false),
				Data::Debug => { unreachable!(); }
			};

			// derive, allow, cfg は他のアトリビュートよりも影響が大きいことが多いので、他のアトリビュートよりも前に追加する
			// そのために、一時的的なリストに順に追加してから、置き換える
			let mut ca_tmp:Vec<Attr> = vec![];

			for c in copied_cfg.iter() {
				ca_tmp.push(
					Attr::Cfg(c.clone())
				);
			}
			if copied_allow.len()>0 {
				ca_tmp.push(
					Attr::Allow(copied_allow.clone())
				);
			}
			if will_copy_derive && copied_derive.len()>0 {
				ca_tmp.push(
					Attr::Derive(copied_derive.clone())
				);
			}

			ca_tmp.append(ca);
			swap(ca,&mut ca_tmp);
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
				let mut this = quote!( #a #v struct #n #g #w );

				let f = self.fields.compose(global);
				this = quote!( #this #f );

				type F = Fields;
				if matches!(self.fields,F::Unit|F::Unnamed(..)) {
					this = quote!( #this; );
				}

				*global = quote!( #global #this );
			}

			match self.has_default() {
				QuadBool::NotAllowed => error(
					"一部の値にはデフォルト値が指定されていますが、他の値には指定されていません",
					Some(&self.src)
				),
				QuadBool::TrueRequired|QuadBool::TrueOptional => {
					let a = self.attributes.compose_default(global);
					let w = add_where(&self.where_condition);
					let f = self.fields.compose_default(global);
					let this = quote!(
						#a impl #g std::default::Default for #n #g #w {
							fn default() -> Self {
								Self #f
							}
						}
					);
					*global = quote!( #global #this );
				},
				_ => {}
			}

			quote!( #n #g )
		}
		fn compose_default(&self,_:&mut TS) -> TS {
			quote!( std::default::Default::default() )
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
			quote!( std::default::Default::default() )
		}
	}

	impl Compose for EnumVariant {
		fn compose(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose(global);
			let n = &self.name;
			let f = self.fields.compose(global);

			quote!( #a #n #f )
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose_default(global);
			let n = &self.name;
			let f = self.fields.compose_default(global);

			quote!( #a #n #f )
		}
	}

	impl Compose for Fields {
		fn compose(&self,global:&mut TS) -> TS {
			match &self {
				Self::Unit => TS::new(),
				Self::Unnamed(f) => f.compose(global),
				Self::Named(f) => f.compose(global)
			}
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			match &self {
				Self::Unit => TS::new(),
				Self::Unnamed(f) => f.compose_default(global),
				Self::Named(f) => f.compose_default(global)
			}
		}
	}

	impl Compose for UnnamedFields {
		fn compose(&self,global:&mut TS) -> TS {
			for d in self.enclosed.iter() {
				d.compose(global);
			}

			let mut grouped = TS::new();
			for f in self.fields.iter() {
				let ft = f.compose(global);
				grouped = quote!( #grouped #ft, );
			}
			quote!( ( #grouped ) )
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			let mut grouped = TS::new();
			for f in self.fields.iter() {
				let ft = f.compose_default(global);
				grouped = quote!( #grouped #ft, );
			}
			quote!( ( #grouped ) )
		}
	}

	impl Compose for NamedFields {
		fn compose(&self,global:&mut TS) -> TS {
			for d in self.enclosed.iter() {
				d.compose(global);
			}

			let mut grouped = TS::new();
			for f in self.fields.iter() {
				let ft = f.compose(global);
				grouped = quote!( #grouped #ft, );
			}
			quote!( { #grouped } )
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			let mut grouped = TS::new();
			for f in self.fields.iter() {
				let ft = f.compose_default(global);
				grouped = quote!( #grouped #ft, );
			}
			quote!( { #grouped } )
		}
	}

	impl Compose for UnnamedField {
		fn compose(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose(global);
			let vis = &self.visibility;
			let val = self.value.compose(global);
			quote!( #a #vis #val )
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose_default(global);
			let v = self.value.compose_default(global);
			quote!( #a #v )
		}
	}

	impl Compose for NamedField {
		fn compose(&self,global:&mut TS) -> TS {
			let a = self.attributes.compose(global);
			let n = &self.name;
			let vis = &self.visibility;
			let val = self.value.compose(global);
			quote!( #a #vis #n: #val )
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
				Self::Data {data,ty} => {
					let cd = data.compose(global);
					ty.as_ref().map_or(cd,|r| r.clone() )
				}
			}
		}
		fn compose_default(&self,global:&mut TS) -> TS {
			match self {
				Self::Type {default:Some(d),..} => d.clone(),
				Self::Type {default:None,..} => {
					quote!( std::default::Default::default() )
				},
				Self::Data{data,..} => data.compose_default(global)
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
				ref where_condition,
				..
			} = self;
			let attr = attributes.compose(global);
			let t = Ident::new(&format!("GenericTypeFor{}",name),Span::call_site());
			let (gt,gi) = match generics.is_empty() {
				true => (
					TS::new(),
					quote!(<#t>)
				),
				false => (
					quote!(<#generics>),
					quote!(<#t,#generics>)
				)
			};
			let (wt,wi) = match where_condition.is_empty() {
				true => (TS::new(),TS::new()),
				false => (
					quote!( where #where_condition ),
					quote!( , #where_condition )
				)
			};
			let this = quote!(
				#attr #visibility trait #name #gt: #artifact #wt {}
				impl #gi #name #gt for #t where #t: #artifact #wi {}
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
			convert_as_data(
				match self {
					Self::Struct(s) => s.has_default(),
					Self::Enum(e) => e.has_default(),
					_ => B::TrueOptional
				}
			)
		}
	}

	impl HasDefault for Struct {
		fn has_default(&self) -> B {
			self.fields.has_default()
		}
	}

	impl HasDefault for Enum {
		fn has_default(&self) -> B {
			merge_as_variants(
				self.variants.iter()
				.map(|v| v.has_default() )
			)
		}
	}

	impl HasDefault for EnumVariant {
		fn has_default(&self) -> B {
			// バリアント内のフィールドのデフォルト値の有無とバリアント自体のデフォルト値の有無を複合的に判断してデフォルト値の有無を決定する
			match (self.is_default,self.fields.has_default()) {
				(true,B::TrueRequired|B::TrueOptional) => B::TrueRequired,
				(false,B::TrueRequired) => B::TrueRequired,
				(false,B::False|B::TrueOptional) => B::False,
				(true,B::False)|(_,B::NotAllowed) => B::NotAllowed,
			}
		}
	}

	impl HasDefault for Fields {
		fn has_default(&self) -> B {
			match &self {
				Self::Unit => B::TrueOptional,
				Self::Unnamed(f) => merge_as_fields(
					f.fields.iter()
					.map(|f| f.value.has_default() )
				),
				Self::Named(f) => merge_as_fields(
					f.fields.iter()
					.map(|f| f.value.has_default() )
				)
			}
		}
	}

	impl HasDefault for FieldValue {
		fn has_default(&self) -> B {
			match self {
				Self::Type{default: Some(_),..} => B::TrueRequired,
				Self::Type{default: None,..} => B::False,
				Self::Data{data,..} => data.has_default()
			}
		}
	}

	#[derive(Clone,Copy,Debug)]
	/// `has_default` で用いられる4元ブール値
	pub enum QuadBool {
		/// 真。この値の場合は必ずデフォルト値を構成しなければならない
		TrueRequired,
		/// 真。この値が定義されていても必ずしもデフォルト値を構成する必要はない
		TrueOptional,
		/// 偽。この値はデフォルト値を定義していないことを表す
		False,
		/// 判別不能。これは列挙体において複数のバリアントがデフォルト値に指定されている場合など、デフォルト値の有無が不適切に定まっている場合に該当する。
		NotAllowed
	}
	type B = QuadBool;

	/// `QuadBool` をフィールドの規則に則って縮約する
	fn merge_as_fields(iter:impl IntoIterator<Item=B>) -> B {
		iter.into_iter()
		.reduce(|b1,b2| {
			// 1つでも TrueRequired や False があればそれが優先される
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

	/// `QuadBool` をバリアントの規則に従って縮約する
	fn merge_as_variants(iter:impl IntoIterator<Item=B>) -> B {
		iter.into_iter()
		.reduce(|b1,b2| {
			// 2個以上の TrueRequired が存在することが認められない。他は TrueOptional 或いは False でなければならない
			match (b1,b2) {
				(B::NotAllowed|B::TrueOptional,_)|(_,B::NotAllowed|B::TrueOptional) => B::NotAllowed,
				(B::TrueRequired,B::TrueRequired) => B::NotAllowed,
				(B::TrueRequired,_)|(_,B::TrueRequired) => B::TrueRequired,
				(B::False,B::False) => B::False
			}
		})
		.unwrap_or(B::NotAllowed)
	}

	/// 構造体/列挙体のフィールド/バリアント各々の `QuadBool` の値を縮約した `QuadBool` の値をサブデータとしての `QuadBool` 値に変換する
	fn convert_as_data(b:B) -> B {
		match b {
			B::NotAllowed => B::NotAllowed,
			// サブデータとしては TrueRequired になったとしても、上の階層のデータがデフォルト値が必要とは限らないので TrueOptional を返す
			B::TrueRequired|B::TrueOptional => B::TrueOptional,
			B::False => B::False
		}
	}

}
use has_default::*;
