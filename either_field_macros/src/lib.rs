use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    Data, GenericParam, Ident, Token, Type, parse::Parse, parse_macro_input, punctuated::Punctuated,
};

fn get_alpha(n: usize) -> String {
    const ALPHABET: [char; 26] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    if n < 26 {
        return ALPHABET[n].to_string();
    }
    let mut x = get_alpha((n / 26) - 1);
    x.push(ALPHABET[n % 26]);
    x
}

#[proc_macro_attribute]
pub fn make_template(
    _attr: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input_as_is = items.clone();
    let mut ast = parse_macro_input!(items as syn::DeriveInput);
    let generics = &mut ast.generics.params;

    if let Data::Struct(ref mut x) = ast.data {
        let fields = &mut x.fields;
        let mut ident_counter = 0;
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
                let parsed = parse_macro_input!(tokens as EitherMacro);
                let mut new_generic_name = get_alpha(ident_counter);
                while generics.iter().any(|x| {
                    if let GenericParam::Type(x) = x {
                        return x.ident.to_string() == new_generic_name;
                    }
                    false
                }) {
                    ident_counter += 1;
                    new_generic_name = get_alpha(ident_counter);
                }
                let ident = Ident::new(&new_generic_name, Span::call_site());
                generics.push(GenericParam::Type(syn::TypeParam {
                    attrs: vec![],
                    ident: ident.clone(),
                    colon_token: None,
                    bounds: Punctuated::new(),
                    eq_token: None,
                    default: None,
                }));
                field.ty = Type::Verbatim(ident.into_token_stream().into());

                for parsed_type in parsed.types {
                    // handle multitype here
                }
            }
            ident_counter += 1;
        }
    } else {
        return input_as_is;
    }

    ast.to_token_stream().into()
}

struct EitherMacro {
    types: Punctuated<Type, Token![|]>,
}

impl Parse for EitherMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            types: Punctuated::parse_separated_nonempty(input)?,
        })
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
