use proc_macro2::{Literal, Span, TokenTree};
use syn::{
    Ident, LitBool, Token, Type, Visibility, bracketed, parse::Parse, punctuated::Punctuated,
};

macro_rules! syn_error {
    ($message: literal) => {
        syn::Result::Err(syn::Error::new(Span::call_site(), $message))
    };
}

// This is the struct that handles
// parsing all the settings
//
// SettingName: Value, ...;
#[derive(Debug, Default)]
pub(crate) struct Settings {
    pub generate_structs: bool,
    pub delete_template: bool,
    pub delete_empty_tuple_fields: bool,
}
// impl Default for Settings {
//     fn default() -> Self {
//         Self {
//             generate_structs: false,
//             delete_template: false,
//             delete_empty_tuple_fields: false,
//         }
//     }
// }

// This is the struct that handles
// parsing all the derived structs
//
// SettingName: Value, ...;
pub(crate) struct AttrInputs {
    pub settings: Settings,
    pub derived_structs: Vec<Derived>,
}
impl Parse for AttrInputs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut settings = Settings::default();
        let mut settings_fork_cursor = input.fork().cursor();
        let mut has_settings = false;
        let mut token_number_first_half = 0;

        while let Some((tt, next)) = settings_fork_cursor.token_tree() {
            token_number_first_half += 1;
            match &tt {
                TokenTree::Punct(punct) if punct.as_char() == ';' => {
                    has_settings = true;
                    break;
                }
                _ => settings_fork_cursor = next,
            }
        }

        match has_settings {
            false => settings = Settings::default(),
            true => {
                if token_number_first_half % 4 != 0 {
                    return syn_error!(
                        "Something went wrong with parsing the derived struct settings"
                    );
                }
                for _ in 0..(token_number_first_half >> 2) {
                    let ident = input.parse::<Ident>()?;
                    let _ = input.parse::<Token![:]>()?;

                    // match token tree here
                    if input.peek(LitBool) {
                        let value = input.parse::<LitBool>()?;
                        match ident.to_string().as_str() {
                            "GenStructs" => settings.generate_structs = value.value,
                            "DeleteTemplate" => settings.generate_structs = value.value,
                            "OmitEmptyTupleFields" => settings.generate_structs = value.value,
                            _ => (),
                        }
                    } else {
                        return syn_error!("Invalid setting value");
                    }

                    let lookahead = input.lookahead1();
                    if lookahead.peek(Token![;]) {
                        let _ = input.parse::<Token![;]>();
                    } else if lookahead.peek(Token![,]) {
                        let _ = input.parse::<Token![,]>();
                    } else {
                        return syn_error!("Invalid character");
                    }
                }
            }
        }

        let parsed: Punctuated<Derived, Token![,]> = Punctuated::parse_terminated(input)?;

        Ok(Self {
            settings,
            derived_structs: parsed.into_iter().collect(),
        })
    }
}

// This is the struct that handles parsing the
// syntax for specifying the derived structs
//
// VIS is the visibility
//
// VIS struct_name: FieldDescriptor
pub(crate) struct Derived {
    pub vis: Visibility,
    pub name: Ident,
    pub fields: std::collections::HashMap<Ident, Type>,
}
impl Parse for Derived {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse::<Visibility>()?;
        let name = input.parse::<Ident>()?;
        let _ = input.parse::<Token![:]>()?;
        let field_list;
        bracketed!(field_list in input);

        let mut fields = std::collections::HashMap::new();
        for (ident_number, field) in
            (<Punctuated<FieldDescriptor, Token![,]>>::parse_separated_nonempty(&field_list)?)
                .into_iter()
                .enumerate()
        {
            let ident = match field.ident {
                Some(ident) => ident,
                None => Ident::new(format!("_{ident_number}").as_str(), Span::call_site()),
            };
            fields.insert(ident, field.field_type);
        }

        Ok(Self { name, fields, vis })
    }
}

// This is the struct that handles parsing the syntax
// for specifying field types in normal structs
//
// field_name: type, ...
pub(crate) struct FieldDescriptor {
    ident: Option<Ident>,
    field_type: Type,
}
impl Parse for FieldDescriptor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = if input.peek2(Token![:]) {
            let ident = if input.peek(Ident) {
                input.parse::<Ident>()?
            } else {
                let x = input.parse::<Literal>()?;
                let y = x.to_string();
                if y.chars().any(|x| !x.is_numeric()) {
                    return syn_error!("Literal is not number");
                }
                Ident::new(format!("_{y}").as_str(), x.span())
            };
            let _ = input.parse::<Token![:]>()?;
            Some(ident)
        } else {
            None
        };

        let field_type = input.parse::<Type>()?;
        Ok(Self { ident, field_type })
    }
}

// This is the struct that handles parsing the either!() macro's contents
//
// (type | type | ... )
pub(crate) struct EitherMacro(pub Vec<Type>);
impl Parse for EitherMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<Type, Token![|]> = Punctuated::parse_separated_nonempty(input)?;
        Ok(Self(parsed.into_iter().collect()))
    }
}
