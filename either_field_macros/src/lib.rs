use proc_macro::TokenStream;
use syn::{Data, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn make_template(
    _attr: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input_as_is = items.clone();
    let ast = parse_macro_input!(items as syn::DeriveInput);

    if let Data::Struct(x) = ast.data {
        let fields = x.fields;
        for field in fields {
            if let Type::Macro(x) = field.ty {
                let macro_type = x.mac;
                if !macro_type
                    .path
                    .segments
                    .iter()
                    .any(|segment| segment.ident.to_string().as_str() == "either")
                {
                    continue;
                }
                // handle multitype here
            }
        }
    } else {
        return input_as_is;
    }

    TokenStream::new()
}

// compiler magic
// this makes an export for LSPs and the compiler to not freak out
// but allows the syntax for make_template
#[proc_macro]
pub fn either(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    input
}
