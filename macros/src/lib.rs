#![allow(confusable_idents)]

use proc_macro::TokenStream;

mod tokens;

mod for_each;
use for_each::*;

mod smart_for_each;
use smart_for_each::*;

mod compose_struct;
use compose_struct::*;

#[proc_macro_attribute]
pub fn print_tokens(attr:TokenStream,item:TokenStream) -> TokenStream {
	tokens::print_tokens(attr,item)
}

#[proc_macro]
pub fn stringify_tokens(item:TokenStream) -> TokenStream {
	tokens::stringify_tokens(item)
}

for_each_interface! {
	par_for_each        Parallel
	par_bridge_for_each ParallelBridge
	for_each            Serial
}

smart_for_each_interface! {}

compose_struct_interface! {}
