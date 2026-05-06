use proc_macro2::TokenStream;
use quote::quote;


pub fn parse_condition(input: &str) -> syn::Result<TokenStream> {
    let input = input.trim();

    if let Some((left, right)) = split_binary(input, " AND ") {
        let l = parse_condition(left)?;
        let r = parse_condition(right)?;
        return Ok(quote! { ::pillar::condition::ConditionExpression::And(
            ::std::boxed::Box::new(#l),
            ::std::boxed::Box::new(#r),
        )});
    }

    if let Some((left, right)) = split_binary(input, " OR ") {
        let l = parse_condition(left)?;
        let r = parse_condition(right)?;
        return Ok(quote! { ::pillar::condition::ConditionExpression::Or(
            ::std::boxed::Box::new(#l),
            ::std::boxed::Box::new(#r),
        )});
    }

    for (op_str, variant) in OPERATORS {
        if let Some(pos) = input.find(op_str) {
            let col = input[..pos].trim().to_string();
            let val_str = input[pos + op_str.len()..].trim();
            let val = parse_value(val_str)?;
            let variant: TokenStream = variant.parse().unwrap();
            return Ok(quote! {
                ::pillar::condition::ConditionExpression::#variant(
                    #col.to_string(),
                    #val,
                )
            });
        }
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("could not parse where clause: `{input}`"),
    ))
}

// Operators ordered longest-first to avoid `>` matching before `>=`
const OPERATORS: &[(&str, &str)] = &[
    (">=", "Gte"),
    ("<=", "Lte"),
    ("!=", "Ne"),
    ("=",  "Eq"),
    (">",  "Gt"),
    ("<",  "Lt"),
];

fn split_binary<'a>(input: &'a str, op: &str) -> Option<(&'a str, &'a str)> {
    input.find(op).map(|pos| (&input[..pos], &input[pos + op.len()..]))
}

fn parse_value(s: &str) -> syn::Result<TokenStream> {
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        let inner = &s[1..s.len() - 1];
        return Ok(quote! { ::pillar::value::Value::String(#inner.to_string()) });
    }

    if let Ok(v) = s.parse::<i64>() {
        return Ok(quote! { ::pillar::value::Value::Int64(#v) });
    }

    if let Ok(v) = s.parse::<f64>() {
        return Ok(quote! { ::pillar::value::Value::Float64(#v) });
    }

    if s == "true" {
        return Ok(quote! { ::pillar::value::Value::Boolean(true) });
    }

    if s == "false" {
        return Ok(quote! { ::pillar::value::Value::Boolean(false) });
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("could not parse value: `{s}`"),
    ))
}
