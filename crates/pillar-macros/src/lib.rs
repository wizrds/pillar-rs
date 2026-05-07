#[allow(unused_extern_crates)]
extern crate self as pillar_macros;

mod attr;
mod column;
mod condition;
mod model;
mod view;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(Model, attributes(pillar))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    model::derive(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(MaterializedView, attributes(pillar))]
pub fn derive_materialized_view(input: TokenStream) -> TokenStream {
    view::derive(parse_macro_input!(input as DeriveInput)).into()
}
