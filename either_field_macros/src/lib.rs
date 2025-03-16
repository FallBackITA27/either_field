use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Data, Token, Type, parse::Parse, parse_macro_input};

#[proc_macro_attribute]
pub fn make_template(
    _attr: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input_as_is = items.clone();
    let mut ast = parse_macro_input!(items as syn::DeriveInput);

    if let Data::Struct(ref mut x) = ast.data {
        let fields = &mut x.fields;
        for field in fields {
            if let Type::Macro(ref mut x) = field.ty {
                let macro_type = &x.mac;
                if !macro_type
                    .path
                    .segments
                    .iter()
                    .any(|segment| segment.ident.to_string().as_str() == "either")
                // TODO: need a better way to check whether the macro is the correct one
                {
                    continue;
                }
                let tokens: TokenStream = macro_type.tokens.clone().into();
                let parsed_types = parse_macro_input!(tokens as EitherMacro);
                // handle multitype here
            }
        }
    } else {
        return input_as_is;
    }

    ast.to_token_stream().into()
}

struct EitherMacro {
    types: Vec<Type>,
}

impl Parse for EitherMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut out = vec![];
        loop {
            out.push(input.parse::<Type>()?);
            if input.parse::<Token![|]>().is_err() {
                break;
            }
        }

        Ok(Self { types: out })
    }
}

// Compiler Magic
// this makes an export for LSPs and the
// compiler to not freak out but allows
// the syntax for make_template to be
// syntactically valid according to the
// compiler. This relies on the fact
// that the attribute macro compiles
// before the function-like macro
#[proc_macro]
pub fn either(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    input
}
