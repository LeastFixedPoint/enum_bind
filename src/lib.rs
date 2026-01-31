use core::panic;
use std::{collections::BTreeMap, fmt::Debug};

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Data, Error, Expr, Fields, FnArg, Meta, Pat, PatIdent, Result, ReturnType, Signature, Token, Variant, Visibility, parenthesized, parse::{Parse, ParseStream}, parse2, punctuated::Punctuated, spanned::Spanned
};

#[cfg(test)]
pub mod tests;

#[proc_macro_derive(Bind, attributes(query, bind))]
pub fn derive_query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_macro(&syn::parse(input).expect("Failed to parse macro input"))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn impl_macro(ast: &syn::DeriveInput) -> Result<TokenStream> {
    let Data::Enum(data_enum) = &ast.data else { panic!("#[derive(Bind)] only applicable to enums") };

    let mut cases = Vec::new();
    for variant in &data_enum.variants {
        cases.extend(get_cases(variant)?);
    }

    let mut functions = Vec::new();
    for attr in &ast.attrs {
        if !attr.path().is_ident("query") { continue; }
        let spec = parse2::<FunctionSpec>(attr.meta.to_token_stream())?;
        spec.validate(&cases)?;
        functions.push(spec.gen_function(&cases)?);
    }

    let name = &ast.ident;
    let generics = &ast.generics;
    let generic_params = &generics.params;

    let result = quote! {
        impl <#generic_params> #name #generics {
            #(#functions)*
        }
    };

    Ok(result.into())
}

#[derive(Debug, PartialEq, Clone)]
enum OutputMode { Option, Strict, Unwrap, Vec }

impl Parse for OutputMode {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "Option" => Ok(OutputMode::Option),
            "Strict" => Ok(OutputMode::Strict),
            "Unwrap" => Ok(OutputMode::Unwrap),
            "Vec" => Ok(OutputMode::Vec),
            _ => Err(Error::new_spanned(ident, "Expected 'Option', 'Strict', 'Unwrap', or 'Vec'")),
        }
    }
}

#[derive(Debug, PartialEq)]
struct FunctionSpec {
    visibility: Visibility,
    signature: Signature,
    output_mode: OutputMode,
    output_name: Option<Ident>,
}

impl FunctionSpec {
    fn gen_function(&self, cases: &Vec<Case>) -> Result<TokenStream> {
        let body = match self.output_mode {
            OutputMode::Strict => self.gen_body_strict(cases)?,
            OutputMode::Unwrap => self.gen_body_unwrap(cases)?,
            OutputMode::Option => self.gen_body_option(cases)?,
            OutputMode::Vec => self.gen_body_vec(cases)?,
        };

        let visibility = &self.visibility;
        let signature = &self.signature;
        Ok(quote! {
            #visibility #signature {
                #body
            }
        })
    }

    fn gen_body_strict(&self, cases: &Vec<Case>) -> Result<TokenStream> {
        let mut arms = Vec::new();
        for case in cases {
            // TODO: Sink the wildcard/capturing patterns
            let Some(pattern) = self.gen_pattern(case) else { continue; };
            let Some(output) = self.gen_output(case) else { continue; };
            arms.push(quote! { #pattern => #output } );
        }

        let match_expr = &self.gen_match_expr();
        Ok(quote! {
            match #match_expr {
                #(#arms),*
            }
        })
    }

    fn gen_body_unwrap(&self, cases: &Vec<Case>) -> Result<TokenStream> {
        let mut arms = Vec::new();
        for case in cases {
            // TODO: Sink the wildcard/capturing patterns
            let Some(pattern) = self.gen_pattern(case) else { continue; };
            let Some(output) = self.gen_output(case) else { continue; };
            arms.push(quote! { #pattern => #output } );
        }
        arms.push(quote! { value => panic!("Cannot determine what to return for value: {value:?}") });

        let match_expr = &self.gen_match_expr();
        Ok(quote! {
            match #match_expr {
                #(#arms),*
            }
        })
    }

    fn gen_body_option(&self, cases: &Vec<Case>) -> Result<TokenStream> {
        let mut arms = Vec::new();
        for case in cases {
            // TODO: Sink the wildcard/capturing patterns
            let Some(pattern) = self.gen_pattern(case) else { continue; };
            let Some(output) = self.gen_output(case) else { continue; };
            arms.push(quote! { #pattern => #output } );
        }
        arms.push(quote! { _ => None });

        let match_expr = &self.gen_match_expr();
        Ok(quote! {
            match #match_expr {
                #(#arms),*
            }
        })
    }

    fn gen_body_vec(&self, cases: &Vec<Case>) -> Result<TokenStream> {
        let match_expr = &self.gen_match_expr();

        let mut arms = Vec::new();
        for case in cases {
            let Some(pattern) = self.gen_pattern(case) else { continue; };
            let Some(output) = self.gen_output(case) else { continue; };
            arms.push(quote! {
                if let #pattern = #match_expr {
                    result.push(#output);
                }
            });
        }

        Ok(quote! {
            let mut result = Vec::new();
            #(#arms)*
            result
        })
    }

    fn validate(&self, cases: &Vec<Case>) -> Result<()> {
        let function = &self.signature.ident;

        if self.signature.receiver().is_some() {
            let name = self.output_name();
            if !cases.iter().any(|r| r.has_binding(name)) {
                return Err(Error::new_spanned(&name,
                    format!(r#"Function "{function}" must return binding "{name}", but no variant has that binding"#)));
            }
        }

        if self.signature.receiver().is_some() {
            let name = self.output_name();
            if self.output_mode == OutputMode::Strict
            && let Some(case) = cases.iter().find(|r| !r.has_binding(name)) {
                let variant = &case.variant.ident;
                return Err(Error::new_spanned(&name,
                    format!(r#"Cannot determine what function "{function}" should return for variant "{variant}""#)));
            }
        }

        if self.signature.receiver().is_none() {
            for input in &self.signature.inputs {
                let name = fn_arg_to_ident(input);
                if let Some(case) = cases.iter().find(|r| !r.has_binding(name)) {
                    let variant = &case.variant.ident;
                    return Err(Error::new_spanned(&name,
                        format!(r#"Variant "{variant}" does not have binding "{name}", \
                                   cannot determine when function "{function}" should return it"#)));
                }
            }
        }

        if self.signature.receiver().is_some() {
            for input in &self.signature.inputs {
                let FnArg::Typed(pat_type) = input else { continue; };
                let Pat::Ident(PatIdent { ident, .. }) = pat_type.pat.as_ref() else {
                    panic!("Expected a simple function argument");
                };
                for case in cases {
                    match case.bindings.get(ident) {
                        None | Some(Binding::Field { .. }) => continue,
                        _ => {
                            let variant = &case.variant.ident;
                            return Err(Error::new_spanned(ident,
                                format!(r#"Argument "{ident}" in function "{function}"
                                           conflicts with a binding of the same name in variant "{variant}""#)));
                        },
                    }
                }
            }
        }

        if let ReturnType::Default = &self.signature.output {
            return Err(Error::new_spanned(&self.signature, "Function must have a return type"));
        }

        Ok(())
    }

    fn gen_match_expr(&self) -> TokenStream {
        if let Some(receiver) = self.signature.receiver() {
            if receiver.reference.is_some() {
                quote! { *self }
            } else {
                quote! { self }
            }
        } else {
            let arg_names: Vec<_> = self.signature.inputs.iter().map(fn_arg_to_ident).collect();
            quote! { (#(#arg_names),*) }
        }
    }

    fn gen_pattern(&self, case: &Case) -> Option<TokenStream> {
        if self.signature.receiver().is_some() {
            return Some(case.gen_self_expr());
        }

        let mut patterns = Vec::new();

        for input in &self.signature.inputs {
            let name = fn_arg_to_ident(input);
            use Binding::*;
            let pattern = match case.bindings.get(name) {
                Some(Field { name }) => quote! { #name @ _ },
                Some(Expr { name, expr }) => quote! { #name @ #expr },
                Some(Never { .. }) => return None,
                None => quote! { #name @ _ },
            };
            patterns.push(pattern);
        }
        Some(quote! { (#(#patterns),*) })
    }

    fn gen_output(&self, case: &Case) -> Option<TokenStream> {
        let output = if self.signature.receiver().is_none() {
            Some(case.gen_self_expr())
        } else {
            use Binding::*;
            match case.bindings.get(self.output_name()) {
                Some(Field { name }) => Some(quote! { #name }),
                Some(Expr { expr, .. }) => Some(quote! { #expr }),
                Some(Never { .. }) | None => None,
            }
        };

        use OutputMode::*;
        
        match (output, &self.output_mode) {
            (Some(output), Option)                => Some(quote! { Some( #output ) }),
            (Some(output), Strict | Unwrap | Vec) => Some(output),
            (None,         Option)                => Some(quote! { None }),
            (None,         Unwrap)                => Some(quote! { panic!("Cannot determine what to return for this variant") }),
            (None,         Strict | Vec)          => None,
        }
    }

    fn output_name(&self) -> &Ident {
        self.output_name.as_ref().unwrap_or(&self.signature.ident)
    }
}

impl Parse for FunctionSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        if ident.to_string() != "query" { return Err(Error::new_spanned(ident, "Expected 'query'")); }
        let input = { let content; parenthesized!(content in input); content };
        let visibility = input.parse::<Visibility>()?;
        let signature = input.parse::<Signature>()?;
        let output_mode: OutputMode;
        let output_name: Option<Ident>;
        if input.is_empty() {
            output_mode = OutputMode::Option;
            output_name = None;
        } else {
            input.parse::<Token![,]>()?;
            input.parse::<Token![return]>()?;
            input.parse::<Token![=]>()?;
            output_mode = input.parse::<OutputMode>()?;
            if input.is_empty() {
                output_name = None;
            } else {
                let input = { let content; parenthesized!(content in input); content };
                output_name = Some(input.parse::<Ident>()?);
            }
        }

        Ok(FunctionSpec { visibility, signature, output_mode, output_name })
    }
}

// fn infer_output(signature: &Signature) -> Result<Argument> {
//     let ReturnType::Type(_, ty) = &signature.output else {
//         return Err(Error::new_spanned(signature, "Function must have a return type"));
//     };
//     if let Type::Path(type_path) = ty.as_ref() {
//         if type_path.path.is_ident("Self") {
//             return Ok(Argument::SelfArg { reference: false });
//         }
//         if type_path.path.segments.len() == 1
//         && (type_path.path.segments[0].ident == "Option" || type_path.path.segments[0].ident == "Vec")
//         && let PathArguments::AngleBracketed(args) = &type_path.path.segments[0].arguments
//         && args.args.len() == 1 
//         && let GenericArgument::Type(Type::Path(inner_type_path)) = &args.args[0]
//         && inner_type_path.path.is_ident("Self") {
//             return Ok(Argument::SelfArg { reference: false });
//         };
//     }
//     return Ok(Argument::Named(signature.ident.clone()));
// }

#[derive(Clone, PartialEq)]
enum Binding {
    Field { name: Ident },
    Expr { name: Ident, expr: Expr },
    Never { name: Ident  },
}

impl Binding {
    fn name(&self) -> &Ident {
        match self {
            Binding::Field { name } => name,
            Binding::Expr { name, .. } => name,
            Binding::Never { name } => name,
        }
    }
}

impl Debug for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::Field { name } => f.debug_tuple("Field")
                .field(&name.to_string())
                .finish(),
            Binding::Expr { name, expr } => f.debug_struct("Expr")
                .field("name", &name.to_string())
                .field("expr", &expr.to_token_stream().to_string())
                .finish(),
            Binding::Never { name } => f.debug_tuple("Never")
                .field(&name.to_string())
                .finish(),
        }
    }
}

impl Parse for Binding {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            return Ok(Binding::Never { name });
        }
        let expr = Expr::parse(input)?;
        Ok(Binding::Expr { name, expr })
    }
}

#[derive(PartialEq)]
struct Case<'v> {
    variant: &'v Variant,
    bindings: BTreeMap<Ident, Binding>,
}

impl Case<'_> {
    fn gen_self_expr(&self) -> TokenStream {
        let variant_name = &self.variant.ident;
                
        match self.variant.fields {
            Fields::Named(_) => {
                let mut field_exprs = Vec::new();
                for field in self.variant.fields.iter() {
                    let Some(field_name) = &field.ident else {
                        panic!("Named field in variant {variant_name} does not have a name: {field:?}");
                    };
                    field_exprs.push(quote! { #field_name });
                }
                quote! { #variant_name { #(#field_exprs),* } }
            },
            Fields::Unnamed(_) => {
                let mut field_exprs = Vec::new();
                for (i, field) in self.variant.fields.iter().enumerate() {
                    let field_ident = format_ident!("_{i}", span = field.span());
                    field_exprs.push(quote! { #field_ident });
                }
                quote! { #variant_name ( #(#field_exprs),* ) }
            },
            Fields::Unit => {
                quote! { Self::#variant_name }
            },
        }
    }
}

impl Debug for Case<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Case")
            .field("variant", &self.variant.to_token_stream().to_string())
            .field("bindings", &self.bindings)
            .finish()
    }
}

impl Case<'_> {
    fn has_binding(&self, name: &Ident) -> bool {
        self.bindings.contains_key(name)
    }
}

fn get_field_name((i, field): &(usize, &syn::Field)) -> Ident {
    if let Some(ident) = &field.ident {
        return ident.clone()
    }
    return format_ident!("_{i}", span = field.span());
}

fn fn_arg_to_ident(arg: &FnArg) -> &Ident {
    let FnArg::Typed(pat_type) = arg else { panic!("Expected a simple function argument"); };
    let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else { panic!("Expected a simple function argument"); };
    &pat_ident.ident
}

fn get_cases(variant: &Variant) -> Result<Vec<Case<'_>>> {
    let variant_name = &variant.ident;

    let mut bindings_from_fields = BTreeMap::new();

    for field in variant.fields.iter().enumerate() {
        let field_name = get_field_name(&field);
        let binding = Binding::Field { name: field_name.clone() };
        bindings_from_fields.insert(field_name, binding);
    }

    let mut cases = Vec::new();
    for attr in &variant.attrs {
        let mut bindings = bindings_from_fields.clone();
        
        if attr.path().is_ident("bind") {
            let Meta::List(meta_list) = &attr.meta else { 
                return Err(Error::new_spanned(&attr, "Expected a list of bind = value pairs inside #[bind(...)]"));
            };
            let parser = Punctuated::<Binding, Token![,]>::parse_terminated;
            for binding in meta_list.parse_args_with(parser)? {
                let binding_name = binding.name();
                if variant.fields.iter().any(|f| f.ident.as_ref() == Some(binding_name)) {
                    return Err(Error::new_spanned(&binding_name, format!(r#"Variant "{variant_name}" already has a field named "{binding_name}"; #[bind(...)] cannot redefine fields"#)));
                };
                bindings.insert(binding_name.clone(), binding);
            }
        }

        cases.push(Case { variant, bindings: bindings });
    }

    if cases.is_empty() {
        cases.push(Case { variant, bindings: bindings_from_fields });
    }

    Ok(cases)
}
