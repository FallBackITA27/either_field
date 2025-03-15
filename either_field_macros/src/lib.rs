use proc_macro::TokenStream;


#[proc_macro_attribute]
pub fn make_template(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    
    println!("{:#?}",item);
    
    return TokenStream::new();
}

