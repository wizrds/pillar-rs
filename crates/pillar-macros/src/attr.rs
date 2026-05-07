use darling::{ast, util, FromDeriveInput, FromField};


#[derive(FromDeriveInput)]
#[darling(attributes(pillar), supports(struct_named))]
pub struct ModelAttrs {
    pub ident: syn::Ident,
    pub data: ast::Data<util::Ignored, FieldAttrs>,
    pub table: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(pillar), supports(struct_named))]
pub struct ViewAttrs {
    pub ident: syn::Ident,
    pub data: ast::Data<util::Ignored, FieldAttrs>,
    pub view: Option<String>,
    pub from: Option<String>,
    pub filter: Option<String>,
}

#[derive(FromField)]
#[darling(attributes(pillar))]
pub struct FieldAttrs {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    pub column: Option<String>,

    #[darling(default)]
    pub primary_key: bool,

    #[darling(default)]
    pub unique: bool,

    #[darling(default)]
    pub skip: bool,
}
