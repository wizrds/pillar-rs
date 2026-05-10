use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;


pub fn derive(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::pillar::convert::FromBatch for #ident #ty_generics #where_clause {
            fn from_batch(
                batch: ::pillar::__private::arrow::record_batch::RecordBatch,
            ) -> ::std::result::Result<::std::vec::Vec<Self>, ::pillar::errors::Error> {
                ::pillar::__private::serde_arrow::from_record_batch(&batch)
                    .map_err(|e| ::pillar::errors::Error::serialization(e.to_string()))
            }
        }
    }
}

/// Generates a `FromBatch` impl for the given identifier with no generics.
/// Used internally by the `Model` and `View` derive macros.
pub fn derive_for(ident: &syn::Ident) -> TokenStream {
    quote! {
        impl ::pillar::convert::FromBatch for #ident {
            fn from_batch(
                batch: ::pillar::__private::arrow::record_batch::RecordBatch,
            ) -> ::std::result::Result<::std::vec::Vec<Self>, ::pillar::errors::Error> {
                ::pillar::__private::serde_arrow::from_record_batch(&batch)
                    .map_err(|e| ::pillar::errors::Error::serialization(e.to_string()))
            }
        }
    }
}
