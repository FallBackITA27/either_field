use proc_macro2::Span;
use syn::{GenericParam, Ident, Macro, Type, punctuated::Punctuated, token::Comma};

fn get_alpha(n: usize) -> String {
    let index = (n % 26) as u8;
    let character = b'A' + index;
    if n < 26 {
        return unsafe { String::from_utf8_unchecked(vec![character; 1]) };
    }
    let mut x = get_alpha((n / 26) - 1);
    x.push(char::from(character));
    x
}

pub(crate) fn generate_generic_name(
    generics: &mut Punctuated<GenericParam, Comma>,
    n: &mut usize,
) -> Ident {
    let mut new_generic_name = get_alpha(*n);
    while generics
        .iter()
        .any(|x| matches!(x, GenericParam::Type(x) if x.ident == new_generic_name))
    {
        *n += 1;
        new_generic_name = get_alpha(*n);
    }
    Ident::new(&new_generic_name, Span::call_site())
}

// TODO: need a better way to check whether the macro
// is the correct one and not one with the same name
pub(crate) fn get_macro_from_type(x: &Type) -> Option<Macro> {
    if let Type::Macro(x) = x
        && x.mac
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "either")
    {
        return Some(x.mac.clone());
    }
    None
}
