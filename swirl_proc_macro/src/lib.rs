#![deny(warnings)]
#![recursion_limit = "128"]
#![cfg_attr(feature = "nightly", feature(proc_macro_diagnostic))]

extern crate proc_macro;

mod background_job;
mod diagnostic_shim;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, ItemFn};

use diagnostic_shim::*;

#[proc_macro_attribute]
pub fn background_job(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            Span::call_site(),
            "swirl::background_job does not take arguments",
        )
        .to_compile_error()
        .into();
    }

    let item = parse_macro_input!(item as ItemFn);
    emit_errors(background_job::expand(item))
}

fn emit_errors(result: Result<proc_macro2::TokenStream, Diagnostic>) -> TokenStream {
    result
        .map(Into::into)
        .unwrap_or_else(|e| e.to_compile_error().into())
}
