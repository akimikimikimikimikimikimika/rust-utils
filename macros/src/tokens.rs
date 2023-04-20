use proc_macro::TokenStream;

pub fn print_tokens(attr:TokenStream,item:TokenStream) -> TokenStream {
	use syn::{parse,Meta,Expr,Lit};

	let mut dst = PrintTo::Stderr;

	(|| {
		macro_rules! unwrap_enum {
			( $case:path = $input:expr ) => {
				match $input {
					$case(x) => x,
					_ => { return }
				}
			};
		}
		let meta = unwrap_enum!( Ok = parse::<Meta>(attr) );
		match meta {
			Meta::Path(p) => {
				let i = unwrap_enum!( Some = p.get_ident() ).to_string();
				match &i[..] {
					"stdout" => { dst = PrintTo::Stdout },
					"stderr" => { dst = PrintTo::Stderr },
					_ => return
				}
			},
			Meta::NameValue(nv) => {
				let i = unwrap_enum!( Some = nv.path.get_ident() ).to_string();
				if i!="file" { return }
				let el = unwrap_enum!( Expr::Lit = &nv.value );
				let ls = unwrap_enum!( Lit::Str = &el.lit );
				dst = PrintTo::File(ls.value());
			},
			_ => return
		}
	})();

	let item_clone = item.clone();

	let src = format!("print_tokens\n{}\n",ts_description(item,0));
	match &dst {
		PrintTo::Stdout => { print!("{}",src); },
		PrintTo::Stderr => { eprint!("{}",src); },
		PrintTo::File(path) => {
			use std::{fs::OpenOptions,io::Write};
			let mut io = OpenOptions::new()
			.write(true).create(true).truncate(true).open(path)
			.expect(&format!("ファイル {} が開けませんでした",path));
			io.write(src.as_bytes())
			.expect(&format!("ファイル {} に書き込めませんでした",path));
		}
	}

	item_clone
}

enum PrintTo {
	Stdout, Stderr, File(String)
}

pub fn stringify_tokens(item:TokenStream) -> TokenStream {
	use proc_macro::{TokenTree,Literal};

	let src = format!("stringify_token\n{}\n",ts_description(item,0));
	TokenStream::from(
		TokenTree::Literal(
			Literal::string(&src)
		)
	)
}

fn ts_description(ts:TokenStream,offset:usize) -> String {
	use proc_macro::{TokenTree,Delimiter};

	let space = " ".repeat(offset);

	ts.into_iter()
	.map(|tt| {
		match tt {
			TokenTree::Ident(i) => {
				format!("{}ident: {}",space,i.to_string())
			},
			TokenTree::Literal(l) => {
				format!("{}liter: {}",space,l.to_string())
			},
			TokenTree::Punct(p) => {
				format!("{}punct: {}",space,p.to_string())
			},
			TokenTree::Group(g) => {
				let s = g.stream();
				let inner = ts_description(s,offset+1);
				match g.delimiter() {
					Delimiter::Parenthesis => {
						format!("{}group (\n{}\n{})",space,inner,space)
					},
					Delimiter::Brace => {
						format!("{}group {{\n{}\n{}}}",space,inner,space)
					},
					Delimiter::Bracket => {
						format!("{}group (\n{}\n{})",space,inner,space)
					},
					Delimiter::None => {
						format!("{}group\n{}",space,inner)
					}
				}
			}
		}
	})
	.collect::<Vec<_>>()
	.join("\n")
}
