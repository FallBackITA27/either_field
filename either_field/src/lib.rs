use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{GenericParam, Generics, Ident, Type, parse_macro_input, punctuated::Punctuated};

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

    let is_tuple = match template_struct.fields {
        syn::Fields::Unit => {
            custom_compiler_error_msg!(out, "Unit structs have no fields to do anything about.");
            return out.into();
        }
        syn::Fields::Unnamed(_) => true,
        syn::Fields::Named(_) => false,
    };

    if is_tuple && !attribute_inputs.settings.generate_structs {
        custom_compiler_error_msg!(out, "Tuple structs require `GenStructs` set to `true`.");
        return out.into();
    }

    match attribute_inputs.settings.generate_structs {
        false => gen_types(out, template_struct, attribute_inputs),
        true => gen_structs(out, template_struct, attribute_inputs, is_tuple),
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
            match derived.fields.get(ident) {
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
                                "Type \"{}\" is not part of the specified possible types: {:?}",
                                v.to_token_stream(),
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
    // this also has to match the order of the generics
    let mut ordered_idents_and_types: Vec<Vec<Type>> = vec![];

    for (field_number, field) in template_struct.fields.iter_mut().enumerate() {
        let type_macro = match helper::get_macro_from_type(&field.ty) {
            Some(x) => x,
            None => {
                ordered_idents_and_types.push(vec![]);
                continue;
            }
        };

        let tokens: TokenStream = type_macro.tokens.clone().into();
        let parsed = parse_macro_input!(tokens as minor_parsing::EitherMacro).0;

        if is_tuple {
            field.ident = Some(Ident::new(
                format!("_{field_number}").as_str(),
                Span::call_site(),
            ));
        }
        field.ty = parsed[0].clone();
        ordered_idents_and_types.push(parsed);
    }

    for derived in attribute_inputs.derived_structs {
        let mut out_struct = template_struct.clone();
        out_struct.ident = derived.name;
        out_struct.vis = derived.vis;
        for (field_num, field) in out_struct.fields.iter_mut().enumerate() {
            let possible_types = &ordered_idents_and_types[field_num];
            let get = derived.fields.get(field.ident.as_ref().unwrap());
            if is_tuple {
                field.ident = None;
            }
            if get.is_none() {
                continue;
            }
            let v = get.unwrap();
            if let Type::Infer(_) = v {
                continue;
            }
            match possible_types.contains(v) {
                true => field.ty = v.clone(),
                false => {
                    custom_compiler_error_msg!(
                        out,
                        "Type \"{}\" is not part of the specified possible types: {:?}",
                        v.to_token_stream(),
                        possible_types
                            .iter()
                            .map(|x| format!("{}", x.to_token_stream()))
                            .collect::<Vec<_>>()
                    );
                    return out.into();
                }
            }
        }

        if attribute_inputs.settings.delete_empty_tuple_fields {
            // here are the fields, that aren't being left out
            match out_struct.fields {
                syn::Fields::Unit => {
                    // This should never happen anyways, as template struct is already shielding us
                    // from this case
                    custom_compiler_error_msg!(
                        out,
                        "Unit structs have no fields to do anything about."
                    );
                    return out.into();
                }
                syn::Fields::Named(named) => {
                    let old_punctuation = named.named;
                    let brace = named.brace_token;
                    out_struct.fields = syn::Fields::Named(syn::FieldsNamed {
                        brace_token: brace,
                        named: old_punctuation
                            .into_iter()
                            .filter_map(|item| {
                                if let Type::Tuple(ref x) = item.ty {
                                    if x.elems.is_empty() {
                                        return None;
                                    }
                                }
                                Some(item)
                            })
                            .collect(),
                    });
                }
                syn::Fields::Unnamed(unnamed) => {
                    let old_punctuation = unnamed.unnamed;
                    let brace = unnamed.paren_token;
                    out_struct.fields = syn::Fields::Unnamed(syn::FieldsUnnamed {
                        paren_token: brace,
                        unnamed: old_punctuation
                            .into_iter()
                            .filter_map(|item| {
                                if let Type::Tuple(ref x) = item.ty {
                                    if x.elems.is_empty() {
                                        return None;
                                    }
                                }
                                Some(item)
                            })
                            .collect(),
                    });
                }
            };
        }

        out.extend::<proc_macro2::TokenStream>(out_struct.into_token_stream());
    }
    if !attribute_inputs.settings.delete_template {
        if is_tuple {
            for field in template_struct.fields.iter_mut() {
                field.ident = None;
            }
        }
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
