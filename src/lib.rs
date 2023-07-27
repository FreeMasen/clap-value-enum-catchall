use heck::{ToLowerCamelCase, ToUpperCamelCase, ToSnakeCase, ToKebabCase};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    token::{Bracket, Comma},
    Data, DeriveInput, Expr, ExprArray, Fields, LitStr, Type, Variant, Attribute, Token, parse::Parse, Meta,
};

#[derive(Clone, Copy, Debug)]
enum Casing {
    Camel,
    Kabob,
    Snake,
    Pascal,
    Lower,
    Upper,
    ScreamingSnake,
}

impl Casing {
    pub fn apply(self, s: &str) -> String {
        
        match self {
            Self::Camel => s.to_lower_camel_case(),
            Self::Pascal => s.to_upper_camel_case(),
            Self::Snake => s.to_snake_case(),
            Self::ScreamingSnake => s.to_snake_case().to_uppercase(),
            Self::Kabob => s.to_kebab_case(),
            Self::Lower => s.to_lowercase(),
            Self::Upper => s.to_uppercase(),
        }
    }
}
#[derive(Debug)]
struct Renamer {
    _ident: Ident,
    _punct: Token![=],
    value: LitStr,
}

impl Parse for Renamer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _ident: input.parse()?,
            _punct: input.parse()?,
            value: input.parse()?
        })
    }
}

impl TryFrom<&[Attribute]> for Casing {
    type Error = syn::Error;
    fn try_from(value: &[Attribute]) -> Result<Self, Self::Error> {
        for attr in value {
            if let Ok(fits) = dbg!(Self::try_from(attr)) {
                return Ok(fits);
            }
        }
        Err(Self::Error::new(Span::call_site(), ""))
    }
}
impl TryFrom<&Attribute> for Casing {
    type Error = syn::Error;
    fn try_from(value: &Attribute) -> Result<Self, Self::Error> {
        let Meta::List(meta_info) = &value.meta else {
            panic!("Unknown attribute")
        };
        assert_eq!(
            meta_info.path.segments.last().map(|s| s.ident.to_string()).unwrap(),
            "catchall",
        );
        let renamer: Renamer = dbg!(syn::parse2(meta_info.tokens.clone()))?;
        Ok(match dbg!(renamer.value.value()).as_str() {
            "kabob-case" => Casing::Kabob,
            "camelCase" => Casing::Camel,
            "PascalCase" => Casing::Pascal,
            "snake_case" => Casing::Snake,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnake,
            "lowercase" => Self::Lower,
            "UPPERCASE" => Self::Upper,
            value => return Err(Self::Error::new(Span::call_site(), format!("Unknown casing {value:?}"))),
        })
    }
}

#[proc_macro_derive(ValueEnumCatchall, attributes(catchall))]
pub fn value_enum_catchall(tokens: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(tokens);
    let parser_name = Ident::new(&format!("__{}Parser", input.ident), Span::call_site());
    let factory = generate_value_parser_factory(&parser_name, &input.ident);
    let Data::Enum(input_data) = &input.data else {
        panic!("Cannot catchall structs")
    };
    let (catchall_ty, variants) = lookup_variants_and_catchall(&input_data.variants, &input.attrs);
    let value_parser =
        generate_typed_value_parser(&parser_name, &input.ident, &variants, catchall_ty);
    (quote::quote! {
        #[derive(Clone)]
        pub struct #parser_name;
        #factory
        #value_parser

    })
    .into()
}

fn lookup_variants_and_catchall(
    variants: &Punctuated<Variant, Comma>,
    attrs: &[Attribute]
) -> ((Type, Ident), Vec<(LitStr, Ident)>) {
    let mut ret_vars = Vec::new();
    let mut maybe_catchall = None;
    let global_casing = Casing::try_from(attrs).ok();
    for var in variants {
        match &var.fields {
            Fields::Unit => {
                let s = if let Ok(local_casing) = dbg!(Casing::try_from(var.attrs.as_slice())) {
                    local_casing.apply(&var.ident.to_string())
                } else if let Some(casing) = dbg!(&global_casing) {
                    casing.apply(&var.ident.to_string())
                } else {
                    var.ident.to_string()
                };

                ret_vars.push((
                    LitStr::new(dbg!(&s), Span::call_site()),
                    var.ident.clone(),
                ))
            },
            Fields::Unnamed(unnamed) => {
                if unnamed.unnamed.len() != 1 {
                    panic!("catchall variants can only have 1 unnamed field");
                }
                if maybe_catchall.is_some() {
                    panic!("There can only be 1 catchall variant");
                }
                maybe_catchall = Some((
                    unnamed.unnamed.first().unwrap().ty.clone(),
                    var.ident.clone(),
                ))
            }
            Fields::Named(_named) => {
                todo!()
            }
        }
    }
    (maybe_catchall.unwrap(), ret_vars)
}

fn generate_value_parser_factory(
    parser_name: &Ident,
    value_type: &Ident,
) -> proc_macro2::TokenStream {
    quote::quote! {
        impl clap::builder::ValueParserFactory for #value_type {
            type Parser = #parser_name;
            fn value_parser() -> Self::Parser {
                #parser_name
            }
        }
    }
}

fn generate_possible_values(
    variants: impl Iterator<Item = LitStr>,
    catchall_placeholder: LitStr,
) -> proc_macro2::TokenStream {
    let mut array: Punctuated<Expr, Comma> = Punctuated::new();
    for lit in variants {
        let expr: Expr =
            syn::parse2(quote::quote!(clap::builder::PossibleValue::new(#lit))).unwrap();
        array.push_value(expr);
        array.push_punct(Default::default());
    }
    let expr: Expr =
        syn::parse2(quote::quote!(clap::builder::PossibleValue::new(#catchall_placeholder)))
            .unwrap();
    array.push_value(expr);
    array.push_punct(Default::default());
    let array = ExprArray {
        attrs: Vec::new(),
        bracket_token: Bracket {
            ..Default::default()
        },
        elems: array,
    };
    quote::quote! {
        fn possible_values(
            &self,
        ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {

            Some(Box::new(#array.into_iter()))
        }
    }
}
fn generate_typed_value_parser(
    parser_name: &Ident,
    value_type: &Ident,
    variants: &[(LitStr, Ident)],
    (catchall_type, catchall_ctor): (Type, Ident),
) -> proc_macro2::TokenStream {
    let mut vars = variants.iter();
    let (first, first_ctor) = vars.next().expect("no variants found");
    let mut matcher = quote::quote! {
        if value == #first {
            return Ok(Self::Value::#first_ctor)
        }
    };

    for (lit, ctor) in vars {
        matcher = quote::quote! {
            #matcher else if value == #lit {
                return Ok(Self::Value::#ctor)
            }
        };
    }
    let catchall_placeholder = LitStr::new(
        &format!("<{}>", type_name(&catchall_type)),
        Span::call_site(),
    );
    let possible_values = generate_possible_values(
        variants.iter().map(|(l, _)| l.clone()),
        catchall_placeholder,
    );
    let handle_error = quote::quote! {
        let handle_err = || {
            let mut err = clap::Error::new(clap::error::ErrorKind::ValueValidation)
                .with_cmd(cmd);
            if let Some(arg) = arg {
                err.insert(clap::error::ContextKind::InvalidArg, clap::error::ContextValue::String(arg.to_string()));
            }

            if let Some(iter) = self.possible_values() {
                let strs: Vec<_> = iter.map(|p| p.get_name().to_string()).collect();
                err.insert(clap::error::ContextKind::SuggestedValue, clap::error::ContextValue::Strings(strs));
            }
            err
        };
    };
    quote::quote! {
        impl clap::builder::TypedValueParser for #parser_name {

            type Value = #value_type;

            #possible_values

            fn parse_ref(
                &self,
                cmd: &clap::Command,
                arg: Option<&clap::Arg>,
                value: &std::ffi::OsStr,
            ) -> Result<Self::Value, clap::Error> {
                #matcher
                #handle_error
                let as_str = value.to_str().ok_or_else(|| {
                    handle_err()
                })?;
                let inner = as_str.parse::<#catchall_type>().map_err(|_| {
                    handle_err()
                })?;
                Ok(Self::Value::#catchall_ctor(inner))
            }

        }
    }
}

fn type_name(ty: &Type) -> String {
    match ty {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .expect("non-empty path")
            .ident
            .to_string()
            .to_lowercase(),
        Type::Reference(r) => type_name(&r.elem),
        _ => todo!(),
    }
}
