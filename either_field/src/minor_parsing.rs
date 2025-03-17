use syn::{bracketed, parse::Parse, punctuated::Punctuated, Ident, Token, Type, Visibility};

pub(crate) struct DerivedList(pub Vec<Derived>);
impl Parse for DerivedList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<Derived, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(Self(parsed.into_iter().collect()))
    }
}

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
        for field in
            <Punctuated<FieldDescriptor, Token![,]>>::parse_separated_nonempty(&field_list)?
        {
            fields.insert(field.ident, field.field_type);
        }

        Ok(Self { name, fields, vis })
    }
}

pub(crate) struct FieldDescriptor {
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

pub(crate) struct EitherMacro(pub Vec<Type>);
impl Parse for EitherMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<Type, Token![|]> = Punctuated::parse_separated_nonempty(input)?;
        Ok(Self(parsed.into_iter().collect()))
    }
}
