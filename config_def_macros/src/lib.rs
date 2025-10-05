extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, ExprPath, Fields, GenericArgument, Lit, MetaNameValue, PathArguments,
    Type, parse_macro_input, punctuated::Punctuated, token,
};

#[proc_macro_derive(EasyConfig, attributes(attr))]
pub fn easy_config_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            fields.named
        } else {
            panic!("Only structs with named fields are supported")
        }
    } else {
        panic!("Only structs are supported")
    };

    let config_key_defs = fields.iter().map(|f| {
        let field_name_str = f.ident.as_ref().unwrap().to_string();
        let mut docs = None;
        let mut default = None;
        let mut importance = None;
        let mut validator = None;
        let mut group = None;

        for attr in &f.attrs {
            if attr.path().is_ident("attr") {
                let parsed_attrs = attr
                    .parse_args_with(Punctuated::<MetaNameValue, token::Comma>::parse_terminated)
                    .expect("Failed to parse config attributes");

                for nv in parsed_attrs {
                    let ident = nv.path.get_ident().unwrap().to_string();

                    match ident.as_str() {
                        "documentation" => {
                            docs = Some(get_string_lit_from_expr(&nv.value).unwrap())
                        }
                        "default" => default = Some(get_string_lit_from_expr(&nv.value).unwrap()),
                        "group" => group = Some(get_string_lit_from_expr(&nv.value).unwrap()),
                        "importance" => importance = Some(get_path_from_expr(&nv.value).unwrap()),
                        "validator" => validator = Some(get_expr(&nv.value).unwrap()),
                        _ => panic!("Unknown attribute: {}", ident),
                    }
                }
            }
        }

        let docs = docs.map(|d| quote! { Some(#d) }).unwrap_or(quote! { None });
        let default = default
            .map(|d| quote! { Some(#d) })
            .unwrap_or(quote! { None });
        let importance = importance
            .map(|i| quote! { Some(#i) })
            .unwrap_or(quote! { None });
        let validator = validator
            .map(|v| quote! { Some(#v) })
            .unwrap_or(quote! { None });
        let group = group
            .map(|g| quote! { Some(#g) })
            .unwrap_or(quote! { None });

        quote! {
            .define(ConfigKey {
                name: #field_name_str, documentation: #docs, default_value: #default,
                importance: #importance, validator: #validator, group: #group,
            })
        }
    });

    let from_props_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let ty = &f.ty;

        // Check if the type is an Option<T>
        if let Type::Path(type_path) = ty &&
            type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Option" {
            // This is an Option<T>, so we need to get the inner type T.
            let inner_ty = if let PathArguments::AngleBracketed(args) = &type_path.path.segments[0].arguments {
                if let Some(GenericArgument::Type(inner)) = args.args.first() {
                    inner
                } else { panic!("Option must have a generic type argument") }
            } else { panic!("Option must have a generic type argument") };

            // Generate special logic for optional fields.
            return quote! {
                    #field_name: {
                        let meta = def.find_key(#field_name_str).ok_or_else(|| ConfigError::MissingName(#field_name_str.to_string()))?;
                        if let Some(val_str) = props.get(#field_name_str).map(|s| s.as_str()).or(meta.default_value) {
                            if let Some(validator) = &meta.validator {
                                validator.validate(#field_name_str, val_str)?;
                            }
                            Some(<#inner_ty as ConfigValue>::parse(#field_name_str, val_str)?)
                        } else {
                            None
                        }
                    }
                };
        }


        // If not an Option<T>, generate the original, required-field logic.
        quote! { #field_name: <#ty as ConfigValue>::parse(#field_name_str, get_value(#field_name_str)?)? }
    });

    let expanded = quote! {
        static CONFIG_DEF: once_cell::sync::Lazy<ConfigDef> = once_cell::sync::Lazy::new(|| {
            let mut builder = ConfigDef::builder();
            builder
                #(#config_key_defs)*;
            builder.build()
        });
        impl #name {
            pub fn config_def() -> &'static ConfigDef { &CONFIG_DEF }
        }
        impl FromConfigDef for #name {
            fn from_props(props: &std::collections::HashMap<String, String>) -> Result<Self, ConfigError> {
                let def = Self::config_def();
                let get_value = |name: &str| -> Result<_, ConfigError> {
                    let meta = def.find_key(name).ok_or_else(|| ConfigError::MissingName(name.to_string()))?;
                    let val_str = props.get(name).map(|s| s.as_str()).or(meta.default_value)
                        .ok_or_else(|| ConfigError::MissingName(name.to_string()))?;
                    if let Some(validator) = &meta.validator {
                        validator.validate(name, val_str)?;
                    }
                    Ok(val_str)
                };
                Ok(Self { #(#from_props_fields),* })
            }
        }
    };
    TokenStream::from(expanded)
}

// --- Helper Functions for Attribute Parsing ---

/// Extracts a `String` from a string literal expression (e.g., `"hello"`).
/// Returns a `syn::Error` if the expression is not a string literal.
fn get_string_lit_from_expr(expr: &Expr) -> syn::Result<String> {
    if let Expr::Lit(expr_lit) = expr
        && let Lit::Str(lit_str) = &expr_lit.lit
    {
        return Ok(lit_str.value());
    }
    Err(syn::Error::new_spanned(expr, "Expected a string literal"))
}

/// Extracts a `Path` from a path expression (e.g., `Importance::HIGH` or `my_validator_fn`).
/// Returns a `syn::Error` if the expression is not a path.
fn get_path_from_expr(expr: &Expr) -> syn::Result<ExprPath> {
    if let Expr::Path(expr_path) = expr {
        return Ok(expr_path.clone());
    }
    Err(syn::Error::new_spanned(
        expr,
        "Expected a path (e.g., an enum variant or a function name)",
    ))
}

fn get_expr(expr: &Expr) -> syn::Result<Expr> {
    Ok(expr.clone())
}
