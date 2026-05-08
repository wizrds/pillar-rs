#[allow(unused_extern_crates)]
extern crate self as pillar_macros;

mod attr;
mod column;
mod condition;
mod model;
mod view;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};


/// Derives [`pillar::model::Model`](pillar_core::model::Model) for a struct, generating a companion
/// `Model` struct, `Column` struct, and typed column accessors in the same scope.
#[proc_macro_derive(Model, attributes(pillar))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    model::derive(parse_macro_input!(input as DeriveInput)).into()
}

/// Derives [`pillar::view::View`](pillar_core::view::View) for a struct, generating a companion
/// `View` struct, `Column` struct, and typed column accessors in the same scope.
#[proc_macro_derive(View, attributes(pillar))]
pub fn derive_view(input: TokenStream) -> TokenStream {
    view::derive(parse_macro_input!(input as DeriveInput)).into()
}
