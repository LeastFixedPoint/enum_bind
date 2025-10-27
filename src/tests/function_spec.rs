use syn::parse_quote;

use crate::{FunctionSpec, OutputMode};

#[test]
fn self_to_arg() {
    let spec: FunctionSpec = parse_quote! {
        query(pub fn foo(self) -> i32, Strict)
    };
    assert_eq!(spec, FunctionSpec {
        visibility: parse_quote! { pub },
        signature: parse_quote! { fn foo(self) -> i32 },
        output_mode: OutputMode::Strict,
        output_name: None,
    });
}

#[test]
fn arg_to_self() {
    let spec: FunctionSpec = parse_quote! {
        query(pub fn from_foo(foo: i32) -> Self, Strict)
    };
    assert_eq!(spec, FunctionSpec {
        visibility: parse_quote! { pub },
        signature: parse_quote! { fn from_foo(foo: i32) -> Self },
        output_mode: OutputMode::Strict,
        output_name: None,
    });
}

#[test]
fn self_and_arg_to_arg() {
    let spec: FunctionSpec = parse_quote! {
        query(pub fn check_foo(&self, foo: i32) -> Option<i32>, Option(foo))
    };
    assert_eq!(spec, FunctionSpec {
        visibility: parse_quote! { pub },
        signature: parse_quote! { fn check_foo(&self, foo: i32) -> Option<i32> },
        output_mode: OutputMode::Option,
        output_name: Some(parse_quote! { foo }),
    });
}