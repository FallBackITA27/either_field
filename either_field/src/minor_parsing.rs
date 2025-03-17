use proc_macro2::{Literal, Span};
use syn::{Ident, Token, Type, Visibility, bracketed, parse::Parse, punctuated::Punctuated};

// This is the struct that handles
// parsing all the derived structs
//
// Derived, ...
pub(crate) struct DerivedList(pub Vec<Derived>);
impl Parse for DerivedList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<Derived, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(Self(parsed.into_iter().collect()))
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
        let mut ident_number = 0;
        for field in
            <Punctuated<FieldDescriptor, Token![,]>>::parse_separated_nonempty(&field_list)?
        {
            let ident = match field.ident {
                Some(ident) => ident,
                None => Ident::new(format!("_{ident_number}").as_str(), Span::call_site()),
            };
            fields.insert(ident, field.field_type);
            ident_number += 1;
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
                Ident::new(format!("_{x}").as_str(), x.span())
            };
            let _ = input.parse::<Token![:]>()?;
            println!("{ident:#?}");
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
