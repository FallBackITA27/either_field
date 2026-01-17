use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    GenericParam, Generics, Ident, Type, parse_macro_input, punctuated::Punctuated, token::Comma,
};

mod helper;
mod minor_parsing;

macro_rules! custom_compiler_error_msg {
    ($out: ident, $format: literal) => {
            let error_message = String::from($format);
            $out.extend::<proc_macro2::TokenStream>(quote! {  compile_error!(#error_message); });
        };
    ($out: ident, $format: literal, $($arg:expr),*) => {
            let error_message = format!($format, $($arg),*);
            $out.extend::<proc_macro2::TokenStream>(quote! {  compile_error!(#error_message); });
        };
}

/// The meat and bone of the crate
///
/// This will turn any template struct, i.e:
/// ```
/// #[make_template(/* ... */)]
/// struct ThisIsAnExample {
///     field_1: either!(() | i32),
///     field_2: either!(() | String)
/// }
/// ```
/// into all the variants defined in the attribute input
/// ```
/// #[make_template(
///     DerivateOne: [
///         field_1: i32
///     ],
///     DerivateTwo: [
///         field_2: String
///     ],
///     DerivateThree: [
///         field_1: i32
///         field_2: String
///     ]
/// )]
/// struct ThisIsAnExample {/* ... */}
/// ```
/// which will effectively turn to the following code
/// ```
/// struct ThisIsAnExample<A, B> {
///     field_1: A,
///     field_2: B
/// }
/// type DerivateOne = ThisIsAnExample<i32, ()>;
/// type DerivateTwo = ThisIsAnExample<(), String>;
/// type DerivateThree = ThisIsAnExample<i32, String>;
/// ```
/// Every unspecified field will use the first argument of [`macro@either`]
/// as default.
///
/// Because the syntax is JSON-like, a common error is having extra commas.
#[proc_macro_attribute]
pub fn make_template(
    attr: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut out = proc_macro2::TokenStream::new();

    let template_struct = parse_macro_input!(items as syn::ItemStruct);
    let attribute_inputs = parse_macro_input!(attr as minor_parsing::AttrInputs);

    match (
        &template_struct.fields,
        attribute_inputs.settings.generate_structs,
    ) {
        (syn::Fields::Unit, true) | (syn::Fields::Unit, false) => {
            custom_compiler_error_msg!(out, "Unit structs have no fields to do anything about.");
            out.into()
        }
        (syn::Fields::Unnamed(_), false) | (syn::Fields::Named(_), false) => {
            gen_types(out, template_struct, attribute_inputs)
        }
        (syn::Fields::Named(_), true) => gen_structs(out, template_struct, attribute_inputs, false),
        (syn::Fields::Unnamed(_), true) => {
            gen_structs(out, template_struct, attribute_inputs, true)
        }
    }
}

fn gen_types(
    mut out: proc_macro2::TokenStream,
    mut template_struct: syn::ItemStruct,
    attribute_inputs: minor_parsing::AttrInputs,
) -> TokenStream {
    let initial_generics: Generics = template_struct.generics.clone();
    let generics = &mut template_struct.generics.params;
    // this also has to match the order of the generics
    let mut ordered_idents_and_types: Vec<(Ident, Vec<Type>)> = vec![];
    let mut ident_counter = 0;
    for field in template_struct.fields.iter_mut() {
        let type_macro = match helper::get_macro_from_type(&field.ty) {
            Some(x) => x,
            None => continue,
        };

        let tokens: TokenStream = type_macro.tokens.clone().into();
        let parsed = parse_macro_input!(tokens as minor_parsing::EitherMacro).0;

        ordered_idents_and_types.push((field.ident.as_ref().unwrap().clone(), parsed));

        let ident = helper::generate_generic_name(generics, &mut ident_counter);
        generics.push(GenericParam::Type(syn::TypeParam {
            attrs: vec![],
            ident: ident.clone(),
            colon_token: None,
            bounds: Punctuated::new(),
            eq_token: None,
            default: None,
        }));
        field.ty = Type::Verbatim(ident.into_token_stream());
        ident_counter += 1;
    }

    let derived_list = attribute_inputs.derived_structs;
    out.extend::<proc_macro2::TokenStream>(template_struct.to_token_stream());
    for derived in derived_list {
        let mut types = vec![];
        for (ident, possible_types) in &ordered_idents_and_types {
            match derived.fields.get(&ident.to_string()) {
                None => types.push(possible_types[0].clone()),
                Some(v) => {
                    if let Type::Infer(_) = v {
                        types.push(possible_types[0].clone());
                        continue;
                    }

                    match possible_types.contains(v) {
                        true => types.push(v.clone()),
                        false => {
                            custom_compiler_error_msg!(
                                out,
                                "Type \"{}\" (struct \"{}\", field \"{}\") is not part of the specified possible types: {:?}",
                                v.to_token_stream(),
                                derived.name,
                                ident,
                                possible_types
                                    .iter()
                                    .map(|x| format!("{}", x.to_token_stream()))
                                    .collect::<Vec<_>>()
                            );
                            return out.into();
                        }
                    }
                }
            }
        }

        let generic_name = template_struct.ident.clone();
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
            vis: derived.vis.clone(),
            ty: std::boxed::Box::new(Type::Verbatim(quote! {
                #generic_name<#(#generic_names),* #comma #(#types),*>
            })),
            generics: initial_generics.clone(),
        };

        out.extend::<proc_macro2::TokenStream>(x.into_token_stream());
    }

    out.into()
}

fn gen_structs(
    mut out: proc_macro2::TokenStream,
    mut template_struct: syn::ItemStruct,
    attribute_inputs: minor_parsing::AttrInputs,
    is_tuple: bool,
) -> TokenStream {
    let mut valid_types = std::collections::HashMap::new();

    /* Parsing Template's Types */
    for (field_number, field) in template_struct.fields.iter_mut().enumerate() {
        let type_macro = match helper::get_macro_from_type(&field.ty) {
            Some(x) => x,
            None => continue,
        };

        let parsed = match syn::parse2::<minor_parsing::EitherMacro>(type_macro.tokens) {
            Ok(v) => v.0,
            Err(e) => return e.into_compile_error().into(),
        };

        match parsed.len() == 1 {
            true => {
                field.ty.clone_from(&parsed[0]);
                continue;
            }
            false => field.ty = parsed[0].clone(),
        }

        valid_types.insert(
            match field.ident.as_ref() {
                Some(v) => v.to_string(),
                None => field_number.to_string(),
            },
            std::collections::HashSet::<Type>::from_iter(parsed),
        );
    }

    /* Spitting Tokens Out */
    for derived in attribute_inputs.derived_structs.into_iter() {
        for attr in &template_struct.attrs {
            attr.to_tokens(&mut out);
        }
        derived.vis.to_tokens(&mut out);
        template_struct.struct_token.to_tokens(&mut out);
        derived.name.to_tokens(&mut out);
        template_struct.generics.to_tokens(&mut out);
        if !is_tuple {
            template_struct.generics.where_clause.to_tokens(&mut out);
        }

        let mut fields_token_stream = proc_macro2::TokenStream::new();

        for (field_number, field) in template_struct.fields.iter().enumerate() {
            let pseudo_ident = match field.ident.as_ref() {
                None => field_number.to_string(),
                Some(v) => v.to_string(),
            };

            let field_type = match derived.fields.get(&pseudo_ident) {
                Some(Type::Infer(_)) | None => &field.ty,
                Some(v) => v,
            };

            let (valid_types, is_valid_type) = match valid_types
                .get(&pseudo_ident)
                .map(|v| (Some(v), v.contains(field_type)))
            {
                Some(v) => v,
                None => (None, true),
            };

            match (field_type, is_tuple) {
                (x, _) if !is_valid_type => {
                    custom_compiler_error_msg!(
                        fields_token_stream,
                        "Type \"{}\" (struct \"{}\", field \"{}\") is not part of the specified possible types: {:?}",
                        x.to_token_stream(),
                        derived.name,
                        pseudo_ident,
                        valid_types
                            .unwrap()
                            .iter()
                            .map(|x| x.to_token_stream().to_string())
                            .collect::<Vec<_>>()
                    );
                }
                (Type::Tuple(syn::TypeTuple { elems, .. }), _)
                    if attribute_inputs.settings.delete_empty_tuple_fields && elems.is_empty() =>
                {
                    continue;
                }
                (x, true) => {
                    x.to_tokens(&mut fields_token_stream);
                    Comma::default().to_tokens(&mut fields_token_stream);
                }
                (x, false) => {
                    field.ident.to_tokens(&mut fields_token_stream);
                    field.colon_token.to_tokens(&mut fields_token_stream);
                    x.to_tokens(&mut fields_token_stream);
                    Comma::default().to_tokens(&mut fields_token_stream);
                }
            };
        }

        match is_tuple {
            true => {
                syn::token::Paren::default()
                    .surround(&mut out, |out| fields_token_stream.to_tokens(out));
                template_struct.generics.where_clause.to_tokens(&mut out);
                syn::token::Semi::default().to_tokens(&mut out);
            }
            false => syn::token::Brace::default()
                .surround(&mut out, |out| fields_token_stream.to_tokens(out)),
        }
    }

    if !attribute_inputs.settings.delete_template {
        out.extend::<proc_macro2::TokenStream>(template_struct.into_token_stream());
    }

    out.into()
}

/// Compiler Magic
///
/// this makes an export for LSPs and the compiler to not freak out but allows the syntax
/// for [`macro@make_template`] to be syntactically valid according to the compiler. This
/// relies on the fact that the [`macro@make_template`] macro compiles before this macro.
#[proc_macro]
pub fn either(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    TokenStream::new()
}
