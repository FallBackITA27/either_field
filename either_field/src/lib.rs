use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    Data, GenericParam, Ident, Token, Type, bracketed, parse::Parse, parse_macro_input,
    punctuated::Punctuated,
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
    attr: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input_as_is = items.clone();
    let mut generic_struct = parse_macro_input!(items as syn::DeriveInput);
    let generics = &mut generic_struct.generics.params;

    if let Data::Struct(ref mut x) = generic_struct.data {
        let fields = &mut x.fields;
        let mut ident_counter = 0;
        for field in fields {
            if let Type::Macro(ref mut x) = field.ty {
                let macro_type = &x.mac;
                if !macro_type
                    .path
                    .segments
                    .iter()
                    .any(|segment| segment.ident == "either")
                // TODO: need a better way to check whether the macro is the correct one
                {
                    continue;
                }
                let tokens: TokenStream = macro_type.tokens.clone().into();
                let parsed = parse_macro_input!(tokens as EitherMacro).0;
                let mut new_generic_name = get_alpha(ident_counter);
                while generics.iter().any(|x| {
                    if let GenericParam::Type(x) = x {
                        return x.ident == new_generic_name;
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
                field.ty = Type::Verbatim(ident.into_token_stream());
            }
            ident_counter += 1;
        }
    } else {
        return input_as_is;
    }

    let derived = parse_macro_input!(attr as DerivedList).0;

    generic_struct.to_token_stream().into()
}

struct DerivedList(Vec<Derived>);
impl Parse for DerivedList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<Derived, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(Self(parsed.into_iter().collect()))
    }
}

struct Derived {
    name: Ident,
    fields: Vec<FieldDescriptor>,
}
impl Parse for Derived {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let _ = input.parse::<Token![;]>()?;
        let field_list;
        bracketed!(field_list in input);

        let parsed: Punctuated<FieldDescriptor, Token![,]> =
            Punctuated::parse_separated_nonempty(&field_list)?;
        let fields = parsed.into_iter().collect();
        Ok(Self { name, fields })
    }
}

struct FieldDescriptor {
    ident: Ident,
    field_type: Type,
}
impl Parse for FieldDescriptor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let _ = input.parse::<Token![:]>()?;
        let field_type = input.parse::<Type>()?;

        Ok(Self { ident, field_type })
    }
}

struct EitherMacro(Vec<Type>);
impl Parse for EitherMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<Type, Token![|]> = Punctuated::parse_separated_nonempty(input)?;
        Ok(Self(parsed.into_iter().collect()))
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
pub fn either(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    TokenStream::new()
}
