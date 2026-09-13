extern crate proc_macro;

#[proc_macro_attribute]
pub fn object(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream { item }

#[proc_macro_derive(Debug)]
pub fn debug(_: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() }

#[proc_macro_derive(From)]
pub fn from(_: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() }

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn serialize(_: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() }

#[proc_macro_derive(Parser, attributes(arg))]
pub fn parser(_: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() }
