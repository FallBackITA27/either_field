use syn::{Macro, Type};

pub(crate) fn get_alpha(n: usize) -> String {
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
    return None;
}
