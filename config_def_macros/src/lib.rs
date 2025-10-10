extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, Fields, GenericArgument, Lit, Meta, PathArguments, Type,
    parse_macro_input, punctuated::Punctuated, token,
};

#[proc_macro_derive(EasyConfig, attributes(attr, merge))]
pub fn easy_config_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            fields.named
        } else {
            panic!("Only structs with named fields are supported")
        }
    } else {
        panic!("Only structs are supported")
    };

    let mut config_key_defs = Vec::new();
    let mut from_props_fields = Vec::new();
    let mut getter_methods = Vec::new();

    for f in fields.iter() {
        let field_name = f.ident.as_ref().unwrap();
        let ty = &f.ty;

        // Check if this field should be merged.
        let is_merge_field = f.attrs.iter().any(|attr| attr.path().is_ident("merge"));

        if is_merge_field {
            // --- Logic for a `#[merge]` field ---

            // 1. Generate code for the ConfigDef builder.
            // This delegates to the inner struct's `config_def` and merges the keys.
            config_key_defs.push(quote! {
                for key in <#ty as FromConfigDef>::config_def()?.config_keys().values() {
                    builder.define(key.clone())?;
                }
            });

            // 2. Generate code for the `from_props` field initializer.
            // This delegates parsing to the inner struct's `from_props`.
            from_props_fields.push(quote! {
                #field_name: <#ty as FromConfigDef>::from_props(props)?
            });
        } else {
            let mut generate_getter = false;
            let field_name_str = field_name.to_string();
            let mut name_attr = None;
            let mut docs = None;
            let mut default = None;
            let mut importance = None;
            let mut validator = None;
            let mut group = None;

            for attr in &f.attrs {
                if attr.path().is_ident("attr") {
                    let parsed_attrs = attr
                        .parse_args_with(Punctuated::<Meta, token::Comma>::parse_terminated)
                        .expect("Failed to parse config attributes");

                    for meta in parsed_attrs {
                        match meta {
                            Meta::Path(path) if path.is_ident("getter") => {
                                generate_getter = true;
                            }
                            Meta::NameValue(nv) => {
                                let ident = nv.path.get_ident().unwrap().to_string();
                                match ident.as_str() {
                                    "name" => {
                                        name_attr =
                                            Some(get_string_lit_from_expr(&nv.value).unwrap())
                                    }
                                    "documentation" => {
                                        docs = Some(get_string_lit_from_expr(&nv.value).unwrap())
                                    }
                                    "default" => {
                                        default = Some(get_string_lit_from_expr(&nv.value).unwrap())
                                    }
                                    "group" => {
                                        group = Some(get_string_lit_from_expr(&nv.value).unwrap())
                                    }
                                    "importance" => importance = Some(get_expr(&nv.value).unwrap()),
                                    "validator" => validator = Some(get_expr(&nv.value).unwrap()),
                                    _ => panic!("Unknown attribute: {}", ident),
                                }
                            }
                            _ => { /* Ignore other attribute types */ }
                        }
                    }
                }
            }

            if generate_getter {
                // We always return a borrow (`&T`). This is safe for all types.
                // For Copy types, the user can dereference with `*`.
                getter_methods.push(quote! {
                    pub fn #field_name(&self) -> &#ty {
                        &self.#field_name
                    }
                });
            }

            let lookup_key = name_attr.clone().unwrap_or_else(|| field_name_str.clone());
            let docs_tokens = docs.map(|d| quote! { Some(#d) }).unwrap_or(quote! { None });
            let default_tokens = default
                .map(|d| quote! { Some(#d) })
                .unwrap_or(quote! { None });
            let importance_tokens = importance
                .map(|i| quote! { Some(#i) })
                .unwrap_or(quote! { None });
            let validator_tokens = validator
                .map(|v| quote! { Some(#v) })
                .unwrap_or(quote! { None });
            let group_tokens = group
                .map(|g| quote! { Some(#g) })
                .unwrap_or(quote! { None });

            config_key_defs.push(quote! {
                builder.define(ConfigKey {
                    name: #lookup_key,
                    documentation: #docs_tokens,
                    default_value: #default_tokens,
                    importance: #importance_tokens,
                    validator: #validator_tokens,
                    group: #group_tokens,
                })?;
            });

            let from_props_quote = if let Type::Path(type_path) = ty
                && type_path.path.segments.len() == 1
                && type_path.path.segments[0].ident == "Option"
            {
                let inner_ty = if let PathArguments::AngleBracketed(args) =
                    &type_path.path.segments[0].arguments
                {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        inner
                    } else {
                        panic!("Option must have a generic type argument")
                    }
                } else {
                    panic!("Option must have a generic type argument")
                };

                quote! {
                    #field_name: {
                        let meta = def.find_key(#lookup_key).ok_or_else(|| ConfigError::MissingName(#lookup_key.to_string()))?;
                        if let Some(val_str) = props.get(#lookup_key).map(|s| s.as_str()).or(meta.default_value) {
                            if let Some(validator) = &meta.validator {
                                validator.validate(#lookup_key, val_str)?;
                            }
                            Some(<#inner_ty as ConfigValue>::parse(#lookup_key, val_str)?)
                        } else {
                            None
                        }
                    }
                }
            } else {
                quote! {
                    #field_name: <#ty as ConfigValue>::parse(#lookup_key, get_value(#lookup_key)?)?
                }
            };
            from_props_fields.push(from_props_quote);
        }
    }

    let expanded = quote! {
        // This `impl` block contains the generated getters.
        // It's separate from the other impls for clarity.
        impl #struct_name {
            #(#getter_methods)*
        }

        static CONFIG_DEF: once_cell::sync::OnceCell<ConfigDef> = once_cell::sync::OnceCell::new();
        impl #struct_name {
            pub fn config_def() -> Result<&'static ConfigDef, ConfigError> {
                CONFIG_DEF.get_or_try_init(|| {
                    let mut builder = ConfigDef::builder();
                    #(#config_key_defs)*
                    Ok(builder.build())
                })
            }
        }
        impl FromConfigDef for #struct_name {
            fn from_props(props: &std::collections::HashMap<String, String>) -> Result<Self, ConfigError> {
                let def = Self::config_def()?;
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

             // Re-direct the trait method to our generated inherent method
            fn config_def() -> Result<&'static ConfigDef, ConfigError> {
                Self::config_def()
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

fn get_expr(expr: &Expr) -> syn::Result<Expr> {
    Ok(expr.clone())
}
