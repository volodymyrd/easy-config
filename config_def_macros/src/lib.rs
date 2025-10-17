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

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("Only structs with named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let mut config_key_inits = Vec::new();
    let mut from_props_fields = Vec::new();
    let mut getter_methods = Vec::new();

    for f in fields.iter() {
        let field_name = f.ident.as_ref().unwrap();
        let field_ty = &f.ty;

        if f.attrs.iter().any(|attr| attr.path().is_ident("merge")) {
            config_key_inits.push(quote! {
                <#field_ty as FromConfigDef>::config_def()?.config_keys().values().cloned().collect::<Vec<_>>()
            });
            from_props_fields.push(quote! {
                #field_name: <#field_ty as FromConfigDef>::from_props(props)?
            });
        } else {
            let mut attrs = ParsedAttributes::default();
            for attr in &f.attrs {
                if attr.path().is_ident("attr") {
                    let parsed = attr
                        .parse_args_with(Punctuated::<Meta, token::Comma>::parse_terminated)
                        .expect("Failed to parse config attributes");
                    attrs.populate_from(parsed);
                }
            }

            if attrs.getter {
                getter_methods.push(quote! {
                    pub fn #field_name(&self) -> &#field_ty { &self.#field_name }
                });
            }

            let field_name_str = field_name.to_string();
            let lookup_key = attrs
                .name
                .map_or(quote! { #field_name_str }, |e| quote! { #e });
            let docs = attrs
                .documentation
                .map(|d| quote! { Some(Into::<String>::into(#d)) })
                .unwrap_or(quote! { None });
            let default = attrs
                .default
                .map(|d| quote! { Some(#d) })
                .unwrap_or(quote! { None });
            let importance = attrs
                .importance
                .map(|i| quote! { Some(#i) })
                .unwrap_or(quote! { None });
            let validator = attrs
                .validator
                .map(|v| quote! { Some(#v) })
                .unwrap_or(quote! { None });
            let group = attrs
                .group
                .map(|g| quote! { Some(Into::<String>::into(#g)) })
                .unwrap_or(quote! { None });
            let internal_config = attrs.internal_config;

            let (is_option, inner_ty) = {
                let mut is_opt = false;
                let mut inner = quote! { #field_ty };

                if let Type::Path(type_path) = field_ty
                    && type_path.path.segments.len() == 1
                    && type_path.path.segments[0].ident == "Option"
                    && let PathArguments::AngleBracketed(params) =
                        &type_path.path.segments[0].arguments
                    && let Some(GenericArgument::Type(t)) = params.args.first()
                {
                    is_opt = true;
                    inner = quote! { #t };
                }

                (is_opt, inner)
            };

            config_key_inits.push(quote! {
                vec![Box::new(ConfigKey::<#inner_ty> {
                    name: #lookup_key,
                    documentation: #docs,
                    default_value: #default,
                    importance: #importance,
                    validator: #validator,
                    group: #group,
                    internal_config: #internal_config,
                }) as Box<dyn ConfigKeyTrait>]
            });

            // Reverted to separate logic paths for `T` and `Option<T>` to fix the error.
            let from_props_logic = if is_option {
                quote! {
                    #field_name: {
                        let key_name = #lookup_key;
                        let meta_opt = def.find_key(key_name);
                        if let Some(val_str) = props.get(key_name) {
                            if let Some(meta) = meta_opt {
                                if let Some(validator) = meta.validator() {
                                    validator.validate(key_name, val_str)?;
                                }
                            }
                            Some(<#inner_ty as ConfigValue>::parse(key_name, val_str)?)
                        } else if let Some(meta) = meta_opt {
                            if let Some(default_val_any) = meta.default_value_any() {
                                let default_val = default_val_any.downcast_ref::<#inner_ty>().unwrap().clone();
                                if let Some(validator) = meta.validator() {
                                    validator.validate(key_name, &default_val.to_config_string())?;
                                }
                                Some(default_val)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
            } else {
                quote! {
                    #field_name: {
                        let key_name = #lookup_key;
                        let meta = def.find_key(key_name).ok_or_else(|| ConfigError::MissingName(key_name.to_string()))?;
                        if let Some(val_str) = props.get(key_name) {
                            if let Some(validator) = meta.validator() {
                                validator.validate(key_name, val_str)?;
                            }
                            <#inner_ty as ConfigValue>::parse(key_name, val_str)?
                        } else if let Some(default_val_any) = meta.default_value_any() {
                            let default_val = default_val_any.downcast_ref::<#inner_ty>().unwrap().clone();
                            if let Some(validator) = meta.validator() {
                                validator.validate(key_name, &default_val.to_config_string())?;
                            }
                            default_val
                        } else {
                            return Err(ConfigError::MissingName(key_name.to_string()));
                        }
                    }
                }
            };
            from_props_fields.push(from_props_logic);
        }
    }

    let expanded = quote! {
        static CONFIG_DEF: once_cell::sync::OnceCell<ConfigDef> = once_cell::sync::OnceCell::new();

        impl #struct_name {
            #(#getter_methods)*
        }

        impl FromConfigDef for #struct_name {
            fn from_props(props: &std::collections::HashMap<String, String>) -> Result<Self, ConfigError> {
                let def = Self::config_def()?;
                Ok(Self { #(#from_props_fields),* })
            }

            fn config_def() -> Result<&'static ConfigDef, ConfigError> {
                CONFIG_DEF.get_or_try_init(|| {
                    let keys: Vec<Box<dyn ConfigKeyTrait>> = vec![
                        #(#config_key_inits),*
                    ].into_iter().flatten().collect();
                    ConfigDef::try_from(keys)
                })
            }
        }
    };
    TokenStream::from(expanded)
}

/// A helper struct to organize parsed attributes within the macro.
#[derive(Default)]
struct ParsedAttributes {
    name: Option<Expr>,
    documentation: Option<Expr>,
    default: Option<Expr>,
    group: Option<Expr>,
    importance: Option<Expr>,
    validator: Option<Expr>,
    getter: bool,
    internal_config: bool,
}

impl ParsedAttributes {
    fn populate_from(&mut self, parsed_attrs: Punctuated<Meta, token::Comma>) {
        for meta in parsed_attrs {
            match meta {
                Meta::Path(path) if path.is_ident("getter") => {
                    self.getter = true;
                }
                Meta::NameValue(nv) => {
                    let ident = nv.path.get_ident().unwrap().to_string();
                    match ident.as_str() {
                        "name" => self.name = Some(nv.value),
                        "documentation" => self.documentation = Some(nv.value),
                        "default" => self.default = Some(nv.value),
                        "group" => self.group = Some(nv.value),
                        "importance" => self.importance = Some(nv.value),
                        "validator" => self.validator = Some(nv.value),
                        "internal_config" => {
                            if let Expr::Lit(expr_lit) = nv.value
                                && let Lit::Bool(lit_bool) = expr_lit.lit
                            {
                                self.internal_config = lit_bool.value();
                            }
                        }
                        _ => panic!("Unknown attribute: {}", ident),
                    }
                }
                _ => {}
            }
        }
    }
}
