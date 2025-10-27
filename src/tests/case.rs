use std::collections::BTreeMap;

use quote::format_ident;
use syn::{Variant, parse_quote};

use crate::{Binding, Case, get_cases};

#[test]
fn parse_empty() {
    let variant: &Variant = &parse_quote! {
        #[bind()] FooBar
    };
    let case = get_cases(&variant).unwrap().pop().unwrap();
    assert_eq!(case, Case {
        variant,
        bindings: BTreeMap::new(),
    });
}

#[test]
fn parse_column_value() {
    let variant: &Variant = &parse_quote! {
        #[bind(foo = "value")] FooBar
    };
    let case = get_cases(&variant).unwrap().pop().unwrap();
    assert_eq!(case, Case {
        variant,
        bindings: {
            let mut map = BTreeMap::new();
            map.insert(format_ident!("foo"), Binding::Expr {
                name: format_ident!("foo"),
                expr: parse_quote! { "value" },
            });
            map
        }
    });
}

#[test]
fn parse_column_expr() {
    let variant: &Variant = &parse_quote! {
        #[bind(foo = 2 + 2)] FooBar
    };
    let case = get_cases(&variant).unwrap().pop().unwrap();
    assert_eq!(case, Case {
        variant,
        bindings: {
            let mut map = BTreeMap::new();
            map.insert(format_ident!("foo"), Binding::Expr {
                name: format_ident!("foo"),
                expr: parse_quote! { 2 + 2 },
            });
            map
        }
    });
}

#[test]
fn parse_column_pattern() {
    let variant: &Variant = &parse_quote! {
        #[bind(foo = Some(_))] FooBar
    };
    let case = get_cases(&variant).unwrap().pop().unwrap();
    assert_eq!(case, Case {
        variant,
        bindings: {
            let mut map = BTreeMap::new();
            map.insert(format_ident!("foo"), Binding::Expr {
                name: format_ident!("foo"),
                expr: parse_quote! { Some(_) },
            });
            map
        }
    });
}

#[test]
fn parse_variant_field() {
    let variant: &Variant = &parse_quote! {
        #[bind()] FooBar { foo: usize }
    };
    let case = get_cases(&variant).unwrap().pop().unwrap();
    assert_eq!(case, Case {
        variant,
        bindings: {
            let mut map = BTreeMap::new();
            map.insert(format_ident!("foo"), Binding::Field { name: format_ident!("foo") });
            map
        }
    });
}

#[test]
fn parse_variant_field_override() {
    let variant: &Variant = &parse_quote! {
        #[bind(foo = "bar")] FooBar { foo: usize }
    };
    let cases = get_cases(&variant);
    assert_eq!(cases.is_err(), true);
}

// #[test]
// fn parse_multiple() {
//     let variant: &Variant = &parse_quote! {
//         #[bind(foo = "value", bar = None)]
//         #[bind(bar = Some(19), qux = _, pff = 42)]
//         FooBar { qux: &'static str, bar: Option<usize> }
//     };
//     let cases = get_cases(&variant).unwrap();
//     assert_eq!(cases[0], Relation {
//         variant,
//         columns: {
//             let mut map = BTreeMap::new();
//             map.insert(format_ident!("bar"), Column::Expr {
//                 name: format_ident!("bar"),
//                 expr: parse_quote! { None },
//                 is_field: true,
//             });
//             map.insert(format_ident!("foo"), Column::Expr {
//                 name: format_ident!("foo"),
//                 expr: parse_quote! { "value" },
//             });
//             map.insert(format_ident!("qux"), Column::Expr {
//                 name: format_ident!("qux"),
//                 expr: parse_quote! { _ },
//                 is_field: true,
//             });
//             map
//         }
//     });
//     assert_eq!(cases[1], Relation {
//         variant,
//         columns: {
//             let mut map = BTreeMap::new();
//             map.insert(format_ident!("bar"), Column::Expr {
//                 name: format_ident!("bar"),
//                 expr: parse_quote! { Some(19) },
//                 is_field: true,
//             });
//             map.insert(format_ident!("pff"), Column::Expr {
//                 name: format_ident!("pff"),
//                 expr: parse_quote! { 42 },
//             });
//             map.insert(format_ident!("qux"), Column::Expr {
//                 name: format_ident!("qux"),
//                 expr: parse_quote! { _ },
//                 is_field: true,
//             });
//             map
//         }
//     });
// }