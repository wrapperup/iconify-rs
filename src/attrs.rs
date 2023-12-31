use syn::{Expr, ExprLit, Lit};

pub fn get_lit_str<'a>(attr_name: &'static str, value: &'a Expr) -> syn::Result<&'a syn::LitStr> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit), ..
    }) = &value
    {
        Ok(lit)
    } else {
        Err(syn::Error::new_spanned(
            value,
            format!("expected {attr_name} attribute to be a string: `{attr_name} = \"...\"`"),
        ))
    }
}

pub fn get_lit_bool(attr_name: &'static str, value: &Expr) -> syn::Result<bool> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Bool(lit),
        ..
    }) = &value
    {
        Ok(lit.value())
    } else {
        Err(syn::Error::new_spanned(
            value,
            format!("expected {attr_name} attribute to be a bool value, `true` or `false`: `{attr_name} = ...`"),
        ))?
    }
}
