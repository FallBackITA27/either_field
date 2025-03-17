use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    bracketed, parse::Parse, parse_macro_input, punctuated::Punctuated, GenericParam, Generics, Ident, Token, Type
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
    let mut generic_struct = parse_macro_input!(items as syn::ItemStruct);
    let initial_generics: Generics = generic_struct.generics.clone();
    let generics = &mut generic_struct.generics.params;

    // this also has to match the order of the generics
    let mut ordered_idents_and_types: Vec<(Ident, Vec<Type>)> = vec![];

    let fields = &mut generic_struct.fields;
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

            match field.clone().ident {
                Some(ident) => ordered_idents_and_types.push((ident, parsed)),
                None => continue,
            };

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

    let derived_list = parse_macro_input!(attr as DerivedList).0;
    let mut out = generic_struct.to_token_stream();
    for derived in derived_list {
        let types = ordered_idents_and_types
            .iter()
            .map(|(ident, possible_types)| match derived.fields.get(ident) {
                None => possible_types[0].clone(),
                Some(v) => match possible_types.contains(v) {
                    false => {
                        let error_message = format!("Type \"{}\" is not part of the specified possible types: {:?}",
                            v.to_token_stream(),
                            possible_types.iter().map(|x| format!("{}", x.to_token_stream())).collect::<Vec<_>>());
                        out.extend::<proc_macro2::TokenStream>(quote! {  compile_error!(#error_message); });
                        v.clone()
                    },
                    true => v.clone(),
                },
            });

        let generic_name = generic_struct.ident.clone();
        let generic_names: Vec<_> = initial_generics
            .type_params()
            .map(|x| x.ident.clone())
            .collect();
        let comma = if generic_names.is_empty() {
            None
        } else {
            Some(syn::token::Comma::default())
        };
        let x = syn::ItemType {
            type_token: syn::token::Type::default(),
            semi_token: syn::token::Semi::default(),
            eq_token: syn::token::Eq::default(),
            attrs: vec![],
            ident: derived.name,
            vis: generic_struct.vis.clone(),
            ty: std::boxed::Box::new(Type::Verbatim(quote! {
                #generic_name<#(#generic_names),* #comma #(#types),*>
            })),
            generics: initial_generics.clone(),
        };

        out.extend::<proc_macro2::TokenStream>(x.into_token_stream());
    }

    out.into()
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
    fields: std::collections::HashMap<Ident, Type>,
}
impl Parse for Derived {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let _ = input.parse::<Token![:]>()?;
        let field_list;
        bracketed!(field_list in input);
        let mut fields = std::collections::HashMap::new();
        for field in
            <Punctuated<FieldDescriptor, Token![,]>>::parse_separated_nonempty(&field_list)?
        {
            fields.insert(field.ident, field.field_type);
        }

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
// that the make_template macro compiles
// before this function-like macro
#[proc_macro]
pub fn either(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    TokenStream::new()
}
