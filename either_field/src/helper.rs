use proc_macro2::Span;
use syn::{GenericParam, Ident, Macro, Type, punctuated::Punctuated, token::Comma};

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
    if let Type::Macro(x) = x {
        if x.mac
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "either")
        {
            return Some(x.mac.clone());
        }
    }
    None
}

pub(crate) fn push_if_empty_tuple(x: &Type, delete: bool, vec: &mut Vec<bool>) {
    match delete && matches!(x, Type::Tuple(syn::TypeTuple { elems, .. }) if elems.is_empty()) {
        true => vec.push(true),
        false => vec.push(false),
    }
}
