use proc_macro::TokenStream as TS1;
use proc_macro2::{
	Span,
	TokenStream as TS,
	TokenTree as TT,
	Delimiter
};
use syn::{
	parse2,
	Expr,Ident
};
use quote::{quote,ToTokens};



macro_rules! smart_for_each_interface {
	() => {
		#[proc_macro]
		pub fn smart_for_each(item:TokenStream) -> TokenStream {
			//! ## `smart_for_each!`
			//! * `par_for_each` の拡張として現在のサブステップに合わせて一部の場の量に対してのみイテレーションを実行します。
			//! * 基本的には `par_for_each` とオプションの指定の仕方は同じです。
			//!
			//! ### 使い方
			//!
			//! ```rust
			//! smart_for_each! {
			//! 	each_nd(a1)
			//! 	each_nd(a2)
			//! 	whole_nd(a3)
			//! 	arms(
			//! 		None -> φ c T
			//! 		φ    -> φ
			//! 		c    -> c
			//! 		T    -> T
			//! 	)
			//! {
			//! 	a1; // a1 の各要素に対してアクセスする
			//! 	*a2; // a2 の各要素に対してアクセスする
			//! 	a3; // a3 のある変数場全体の2次元配列にアクセスする
			//! } }
			//! ```
			//! * `arms` オプションでサブステップ (→左側) ごとに計算する対象となる変数を指定 (→右側) します
			//! * `each_nd` で対象となる変数に対してセル1つずつイテレートします
			//! * `whole_nd` で対象となる変数の全体へのアクセスを提供します
			//! * ビルド時にマクロ展開した結果 (`par_for_each!` は展開しない) を表示するには `debug()` を使用します。
			smart_for_each_impl(item)
		}
	};
}
pub(crate) use smart_for_each_interface;



/// smart_for_each! 実装のエントリポイント
pub fn smart_for_each_impl(ts:TS1) -> TS1 {
	let ts = Input::new(TS::from(ts)).construct();
	TS1::from(ts)
}



mod typedef {
	use super::*;

	/// 与えられた情報をパースしたデータ
	pub struct Input {
		/// 入力したマクロのコード
		pub src: String,
		/// `debug()` がオプションに入っていて、デバッグ出力するかどうか
		pub debug: bool,
		/// 変数による分岐のデータ
		pub arms: Vec<Arm>,
		/// 引数のリスト
		pub args: Vec<Arg>,
		/// `for_each` で実行される内容
		pub body: Option<TS>
	}

	pub enum Arg {
		Index {
			t: Option<Ident>,
			i: Ident,
			j: Ident,
			size: Expr
		},
		Nd {
			var: Ident,
			array: Expr,
			mutable: bool,
			each: bool
		}
	}

	pub struct Arm {
		pub ss: Substep,
		pub vars: Vec<Var>
	}

	macro_rules! variables {
		( $( $var:ident : $index:literal )+ ) => {

			#[allow(non_camel_case_types)]
			#[derive(Clone,Copy,Debug)]
			/// 変数を表す
			pub enum Var { $(
				#[doc=concat!("変数 ",stringify!($var)," を表します")]
				$var,
			)+ }

			impl Var {
				pub fn from_str(s:&str) -> Option<Self> {
					match s {
						$( stringify!($var) => Some(Self::$var), )+
						_ => None
					}
				}
				pub fn index_ident(&self) -> Ident {
					Ident::new(
						match self {
							$( Self::$var => $index, )+
						},
						Span::call_site()
					)
				}
			}

		};
	}

	macro_rules! ss {
		( $( $var:ident )+ ) => {

			#[allow(non_camel_case_types)]
			#[derive(Clone,Copy,Debug)]
			/// サブ段階を表す
			pub enum Substep {
				$( $var, )+
			}

			impl Substep {
				pub fn from_str(s:&str) -> Option<Self> {
					match s {
						$( stringify!($var) => Some(Self::$var), )+
						_ => None
					}
				}
			}

			pub type SS = Substep;

		};
	}

	variables! {
		φ: "iφ"
		ν: "iν"
		c: "ic"
		μ: "iμ"
		T: "iT"
		e: "ie"
	}

	ss! { φ c T None }

}
use typedef::*;



mod input {
	use super::*;

	impl Input {

		pub fn new(ts:TS) -> Self {
			let mut s = Self {
				src: ts_string(&ts),
				debug: false,
				arms: vec![
					Arm {
						ss: SS::None,
						vars: vec![Var::φ,Var::ν,Var::c,Var::μ,Var::T,Var::e]
					},
					Arm {
						ss: SS::φ,
						vars: vec![Var::φ,Var::ν]
					},
					Arm {
						ss: SS::c,
						vars: vec![Var::c,Var::μ]
					},
					Arm {
						ss: SS::T,
						vars: vec![Var::T,Var::e]
					},
				],
				args: vec![],
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
							Delimiter::Parenthesis => {
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
							Delimiter::Brace => {
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
			// each_nd($array)
			// $var = each_nd($array)
			// each_nd(mut $array)
			// $var = each_nd(mut $array)
			// mut each_nd($array)
			// $var = mut each_nd($array)
			// whole_nd($array)
			// $var = whole_nd($array)
			.or_else(|| {
				let mut mutable = false;
				let mut each = true;
				match &p.name[..] {
					"each_nd" => {},
					"mut each_nd" => { mutable = true; },
					"whole_nd" => { each = false; },
					_ => { return None; }
				}

				let cmf = check_mut_flag(p.args.clone());
				if cmf.0 && !mutable { mutable = true; }
				else if cmf.0 && mutable { return None; }

				let var = parse2::<Ident>(
					match p.vars.is_empty() {
						false => p.vars.clone(),
						true  => cmf.1.clone()
					}
				).ok()?;

				let array = parse2::<Expr>(cmf.1).ok()?;

				self.args.push(
					Arg::Nd { mutable, each, var, array }
				);

				Some(())
			})
			// $t,$i,$j = index($size_tuple)
			// $i,$j = index($size_tuple)
			.or_else(|| {
				if p.vars.is_empty() { return None }
				if p.name!="index" { return None }

				let vars_vts = split_ts(p.vars.clone());
				let (t,i,j) = match vars_vts.len() {
					3 => (
						Some( parse2::<Ident>(vars_vts[0].clone()).ok()? ),
						parse2::<Ident>(vars_vts[1].clone()).ok()?,
						parse2::<Ident>(vars_vts[2].clone()).ok()?,
					),
					2 => (
						None,
						parse2::<Ident>(vars_vts[0].clone()).ok()?,
						parse2::<Ident>(vars_vts[1].clone()).ok()?,
					),
					_ => { return None; }
				};

				let size = parse2::<Expr>(p.args.clone()).ok()?;

				self.args.push(
					Arg::Index { t, i, j, size }
				);

				Some(())
			})
			// arms(...)
			.or_else(|| {
				if !p.vars.is_empty() { return None }
				if p.name!="arms" { return None }

				parse_arm(p.args.clone())
				.map(|arms| {
					self.arms = arms;
				})
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

	/// arm のパースを行う
	fn parse_arm(ts:TS) -> Option<Vec<Arm>> {
		let mut step:u8 = 0;
		let mut ss:Option<SS> = None;
		let mut vars:Vec<Var> = vec![];
		let mut arms:Vec<Arm> = vec![];

		for tt in ts {
			let s = tt.to_string();
			match (step,tt,&s[..]) {
				(0,TT::Ident(_),s) => {
					step = 1;
					ss = SS::from_str(s);
					if ss.is_none() { return None; }
				},
				(1,TT::Punct(_),"-") => { step = 2; },
				(2,TT::Punct(_),">") => { step = 3; },
				(3,TT::Ident(_),s) => {
					if let Some(v) = Var::from_str(s) {
						vars.push(v);
					}
					else { return None; }
				},
				(3,TT::Punct(_),",") => {
					step = 0;
					arms.push(
						Arm { ss: ss.unwrap(), vars: vars.clone() }
					);
					ss = None;
					vars.clear();
				},
				_ => { return None; }
			}
		}
		if ss.is_some() {
			arms.push(
				Arm { ss: ss.unwrap(), vars: vars.clone() }
			);
		}

		Some(arms)
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

	/// デバッグ用に入力されたコードを文字列化する
	fn ts_string(ts:&TS) -> String {
		format!("smart_for_each! {{\n{}\n}}",ts)
	}

}



mod construct {
	use super::*;
	use std::iter::zip;

	impl Input {

		pub fn construct(self) -> TS {

			let mut arms = quote!();
			for arm in self.arms.iter() {
				let ss = Ident::new(
					&format!("{:?}",arm.ss),Span::call_site()
				);
				let contents = self.convert(arm);
				arms = quote!(
					#arms
					Substep::#ss => { #contents },
				);
			}
			arms = quote!( {
				#[allow(non_snake_case)]
				match lm_status.lock().unwrap().sub_step { #arms }
			} );

			if self.debug {
				let src = arms.to_string();
				eprintln!(
					"The macro code\n------\n{}\n------\nwill be converted to\n------\n{}\n------\n\n",
					self.src, src
				);
			}

			arms
		}

		fn convert(&self,arm:&Arm) -> TS {
			if arm.vars.len()==0 {
				return quote!( panic!() );
			}

			let mut var_def = quote!();
			let mut args = quote!();
			let mut body_frac = (0..arm.vars.len()).map(|_| quote!() ).collect::<Vec<_>>();

			let index_joined =
				arm.vars.iter()
				.map(|v| v.index_ident() )
				.collect::<Vec<_>>().slice();

			for arg in self.args.iter() {
				match arg {
					Arg::Index {t:to,i,j,size} => {
						args = quote!( #args
							#i,#j = index(#size.0,#size.1)
						);
						if let Some(t) = to {
							for (b,v) in zip(body_frac.iter_mut(),arm.vars.iter()) {
								let iv = v.index_ident();
								*b = quote!( #b
									let #t = #iv;
								);
							}
						}
					},
					Arg::Nd {var,array,mutable,each} => {

						let split_list =
							arm.vars.iter()
							.map(|v| {
								Ident::new(
									&format!("{}_{:?}",var,v),
									Span::call_site()
								)
							})
							.collect::<Vec<_>>();

						var_def = match mutable {
							false => {
								let split_joined = split_list.slice();
								quote!( #var_def
									let #split_joined = #array.split_to_a2(#index_joined);
								)
							},
							true => {
								let split_joined = split_list.slice_mut();
								quote!( #var_def
									let #split_joined = #array.split_to_a2_mut(#index_joined);
								)
							},
						};

						if *each {
							for s in split_list.iter() {
								args = match mutable {
									false => quote!( #args each_nd(#s) ),
									true => quote!( #args each_nd(mut #s) )
								};
							}
						}

						for (b,s) in zip(body_frac.iter_mut(),split_list.iter()) {
							*b = quote!( #b let #var = #s; );
						}

					}
				}
			}

			let body = TS::from(
				self.body.as_ref().unwrap().clone()
			);
			let mut whole_body = quote!();
			for b in body_frac {
				whole_body = quote!(
					#whole_body
					(|| { #b #body })();
				);
			}

			quote!(
				#var_def
				par_for_each! { #args; #whole_body }
			)
		}

	}

}



/// トークンツリーの Vec 型をコンマ区切りなどで結合する関数等を与えるトレイト
trait Join {
	/// 要素をスライスにします。要素が1つだけの場合もタプルになります。
	fn slice(&self) -> TS;
	/// 要素を書き換え可能な形で (各要素を `mut x` として) スライスにします。要素が1つだけの場合もタプルになります。
	fn slice_mut(&self) -> TS;
}
impl<T> Join for Vec<T> where T: ToTokens {
	fn slice(&self) -> TS {
		let mut joined = quote!();
		for item in self.iter() {
			joined = quote!( #joined #item, );
		}
		quote!( [#joined] )
	}
	fn slice_mut(&self) -> TS {
		let mut joined = quote!();
		for item in self.iter() {
			joined = quote!( #joined mut #item, );
		}
		quote!( [#joined] )
	}
}
