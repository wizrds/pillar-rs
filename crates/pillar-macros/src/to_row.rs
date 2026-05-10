use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attr::FieldAttrs;


#[derive(darling::FromDeriveInput)]
#[darling(attributes(pillar), supports(struct_named))]
struct ToRowAttrs {
    ident: syn::Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttrs>,
}

pub fn derive(input: DeriveInput) -> TokenStream {
    match ToRowAttrs::from_derive_input(&input) {
        Err(e) => e.write_errors(),
        Ok(attrs) => impl_to_row(attrs),
    }
}

pub fn impl_to_row_for(ident: &syn::Ident, fields: &[FieldAttrs]) -> TokenStream {
    let row_fields = fields.iter().filter(|f| !f.skip).map(|f| {
        let field_ident = f.ident.as_ref().unwrap();

        quote! {
            ::pillar::value::Value::from(self.#field_ident.clone())
        }
    });

    quote! {
        impl ::pillar::convert::ToRow for #ident {
            fn to_row(&self) -> ::std::vec::Vec<::pillar::value::Value> {
                vec![#(#row_fields),*]
            }
        }
    }
}

fn impl_to_row(attrs: ToRowAttrs) -> TokenStream {
    let fields = attrs.data.take_struct().unwrap().fields;

    impl_to_row_for(&attrs.ident, &fields)
}
