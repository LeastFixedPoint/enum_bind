use syn::{parse_quote, parse2};
use quote::{format_ident, quote};

use crate::Binding;

#[test]
fn test_parse_binding_expr_pat() {
    assert_eq!(
        parse2::<Binding>(quote! { role = "engineer" }).unwrap(),
        Binding::Expr {
            name: format_ident!("role"),
            expr: parse_quote! { "engineer" },
        }
    );
}

#[test]
fn test_parse_binding_only_pat() {
    assert_eq!(
        parse2::<Binding>(quote! { role = Some(_) }).unwrap(),
        Binding::Expr {
            name: format_ident!("role"),
            expr: parse_quote! { Some(_) },
        }
    );
}

#[test]
fn test_parse_binding_only_expr() {
    assert_eq!(
        parse2::<Binding>(quote! { a = 2 + 2 }).unwrap(),
        Binding::Expr {
            name: format_ident!("a"),
            expr: parse_quote! { 2 + 2 },
        }
    );
}
