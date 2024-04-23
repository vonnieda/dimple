use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ModelSupport)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl From<#name> for Model {
            fn from(value: #name) -> Self {
                Self::#name(value)
            }
        }
        
        impl From<&#name> for Model {
            fn from(value: &#name) -> Self {
                Self::#name(value.clone())
            }
        }
        
        impl From<Model> for #name {
            fn from(value: Model) -> Self {
                match value {
                    Model::#name(value) => value,
                    _ => panic!(),
                }
            }
        }
        
        // impl ModelSupport for #name {
        //     fn model(&self) -> Model {
        //         Model::#name(self.clone())
        //     }
        // }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
