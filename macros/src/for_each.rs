use proc_macro::TokenStream as TS1;
use proc_macro2::{
	Span,
	TokenStream as TS,
	TokenTree as TT,
	Delimiter as TD,
};
use syn::{
	parse2,
	Expr,ExprRange,ExprTuple,ExprPath,
	Ident,Meta,Type
};
use quote::{quote,ToTokens};



/// ドキュメント付きで `for_each!` を定義するマクロ
macro_rules! for_each_interface {
	( $( $func_name:ident $exec_mode:ident )+ ) => { $(
		#[proc_macro]
		pub fn $func_name(item:TokenStream) -> TokenStream {
			//! ## `for_each!`, `par_for_each!`, `par_bridge_for_each!`
			//! * 複数の配列やコレクション等に対して簡単な表記で `for_each` が実行できるマクロです。
			//! * イテレート対象の項目を `each(array)` や `index(3,2)` のように関数的な表記で指定します。
			//! * 直列と並列を簡単に切り替えられ、 `cfg(disable_parallel_execution)` が有効になっている場合は並列指定でも直列に切り替わります。
			//! * `ndarray` の多次元配列に対して扱いやすいように工夫されています。
			//!
			//! ### 例
			//! ```rust
			//! // 2つの ndarray に対してループを回します。
			//! par_for_each! {
			//! 	x1 = each_nd(array1)
			//! 	x2 = each_nd(array2)
			//! 	{ /* 処理内容 */ }
			//! }
			//!
			//! // 一方のデータを倍にして他方に代入します。
			//! par_for_each! {
			//! 	x1 = each_nd(mut array1)
			//! 	x2 = each_nd(    array2)
			//! 	{ *x1 = *x2 * 2; }
			//! }
			//!
			//! // インデクスと共にループを回す
			//! par_for_each! {
			//! 	x   = each_nd(array)
			//! 	i,j = index(16,32)
			//! 	{ /* 処理内容 */ }
			//! }
			//!
			//! // リダクションにより、合計値を得る (OpenMP の reduction に似ている)
			//! let mut sum:f64;
			//! par_for_each! {
			//! 	x = each_nd(array)
			//! 	fold(+:sum) // sum_inner = fold(+:sum_outer) と内外で変数を変えても良い
			//! 	{ sum += *x; }
			//! }
			//! ```
			//!
			//! ### マクロの名称
			//!
			//! | | |
			//! |:--|:--|
			//! | 直列 | `for_each` |
			//! | 並列 | `par_for_each` |
			//! | 並列ブリッジ | `par_bridge_for_each` |
			//!
			//! * 並列ブリッジは `.par_bridge()` を使用して直列の要素を並列化したものです。他の並列手法に比べてパフォーマンスが落ちる可能性がありますが、他の並列手法に対応していないものをイテレートすることも可能です。
			//!
			//! ### 関数一覧
			//!
			//! #### `each_nd(a)`
			//! ```rust
			//! cell = each_nd(a)
			//! &cell = each_nd(a)
			//! each_nd(a)
			//! ```
			//! * `ndarray` のN次元配列 `array` の各要素に対してイテレートします。
			//! * ループ内からは `cell` によりアクセスできます。
			//! * `cell` は `&T` 型です。
			//! * 2行目の表式を使うと `cell` は参照が外れた `T` 型になります。
			//! * 3行目の表式ではループ内の要素アクセスも、ループ外の配列へのアクセスも変数 `a` を使用します。
			//!
			//! #### `each_nd(mut a)`
			//! ```rust
			//! cell = each_nd(mut a)
			//! cell = mut each_nd(a)
			//! each_nd(mut a)
			//! ```
			//! * `ndarray` のN次元配列 `array` の各要素に対してミュータブルにイテレートします。
			//! * `cell` は `&mut T` 型です。
			//! * 3行目の表式ではループ内の要素アクセスも、ループ外の配列へのアクセスも変数 `a` を使用します
			//!
			//! #### `index(n,m,...)`
			//! ```rust
			//! n = index(10)
			//! i,j,k = index(3,2,5)
			//! (i,j,k) = index(3,2,5)
			//! tuple = index(3,2,5)
			//! ```
			//! * 整数に対してイテレートします。
			//! * 1行目の表式の場合、 `n` は 0..10 でイテレートします。
			//! * 2行目の表式の場合、 `i` は `0..3`, `j` は `0..2`, `k` は `0..5` で 3×2×5 のイテレートになります。インデクスの個数は任意です。
			//! * N次元配列と併せてイテレートすることでN次元配列のインデクスが得られます。
			//! * 3,4 行目のようにタプル型としてインデクスを受け取ることも可能です
			//!
			//! #### `index(range)`
			//! ```rust
			//! n = index(3..6)
			//! n = index(2_u8..=9_u8)
			//! ```
			//! * 整数の範囲に対してイテレートします
			//! * 1行目のように大きい方の端が開いている範囲に関しては多くの整数型に対応しており、通常は型を明示しなくても使用できます。
			//! * 2行目のように大きい方の端が閉じている範囲に関しては `rayon` ライブラリの制約上、 `i16`, `i8`, `u16`, `u8` しか使用できません。 Rust では型が明示されない整数は `i32` となってしまうため、例のように型を明示した整数表記を使用する必要があります。
			//!
			//! #### ~~`index(from a)`~~ (準備中)
			//! ```rust
			//! i,j,k = index(from a)
			//! tuple = index(from a)
			//! ```
			//! * こちらはイテレートする範囲を与える代わりにN次元配列 `a` の形状に合わせてイテレートします。
			//! * `par_for_each` には対応していません。
			//!
			//! #### ~~`lanes(axis:n a)`~~ (準備中)
			//! ```rust
			//! sa = lanes(axis:0 a)
			//! sa = lanes(axis:2 a)
			//! ```
			//! * `array.lanes(Axis(axis))` と同じで、N次元配列 `a` の `n` 軸以外でイテレートします。 `sa` には `n` 軸からなる1次元配列が与えられます。
			//! * `par_for_each` には対応していません。
			//!
			//! #### `lanes(axis:n mut a)`
			//! ```rust
			//! sa = lanes(axis:0 mut a)
			//! ```
			//! * こちらは各レーンの要素に対して書き換え可能なイテレータです。
			//! * `par_for_each` には対応していません。
			//!
			//! #### `fold(op:var)`
			//! ```rust
			//! fold(+:sum)
			//! sum_inner = fold(+:sum_outer)
			//! ```
			//! * リダクションを行います。つまりループ内で与えた値を演算して外の変数に代入します。 OpenMP の `reduction` と同様に機能します。
			//! * 1行目の場合、ループ内で `sum` に値を足す処理を行うと、ループ外の `sum` に全てのループでの合計値が書き込まれます。並列でも安全に計算されます。
			//! ```rust
			//! let mut sum:u8 = 0;
			//! par_for_each! {
			//! 	x = index(10)
			//! 	fold(+:sum)
			//! 	{ sum += x; }
			//! }
			//! assert!(sum,36);
			//! ```
			//! * 2行目のように、内外で使用する変数を分けることも可能です。
			//! * `op` の箇所には演算子が入ります。対応している演算は以下の通りです。
			//!
			//! | 演算子 | 演算 | 単位元 |
			//! |:--|:--|:--|
			//! | `+`, `add` | 加算 | `0` |
			//! | `-`, `sub` | 減算 | `0` |
			//! | `*`, `mul` | 乗算 | `1` |
			//! | `max` | 最大値 | 表現可能な最小の値 |
			//! | `min` | 最小値 | 表現可能な最大の値 |
			//! | `&`, `bitand` | ビットAND | `!0` |
			//! | `\|`, `bitor` | ビットOR | `0` |
			//! | `^`, `bitxor` | ビットXOR | `0` |
			//! | `&&`, `and` | ブールAND | `true` |
			//! | `\|\|`, `or` | ブールOR | `false` |
			//!
			//! * 単位元の型が判定できないためにコンパイルエラーを発することがあり、その場合は `fold(+:var)` の代わりに `fold(+(f64):var)` などと記載して型を明示することができます。
			//! * OpenMP の挙動に準拠するために、例えば加算であれば外の変数の元々の値にループでの値を足し合わせていきますが、ループの値を足し合わせた結果を外の変数に代入するのであれば `fold` の代わりに `fold_assign` を使用します。
			//! 	* この場合には、外の変数は `let mut sum:u8;` のように初期化していない状態で定義しておくことも可能です。
			//!
			//! ### `reduce(op:var)`
			//! ```rust
			//! reduce(+:sum)
			//! sum_inner = reduce(+:sum_outer)
			//! ```
			//! * リダクションを行います。こちらは `fold` と違い、ループ内では加算などの処理は必要なく、単に変数に代入するだけで済みます。ループ処理後に指定した演算を行なって、変数に代入します。対応する演算は同じです。
			//! ```rust
			//! let mut sum:u8 = 0;
			//! par_for_each! {
			//! 	x = index(10)
			//! 	reduce(+:s)
			//! 	{ sum = x; }
			//! }
			//! assert!(sum,36);
			//! ```
			//! * 並列計算の場合、通常は `fold` を使用した方が効率の良い計算ができます。
			//! * `fold` と同様、単位元の型を明示することができます: `reduce(add(f64):sum)`
			//! * `fold_assign` と同じく `reduce_assign` も使用できます。詳しくは `fold` を参照。
			//!
			//! ### `par_cond_bool(condition)`
			//! * `par_for_each` や `par_bridge_for_each` の場合に、並列に実行する条件 (ブール値) を指定します。 `for_each` で指定しても無視されます。
			//! * 通常は無条件に並列実行しますが、このオプションが付加されている場合は `condition` の評価値が真の場合のみ並列に実行されます。
			//! * 複数個の `par_cond_bool` オプションが指定された場合は、それら全てが真の場合にのみ並列に実行されます。
			//! * 実行時に判定を行うので、コンパイル時点で並列の場合、直列の場合双方でビルドが通るようにしておく必要があります。
			//!
			//! ### `par_cond_cfg(condition)`
			//! * `par_for_each` や `par_bridge_for_each` の場合に、並列に実行する条件 (cfg のメタ値) を指定します。 `for_each` で指定しても無視されます。
			//! * 通常は無条件に並列実行しますが、このオプションが付加されている場合は `condition` を満たす場合 (`#[cfg(condition)]` アトリビュードで無視されない場合) のみ並列に実行されます。
			//! * 複数個の `par_cond_cfg` オプションが指定された場合は、それら全てを満たす場合にのみ並列に実行されます。
			//!
			//! ### `debug()`
			//! ビルド時にマクロ展開した結果を出力します。コンパイルエラーが発生する場合に原因を探すのに役立ちます。
			//!
			//! ### 注意
			//! * `par_for_each` で複数の対象をイテレートする場合、全ての要素数が一致していないと実行時エラーが発生します。

			for_each_impl(item,EM::$exec_mode)
		}
	)+ };
}
pub(crate) use for_each_interface;



/// `for_each!` 実装のエントリポイント
pub fn for_each_impl(st:TS1,exec:EM) -> TS1 {
	let ts = Input::new(TS::from(st),exec).construct();
	TS1::from(ts)
}



/// データ構造を定義するモジュール
mod typedef {
	use super::*;

	/// 与えられた情報をパースしたデータ
	pub struct Input {
		/// 入力したマクロのコード
		pub src: String,
		/// `debug()` がオプションに入っていて、デバッグ出力するかどうか
		pub debug: bool,
		/// 引数のリスト
		pub args: Vec<Arg>,
		/// リダクションのモード
		pub reduction: RM,
		/// 直列/並列の実行モード
		pub execution: EM,
		/// 並列実行の場合、実際に並列になる条件 (`bool` 型による実行時指定)
		pub par_cond_bool: Vec<Expr>,
		/// 並列実行の場合、実際に並列になる条件 (`cfg(*)` によるコンパイル時指定)
		pub par_cond_cfg: Vec<Meta>,
		/// `for_each` で実行される内容
		pub body: Option<TS>
	}

	#[allow(dead_code)]
	/// ループの項目
	pub enum Arg {
		/// 多次元の整数インデクスを与えます
		IndexMultipleInt {
			vars: Vec<Ident>,
			size: Vec<Expr>
		},
		/// 1次元の整数インデクスを与えます
		IndexInt {
			var: Ident,
			size: Expr
		},
		/// 1次元の範囲を指定したインデクスを与えます
		IndexRange {
			var: Ident,
			range: ExprRange
		},
		/// NDArray に準拠したインデクスを与えます
		IndexFromNdArray,
		/// 配列をイテレートします
		Each {
			/// NDArray の場合は true 、一般の配列の場合は false
			nd: bool,
			/// ミュータブルなイテレートか
			mutable: bool,
			/// `var` を参照外しするかどうか
			dereference: bool,
			/// ループ内で取り出す変数名
			var: Ident,
			/// イテレートする対象の NDArray
			array: Expr
		},
		/// NDArray のある次元軸に関してイテレートします
		Lanes {
			mutable: bool,
			var: Ident,
			axis: Expr,
		},
		/// リダクションします
		Reduction {
			assignment: bool,
			operator: ReductionOperator,
			var_inside: Ident,
			var_outside: Expr
		}
	}

	/// リダクションの演算子
	pub enum ReductionOperator {
		#[doc="加法"] Add(Option<Type>),
		#[doc="減法"] Sub(Option<Type>),
		#[doc="乗法"] Mul(Option<Type>),
		#[doc="ビット論理積"] BitAnd(Option<Type>),
		#[doc="ビット論理和"] BitOr(Option<Type>),
		#[doc="ビット排他的論理和"] BitXor(Option<Type>),
		#[doc="ブール値の論理積"] And,
		#[doc="ブール値の論理和"] Or,
		#[doc="最大値"] Max(Option<Type>),
		#[doc="最小値"] Min(Option<Type>)
	}
	pub type RO = ReductionOperator;

	#[derive(Clone,Copy,PartialEq,Eq)]
	/// リダクションの仕方
	pub enum ReductionMode {
		/// リダクションを行わない
		None,
		/// reduce のリダクション
		Reduce,
		/// fold のリダクション
		Fold
	}
	pub type RM = ReductionMode;

	#[derive(Clone,Copy,PartialEq,Eq)]
	/// 実行モード
	pub enum ExecutionMode {
		/// 並列
		Parallel,
		/// `ParallelBridge` による並列
		ParallelBridge,
		/// 直列
		Serial
	}
	pub type EM = ExecutionMode;

	/// 値を実際の形式に整形したデータ
	pub struct Converted {
		/// 実行モードの指定
		pub execution: EM,
		/// リダクションの指定
		pub reduction: RM,
		/// 実行内容の本体
		pub body: TS,
		/// イテレータのリスト
		pub iterators: Vec<TS>,
		/// 無名関数の引数となる変数のリスト
		pub lambda_args: Vec<TS>,
		/// ボディの実行前に事前の定義する内容の文
		pub advance_defs: TS,
		/// リダクションする場合の、単位元のリスト
		pub reduction_identities: Vec<TS>,
		/// リダクションする場合の、ボディからアクセスできる変数のリスト
		pub reduction_vars_inside: Vec<Ident>,
		/// リダクションする場合の、外部で使用する一時変数のリスト
		pub reduction_vars_outside: Vec<Ident>,
		/// リダクションする場合の、外部変数への代入文
		pub reduction_outside_assignment: TS,
		/// リダクションする場合の、 reduce 関数の計算内容
		pub reduction_func: Vec<TS>,
		/// リダクションする場合の、 reduce 関数の第1引数のリスト
		pub reduction_func_args_1st: Vec<Ident>,
		/// リダクションする場合の、 reduce 関数の第2引数のリスト
		pub reduction_func_args_2nd: Vec<Ident>,
		/// `use rayon::iter::IntoParallelIterator;` を追加するフラグ
		pub use_into_parallel_iterator: bool,
		/// `use rayon::iter::ParallelBridge;` を追加するフラグ
		pub use_parallel_bridge: bool,
		/// `use rayon::iter::ParallelIterator;` を追加するフラグ
		pub use_parallel_iterator: bool,
		/// `use rayon::iter::IndexedParallelIterator;` を追加するフラグ
		pub use_indexed_parallel_iterator: bool,
		/// `use ndarray::indices;` を追加するフラグ
		pub use_ndarray_indices: bool
	}

}
use typedef::*;
pub use typedef::EM;



/// ユーテリティ群
mod utils {
	use super::*;

	/// `Option<_>` を返す関数内で使用できる、 enum 型の中身を取り出すマクロ
	macro_rules! unwrap_enum {
		( $case:path = $input:expr ) => {
			match $input {
				$case(x) => x,
				_ => { return None }
			}
		};
	}
	pub(super) use unwrap_enum;

	/// イテレータに try_map_collect 関数を実装する
	pub trait TryMapCollect<T1> {
		/// イテレータの要素に対してクロージャを実行した際に1つでも `None` が返ってきたら `None` を返し、全てが `Some(_)` であれば、マップした上で `Vec` 型に集約して返します。
		fn try_map_collect<T2>(self,f:impl FnMut(T1)->Option<T2>) -> Option<Vec<T2>>;
	}
	impl<I,T1> TryMapCollect<T1> for I where I: Iterator<Item=T1> {
		fn try_map_collect<T2>(self,mut f:impl FnMut(T1)->Option<T2>) -> Option<Vec<T2>> {
			let mut failure = false;
			let v = self.map_while(|item| {
				match f(item) {
					ret @ Some(_) => ret,
					None => { failure = true; None }
				}
			}).collect::<Vec<T2>>();
			if !failure { Some(v) }
			else { None }
		}
	}

	/// トークンツリーの Vec 型をコンマ区切りなどで結合する関数等を与えるトレイト
	pub trait Join {
		/// 要素をコンマで繋いだトークンストリームを生成します。
		fn comma_join(&self) -> TS;
		/// 要素をネストしたタプルにします。全てのタプルが2つのの要素のみからなるようにします
		fn nested_tuple(&self) -> TS;
		/// 要素をタプルにします。要素が1つだけの場合もタプルになります。
		fn tuple(&self) -> TS;
		/// 要素を書き換え可能な形で (各要素を `mut x` として) タプルにします。要素が1つだけの場合もタプルになります。
		fn tuple_mut(&self) -> TS;
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
		fn nested_tuple(&self) -> TS {
			if self.len()==0 { return quote!(); }
			let first = &self[0];
			let mut joined = quote!( #first );
			for item in self.iter().skip(1) {
				joined = quote!( (#joined,#item) );
			}
			joined
		}
		fn tuple(&self) -> TS {
			let mut joined = quote!();
			for item in self.iter() {
				joined = quote!( #joined #item, );
			}
			quote!( (#joined) )
		}
		fn tuple_mut(&self) -> TS {
			let mut joined = quote!();
			for item in self.iter() {
				joined = quote!( #joined mut #item, );
			}
			quote!( (#joined) )
		}
	}

}
use utils::*;



/// 入力データをパースしてデータ構造を得るモジュール
mod input {
	use super::*;

	impl Input {

		pub fn new(ts:TS,exec:EM) -> Self {
			let mut s = Self {
				src: ts_string(&ts,exec),
				debug: false,
				args: vec![],
				reduction: RM::None,
				execution: exec,
				par_cond_bool: vec![],
				par_cond_cfg: vec![
					parse2::<Meta>(quote!( not(disable_parallel_execution) ))
					.unwrap()
				],
				body: None
			};
			s.parse(ts);
			s
		}

		fn parse(&mut self,ts:TS) {
			let mut current:Vec<TT> = Vec::new();

			let mut iter = ts.into_iter();
			loop {
				let tt = match iter.next() {
					Some(x) => x,
					None => break
				};
				match &tt {
					TT::Group(g) => {
						match g.delimiter() {
							TD::Parenthesis => {
								let p = FnParse::parse(&current,g.stream())
								.unwrap_or_else(|| {
									let mut ts = TS::from_iter(current.to_vec());
									ts.extend([TT::from(g.clone())]);
									let s = ts.to_string();
									panic!("パースに失敗しました: {}",s);
								});
								self.match_arg(p);
								current.clear();
								continue;
							},
							TD::Brace => {
								self.body = Some(g.stream());
								continue;
							},
							_ => {
								panic!("パースに失敗しました: {}",g.to_string());
							}
						}
					},
					TT::Punct(p) => {
						if p.as_char()==';' {
							self.body = Some(TS::from_iter(iter));
							break;
						}
					},
					_ => {}
				}
				if self.body.is_some() {
					panic!("ブロックの後には引数を指定できません");
				}
				current.push(tt);
			}

			if self.args.len()==0 { panic!("引数がありません"); }
			if self.body.is_none() { panic!("ボディがありません"); }
		}

		fn match_arg(&mut self,p:FnParse) {
			None
			// $var = index($range)
			.or_else(|| {
				if p.vars.is_empty() { return None }
				if p.name!="index" { return None }

				let var = parse2::<Ident>(p.vars.clone()).ok()?;
				let range = parse2::<ExprRange>(p.args.clone()).ok()?;

				self.args.push(
					Arg::IndexRange { var, range }
				);

				Some(())
			})
			// index($var)
			// $var = index($size)
			.or_else(|| {
				if p.name!="index" { return None }

				let var = parse2::<Ident>(
					match p.vars.is_empty() {
						false => p.vars.clone(),
						true  => p.args.clone()
					}
				).ok()?;
				let size = parse2::<Expr>(p.args.clone()).ok()?;

				self.args.push(
					Arg::IndexInt { var, size }
				);

				Some(())
			})
			// ($var1,$var2,$var3,...) = index($size1,$size2,$size3,...)
			// $var1,$var2,$var3,... = index($size1,$size2,$size3,...)
			.or_else(|| {
				if p.vars.is_empty() { return None }
				if p.name!="index" { return None }

				let vars_vts = split_ts(p.vars.clone());

				let size =
				split_ts(p.args.clone()).into_iter()
				.try_map_collect(|ts| {
					parse2::<Expr>(ts).ok()
				})?;

				let vars = match vars_vts.len() {
					n if n==size.len() => {
						vars_vts.iter()
						.try_map_collect(|ts| {
							parse2::<Ident>(ts.clone()).ok()
						})?
					},
					1 => {
						let et = parse2::<ExprTuple>(p.vars.clone()).ok()?;
						if et.elems.len()!=size.len() { return None; }

						et.elems.iter()
						.try_map_collect(|e| {
							let ep = unwrap_enum!( Expr::Path = e );
							expr_path_to_ident(ep)
						})?
					},
					_ => { return None; }
				};

				self.args.push(
					Arg::IndexMultipleInt { vars, size }
				);

				Some(())
			})
			// each($array)
			// $var = each($array)
			// each(mut $array)
			// $var = each(mut $array)
			// mut each($array)
			// $var = mut each($array)
			// each_nd($array)
			// $var = each_nd($array)
			// each_nd(mut $array)
			// $var = each_nd(mut $array)
			// mut each_nd($array)
			// $var = mut each_nd($array)
			.or_else(|| {
				let (nd,mut mutable) = match &p.name[..] {
					"each" => (false,false),
					"mut each" => (false,true),
					"each_nd" => (true,false),
					"mut each_nd" => (true,true),
					_ => { return None; }
				};

				let cmf = check_mut_flag(p.args.clone());
				if cmf.0 && !mutable { mutable = true; }
				else if cmf.0 && mutable { return None; }

				let (var,dereference) = match p.vars.is_empty() {
					false => {
						let (dt,i) = check_var_deref(p.vars
						.clone())?;
						let d = match (dt,mutable) {
							(DerefType::None,_) => false,
							(DerefType::Deref,false)|(DerefType::DerefMut,true) => true,
							_ => { return None; }
						};
						(i,d)
					},
					true  => {
						let i = parse2::<Ident>(cmf.1.clone()).ok()?;
						(i,false)
					}
				};

				let array = parse2::<Expr>(cmf.1).ok()?;

				self.args.push(
					Arg::Each { nd, mutable, dereference, var, array }
				);

				Some(())
			})
			// reduce($op:$var)
			// fold($op:$var)
			// $inner = reduce($op:$outer)
			// $inner = fold($op:$outer)
			// reduce_assign($op:$var)
			// fold_assign($op:$var)
			// $inner = reduce_assign($op:$outer)
			// $inner = fold_assign($op:$outer)
			.or_else(|| {
				let (mode,assignment) = match &p.name[..] {
					"reduce"        => (RM::Reduce,false),
					"fold"          => (RM::Fold,false),
					"reduce_assign" => (RM::Reduce,true),
					"fold_assign"   => (RM::Fold,true),
					_               => { return None; }
				};
				match (self.reduction,&mode) {
					(RM::None,_)|(RM::Reduce,RM::Reduce)|(RM::Fold,RM::Fold) => {},
					_ => { panic!("reduce と fold を同時には指定できません"); }
				}

				self.reduction = mode;

				let (op,var_outside) = parse_reduction_args(p.args.clone())?;

				let var_inside = match p.vars.is_empty() {
					false => parse2::<Ident>(p.vars.clone()).ok()?,
					true => {
						parse2::<Ident>(var_outside.to_token_stream())
						.ok()?
					}
				};

				self.args.push(
					Arg::Reduction { assignment, operator: op, var_inside, var_outside }
				);

				Some(())
			})
			// par_cond_bool($condition)
			.or_else(|| {
				if !p.vars.is_empty() { return None; }
				if p.name!="par_cond_bool" { return None; }

				self.par_cond_bool.push(
					parse2::<Expr>(p.args.clone()).ok()?
				);

				Some(())
			})
			// par_cond_cfg($condition)
			.or_else(|| {
				if !p.vars.is_empty() { return None; }
				if p.name!="par_cond_cfg" { return None; }

				self.par_cond_cfg.push(
					parse2::<Meta>(p.args.clone()).ok()?
				);

				Some(())
			})
			// debug()
			.or_else(|| {
				if !p.vars.is_empty() { return None; }
				if !p.args.is_empty() { return None; }
				if p.name!="debug" { return None; }

				self.debug = true;

				Some(())
			})
			// どのパターンにもマッチしなかった場合
			.unwrap_or_else(|| {
				let mut src = match p.vars.is_empty() {
					false => format!("{} = ",p.vars),
					true  => String::new()
				};
				src += &p.name;
				src += &format!("({})",p.args);

				panic!("パースに失敗しました: {}",src);
			});
		}

	}

	/// vars = name(args) の型のオプション引数をそれぞれごとに分割したデータ
	struct FnParse {
		pub vars:TS,
		pub name:String,
		pub args:TS
	}

	impl FnParse {
		/// * 生のトークンストリームから `FnParse` 型のデータを生成する
		/// * `prefix` で `vars = name` の箇所のトークンストリームを分割したトークンツリーを受け取り、 `args` で引数 `args` のトークンストリームを受け取る
		pub fn parse(prefix:&Vec<TT>,args:TS) -> Option<Self> {
			let (var_len,name_pos) =
			prefix.iter()
			.position(|tt| {
				if let TT::Punct(p) = tt {
					p.as_char() == '='
				}
				else { false }
			})
			.map_or_else(|| (0,0), |p| (p,p+1) );

			let vars = TS::from_iter(prefix[0..var_len].to_vec());

			let name_ts = &prefix[name_pos..];
			let mut all_ident = true;
			let name = name_ts.into_iter()
			.map_while(|tt| {
				if let TT::Ident(i) = tt { Some(i.to_string()) }
				else {
					all_ident = false;
					None
				}
			})
			.collect::<Vec<_>>()
			.join(" ");
			if !all_ident { return None; }

			Some( Self { vars, name, args } )
		}
	}

	/// 引数のようなコンマ区切りのトークンストリームをコンマで分割して Vec にする
	fn split_ts(ts:TS) -> Vec<TS> {
		let mut whole:Vec<TS> = Vec::new();
		let mut current:Vec<TT> = Vec::new();

		for tt in ts {
			if let TT::Punct(p) = &tt {
				if p.as_char()==',' {
					whole.push(TS::from_iter(current.to_vec()));
					current.clear();
					continue
				}
			}
			current.push(tt);
		}

		if current.len()>0 {
			whole.push(TS::from_iter(current));
		}

		whole
	}

	/// 引数を変数として解析を試み、解析できた場合、参照外しがあるかもチェックする
	fn check_var_deref(ts:TS) -> Option<(DerefType,Ident)> {
		let e = parse2::<Expr>(ts).ok()?;
		match e {
			Expr::Path(ep) => {
				expr_path_to_ident(&ep).map( |i| (DerefType::None,i) )
			},
			Expr::Reference(er) => {
				let ep = unwrap_enum!( Expr::Path = *er.expr );
				expr_path_to_ident(&ep)
				.map(|i| {
					let m = match er.mutability {
						Some(_) => DerefType::DerefMut,
						None => DerefType::Deref
					};
					(m,i)
				})
			},
			_ => { return None; }
		}
	}

	/// `check_var_deref` の返値に用いる参照外しの種類
	enum DerefType {
		/// 参照はない
		None,
		/// 参照外しをしている (`&x`)
		Deref,
		/// ミュータブルな参照外しをしている (`&mut x`)
		DerefMut
	}

	/// 引数に `mut` のフラグがあるかないかを分別し、フラグの有無と、対象の変数を返す
	fn check_mut_flag(ts:TS) -> (bool,TS) {
		let mut iter = ts.into_iter().peekable();
		match iter.peek() {
			None => ( false, TS::new() ),
			Some(TT::Ident(i)) => {
				if i.to_string()=="mut" {
					( true, TS::from_iter(iter.skip(1)) )
				}
				else {
					( false, TS::from_iter(iter) )
				}
			}
			Some(_) => ( false, TS::from_iter(iter) )
		}
	}

	/// `ExprPath` 型から `Ident` 型の取り出しを試みる
	fn expr_path_to_ident(ep:&ExprPath) -> Option<Ident> {
		let seg_list = &ep.path.segments;
		seg_list.last().map(|s| s.ident.clone() )
	}

	/// リダクションの引数をパースする
	fn parse_reduction_args(ts:TS) -> Option<(RO,Expr)> {
		let tokens = ts.clone().into_iter().count();
		if tokens<3 { return None; }
		let mut iter = ts.into_iter();
		let mut captured = TS::new();
		while let Some(tt) = iter.next() {
			if tt.to_string()==":" { break }
			captured = quote!( #captured #tt );
		}
		let op = reduction_op(captured)?;
		let var = parse2::<Expr>(TS::from_iter(iter)).ok()?;
		Some((op,var))
	}

	/// リダクションの演算子を判定する
	fn reduction_op(ts:TS) -> Option<RO> {
		let mut ops = TS::new();
		let mut t:Option<Type> = None;
		for tt in ts {
			if let TT::Group(g) = tt {
				t = Some(parse2::<Type>(g.stream()).ok()?);
				break;
			}
			else { ops = quote!( #ops #tt ); }
		}
		Some( match &ops.to_string()[..] {
			"+"|"add"|"addition"|"sum"|"summation" => RO::Add(t),
			"-"|"sub"|"subtraction" => RO::Sub(t),
			"*"|"mul"|"multiply"|"multiplication"|"prod"|"product" => RO::Mul(t),
			"&&"|"and"|"all" => RO::And,
			"||"|"or"|"any" => RO::Or,
			"&"|"bitand" => RO::BitAnd(t),
			"|"|"bitor" => RO::BitOr(t),
			"^"|"bitxor" => RO::BitXor(t),
			"min"|"minimum" => RO::Min(t),
			"max"|"maximum" => RO::Max(t),
			_ => { return None; }
		} )
	}

	/// デバッグ用に入力されたコードを文字列化する
	fn ts_string(ts:&TS,exec:EM) -> String {
		let ts_str = ts.to_string();
		match exec {
			EM::Serial => format!("for_each! {{\n{}\n}}",ts_str),
			EM::ParallelBridge => format!("par_bridge_for_each! {{\n{}\n}}",ts_str),
			EM::Parallel => format!("par_for_each! {{\n{}\n}}",ts_str)
		}
	}

}



/// 並列化する条件をもとに construct で生成される実行コードを組み合わせるモジュール
mod switcher {
	use super::*;

	impl Input {

		/// 実行コードを構成する。デバッグ出力内容もここで用意する。
		pub fn construct(self) -> TS {
			// 構成は `construct_main` に丸投げする
			let src = self.construct_main();

			if self.debug {
				let src_str = src.to_string();
				eprintln!(
					"The macro code\n------\n{}\n------\nwill be converted to\n------\n{}\n------\n\n",
					self.src,
					src_str
				);
			}

			src
		}

		/// 実行コード構成のメイン部分
		fn construct_main(&self) -> TS {
			let bl = self.par_cond_bool.len();
			let cl = self.par_cond_cfg.len();

			// 直列の場合と、 `par_cond_bool` や `par_cond_cfg` が全く指定されていない場合
			if matches!(self.execution,EM::Serial) || ( bl==0 && cl==0 ) {
				let c = Converted::new(&self,self.execution).construct_whole();
				return quote!( {#c} );
			}

			let b = cond_bool_concat(&self.par_cond_bool);
			let c = cond_cfg_concat(&self.par_cond_cfg);

			let p = Converted::new(&self,self.execution).construct_whole();
			let s = Converted::new(&self,EM::Serial).construct_whole();

			// `par_cond_bool` や `par_cond_cfg` の指定のされ方に合わせて条件分岐する
			match (self.par_cond_bool.len(),self.par_cond_cfg.len()) {
				(0,0) => { unreachable!() },
				(_,0) => {
					quote!(
						if #b { #p }
						else { #s }
					)
				},
				(0,_) => {
					quote!(
						#[cfg(#c)] { #p }
						#[cfg(not(#c))] { #s }
					)
				},
				(_,_) => {
					quote!(
						#[cfg(#c)] {
							if #b { #p }
							else { #s }
						}
						#[cfg(not(#c))] { #s }
					)
				}
			}
		}

	}

	/// `par_cond_bool` をここで1つにまとめる
	fn cond_bool_concat(items:&Vec<Expr>) -> TS {
		match items.len() {
			0 => quote!( false ),
			1 => {
				let item = &items[0];
				quote!( #item )
			},
			_ => {
				let first = &items[0];
				let mut src = quote!( (#first) );
				for item in items.iter().skip(1) {
					src = quote!( #src && (#item) );
				}
				src
			}
		}
	}

	/// `par_cond_cfg` をここで1つにまとめる
	fn cond_cfg_concat(items:&Vec<Meta>) -> TS {
		if items.len()==0 { return quote!(); }

		let first = &items[0];
		let mut src = quote!( #first );
		if items.len()==1 { return src; }
		for item in items.iter().skip(1) {
			src = quote!( #src, #item );
		}
		quote!( all( #src ) )
	}

}



/// input で得たデータ構造をもとに実行コードを構成する各パーツを生成するモジュール
mod converted {
	use super::*;

	impl Converted {

		/// 構造体の生成して、各 `Arg` ごとに構築 (`make_element` に丸投げ)
		pub fn new(input:&Input,exec:EM) -> Self {
			let mut s = Self {
				execution: exec,
				reduction: input.reduction,
				body: TS::from(input.body.as_ref().unwrap().clone()),
				iterators: vec![],
				lambda_args: vec![],
				advance_defs: TS::new(),
				reduction_identities: vec![],
				reduction_vars_inside: vec![],
				reduction_vars_outside: vec![],
				reduction_outside_assignment: TS::new(),
				reduction_func: vec![],
				reduction_func_args_1st: vec![],
				reduction_func_args_2nd: vec![],
				use_into_parallel_iterator: false,
				use_parallel_bridge: false,
				use_parallel_iterator: false,
				use_indexed_parallel_iterator: false,
				use_ndarray_indices: false,
			};
			for arg in input.args.iter() {
				s.make_element(arg);
			}
			s
		}

		/// `Arg` の各アイテムごとに構成要素を構築
		fn make_element(&mut self,arg:&Arg) {
			match arg {
				Arg::IndexMultipleInt {vars,size} => {
					// イテレーションで取り扱いやすくするために、全てのインデクスを1つのインデクスでイテレートし、各ループで元のN成分のインデクスに分けて使う。

					let ip = make_ip_var(vars);

					let size_ts = size.iter()
					.map(|e| e.to_token_stream() )
					.collect::<Vec<_>>();

					let product = product(&size_ts);
					let mut iter = quote!( ( 0..(#product) ) );
					if matches!(self.execution,EM::Parallel) {
						self.use_into_parallel_iterator = true;
						iter = quote!( #iter.into_par_iter() );
					}

					let id = index_decomposition(ip.clone(),&size_ts);
					let vars_t = vars.tuple();
					let ad = quote!( let #vars_t = #id; );

					self.iterators.push(iter);
					self.lambda_args.push(ip.to_token_stream());
					let adw = &mut self.advance_defs;
					*adw = quote!( #adw #ad );
				},
				Arg::IndexInt {var,size} => {
					let mut iter = quote!( ( 0..(#size) ) );
					if matches!(self.execution,EM::Parallel) {
						self.use_into_parallel_iterator = true;
						iter = quote!( #iter.into_par_iter() );
					}

					self.iterators.push(iter);
					self.lambda_args.push(var.to_token_stream());
				},
				Arg::IndexRange {var,range} => {
					let mut iter = quote!( (#range) );
					if matches!(self.execution,EM::Parallel) {
						self.use_into_parallel_iterator = true;
						iter = quote!( #iter.into_par_iter() );
					}

					self.iterators.push(iter);
					self.lambda_args.push(var.to_token_stream());
				},
				Arg::Each {nd,mutable,dereference,var,array} => {
					if self.execution==EM::Parallel {
						self.use_into_parallel_iterator = true;
					}
					let iter = match (self.execution,nd,mutable) {
						(EM::Parallel,false,false) => quote!( (#array).as_slice().into_par_iter() ),
						(EM::Parallel,false,true) => quote!( (#array).as_mut_slice().into_par_iter() ),
						(EM::Parallel,true,false) => quote!( (#array).as_slice().unwrap().into_par_iter() ),
						(EM::Parallel,true,true) => quote!( (#array).as_slice_mut().unwrap().into_par_iter() ),
						(_,_,false) => quote!( (#array).iter() ),
						(_,_,true) => quote!( (#array).iter_mut() )
					};

					let la = match (mutable,dereference) {
						(_,false) => quote!( #var ),
						(false,true) => quote!( &#var ),
						(true,true) => quote!( &mut $var )
					};

					self.iterators.push(iter);
					self.lambda_args.push(la);
				},
				Arg::Reduction {assignment,operator,var_inside,var_outside} => {
					let id = reduction_identity(operator);
					let rfa1 = make_rfa_var1(var_inside);
					let rfa2 = make_rfa_var2(var_inside);
					let rf = reduction_operation(operator,&rfa1,&rfa2);
					let tv = make_tmp_var(var_inside);
					let oa = match *assignment {
						false => reduction_outside_assignment(operator,&tv,var_outside),
						true => quote!( #var_outside = #tv; )
					};

					self.reduction_identities.push(id);
					self.reduction_vars_inside.push(var_inside.clone());
					self.reduction_vars_outside.push(tv);
					let oaw = &mut self.reduction_outside_assignment;
					*oaw = quote!( #oaw #oa );
					self.reduction_func_args_1st.push(rfa1);
					self.reduction_func_args_2nd.push(rfa2);
					self.reduction_func.push(rf);
				},
				_ => { todo!() }
			}
			if self.iterators.len()==0 {
				panic!("イテレーションする項目が1つ以上必要です");
			}
		}

	}

	/// Vec 型のトークンの積を計算する
	fn product<'a,I>(terms:I) -> TS where I: IntoIterator<Item=&'a TS> {
		let mut term = quote!();
		for (i,e) in terms.into_iter().enumerate() {
			if i>0 { term = quote!( #term * (#e) ); }
			else { term = quote!( (#e) ); }
		}
		term
	}

	/// インデクスの積を各インデクスに分解する
	fn index_decomposition(ip:Ident,size:&Vec<TS>) -> TS {
		let mut whole = quote!();
		for (i,e) in size.iter().enumerate() {
			let p = product(&size[(i+1)..]);
			let each = match p.is_empty() {
				false => quote!( #ip / (#p) % (#e) ),
				true => quote!( #ip % (#e) ),
			};
			if i==0 { whole = each; }
			else {
				whole = quote!( #whole, #each );
			}
		}
		quote!( ( #whole ) )
	}

	/// 全てのリダクション項目の単位元をまとめたタプルを生成する
	fn reduction_identity(op:&RO) -> TS {
		match op {
			RO::Add(None)|RO::Sub(None) => quote!( zero() ),
			RO::Add(Some(t))|RO::Sub(Some(t)) => quote!( zero::<#t>() ),
			RO::Mul(None) => quote!( one()  ),
			RO::Mul(Some(t)) => quote!( one::<#t>() ),
			RO::Max(None) => quote!( minimum_value() ),
			RO::Max(Some(t)) => quote!( minimum_value::<#t>() ),
			RO::Min(None) => quote!( maximum_value() ),
			RO::Min(Some(t)) => quote!( maximum_value::<#t>() ),
			RO::BitAnd(None) => quote!( !zero() ),
			RO::BitAnd(Some(t)) => quote!( !zero::<#t>() ),
			RO::BitOr(None)|RO::BitXor(None) => quote!( zero() ),
			RO::BitOr(Some(t))|RO::BitXor(Some(t)) => quote!( zero::<#t>() ),
			RO::And => quote!( true ),
			RO::Or => quote!( false )
		}
	}

	/// 全てのリダクションの演算をまとめたタプルを生成する
	fn reduction_operation(op:&RO,a1:&Ident,a2:&Ident) -> TS {
		match op {
			RO::Add(_)    => quote!( #a1 + #a2 ),
			RO::Sub(_)    => quote!( #a1 + #a2 ),
			RO::Mul(_)    => quote!( #a1 * #a2 ),
			RO::Max(_)    => quote!( #a1.max(#a2) ),
			RO::Min(_)    => quote!( #a1.min(#a2) ),
			RO::And       => quote!( #a1 && #a2 ),
			RO::Or        => quote!( #a1 || #a2 ),
			RO::BitAnd(_) => quote!( #a1 & #a2 ),
			RO::BitOr(_)  => quote!( #a1 | #a2 ),
			RO::BitXor(_) => quote!( #a1 ^ #a2 )
		}
	}

	/// リダクションした結果を最終的に外部変数への代入する文を生成する
	fn reduction_outside_assignment(op:&RO,i:&Ident,o:&Expr) -> TS {
		match op {
			RO::Add(_)    => quote!( #o += #i; ),
			RO::Sub(_)    => quote!( #o -= #i; ),
			RO::Mul(_)    => quote!( #o *= #i; ),
			RO::Max(_)    => quote!( #o.max_assign(#i); ),
			RO::Min(_)    => quote!( #o.min_assign(#i); ),
			RO::And       => quote!( #o.and_assign(#i); ),
			RO::Or        => quote!( #o.or_assign(#i); ),
			RO::BitAnd(_) => quote!( #o &= #i; ),
			RO::BitOr(_)  => quote!( #o |= #i; ),
			RO::BitXor(_) => quote!( #o ^= #i; )
		}
	}

	/// インデクス積の変数を定義する
	fn make_ip_var(vars:&Vec<Ident>) -> Ident {
		let name = format!(
			"ip_{}",
			vars.iter()
			.map(|i| i.to_string() )
			.collect::<Vec<_>>()
			.join("_")
		);
		Ident::new(&name,Span::mixed_site())
	}

	/// リダクションの1つ目の変数を定義する
	fn make_rfa_var1(var:&Ident) -> Ident {
		Ident::new(&format!("a1_{}",var.to_string()),Span::call_site())
	}
	/// リダクションの2つ目の変数を定義する
	fn make_rfa_var2(var:&Ident) -> Ident {
		Ident::new(&format!("a2_{}",var.to_string()),Span::call_site())
	}

	/// リダクション結果を一時的に格納する変数を定義する
	fn make_tmp_var(var:&Ident) -> Ident {
		Ident::new(&format!("tmp_{}",var.to_string()),Span::call_site())
	}

}



/// converted で生成されたパーツを組み合わせて実際の実行コードを生成するモジュール
mod construct {
	use super::*;

	impl Converted {

		/// for_each ループ全体を構築する
		pub fn construct_whole(mut self) -> TS {
			let iter = self.make_iterator();
			let la = self.make_lambda_args();
			let ad = &self.advance_defs;
			let body = &self.body;

			if !matches!(self.execution,EM::Serial) { self.use_parallel_iterator = true; }

			let mut src =
			if matches!(self.reduction,RM::None) {
				quote!(
					#iter.for_each(
						|#la| { #ad #body }
					)
				)
			}
			else {
				let id = self.reduction_identities.tuple();
				let oa = &self.reduction_outside_assignment;
				let tv = self.reduction_vars_outside.tuple();
				let def = self.reduction_vars_inside.tuple_mut();
				let ret = self.reduction_vars_inside.tuple();
				let reduction = self.make_reduction_func();

				match (self.reduction,self.execution) {
					(RM::Fold,EM::Parallel|EM::ParallelBridge) => {
						quote!(
							let #tv =
							#iter.fold(
								|| #id,
								|#la| {
									#ad
									(|| { #body })();
									#ret
								}
							)
							.reduce(
								|| #id,
								#reduction
							);
							#oa
						)
					},
					(RM::Fold,EM::Serial) => {
						quote!(
							let #tv =
							#iter.fold(
								#id,
								|#la| {
									#ad
									(|| { #body })();
									#ret
								}
							);
							#oa
						)
					},
					(RM::Reduce,EM::Parallel|EM::ParallelBridge) => {
						quote!(
							let #tv =
							#iter.map(
								|#la| {
									#ad
									#[allow(unused_mut,unused_assignments)]
									let #def = #id;
									(|| { #body })();
									#ret
								}
							)
							.reduce(
								|| #id,
								#reduction
							);
							#oa
						)
					},
					(RM::Reduce,EM::Serial) => {
						quote!(
							let #tv =
							#iter.map(
								|#la| {
									#ad
									#[allow(unused_mut,unused_assignments)]
									let #def = #id;
									(|| { #body })();
									#ret
								}
							)
							.reduce(#reduction)
							.unwrap();
							#oa
						)
					},
					_ => { unreachable!(); }
				}
			};

			let import = self.make_import();
			src = quote!( #import #src );

			src
		}

		/// 全てのイテレーション項目を突き合わせた (zip) イテレータを生成する
		fn make_iterator(&mut self) -> TS {
			let first = &self.iterators[0];
			let mut zi = quote!( #first );

			if matches!(self.execution,EM::Parallel) {
				if self.iterators.len()>1 {
					self.use_indexed_parallel_iterator = true;
					for i in self.iterators.iter().skip(1) {
						zi = quote!( #zi.zip_eq(#i) );
					}
				}
			}
			else {
				if self.iterators.len()>1 {
					for i in self.iterators.iter().skip(1) {
						zi = quote!( #zi.zip(#i) );
					}
				}
			}

			if matches!(self.execution,EM::ParallelBridge) {
				self.use_parallel_bridge = true;
				zi = quote!( #zi.par_bridge() );
			}

			zi
		}

		/// ループ本体の引数を構築する
		fn make_lambda_args(&self) -> TS {
			let l_args = self.lambda_args.nested_tuple();

			if !matches!(self.reduction,RM::Fold) {
				return l_args;
			}
			else {
				let r_args = self.reduction_vars_inside.tuple_mut();
				match (l_args.is_empty(),r_args.is_empty()) {
					(false,false) => quote!( #r_args, #l_args ),
					(false,true) => quote!( _, #l_args ),
					(true,false) => quote!( #r_args, _ ),
					(true,true) => quote!( _,_ )
				}
			}
		}

		/// リダクションを行う関数を生成する
		fn make_reduction_func(&self) -> TS {
			let a1 = self.reduction_func_args_1st.tuple();
			let a2 = self.reduction_func_args_2nd.tuple();
			let op = self.reduction_func.tuple();
			quote!( |#a1,#a2| { #op } )
		}

		/// for_each を実行するにあたって必要なモジュールがあればインポートする文を生成する
		fn make_import(&self) -> TS {
			let mut src = quote!();
			macro_rules! add {
				( $( $key:ident -> $item:path ),+ $(,)? ) => {
					$( if self.$key {
						src = quote!( #src use $item; )
					} )+
				};
			}
			add!(
				use_into_parallel_iterator ->
					rayon::iter::IntoParallelIterator,
				use_parallel_bridge ->
					rayon::iter::ParallelBridge,
				use_parallel_iterator ->
					rayon::iter::ParallelIterator,
				use_indexed_parallel_iterator ->
					rayon::iter::IndexedParallelIterator,
				use_ndarray_indices ->
					ndarray::indices,
			);
			src
		}

	}

}
