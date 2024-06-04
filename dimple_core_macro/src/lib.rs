use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ModelSupport)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let name_str = name.to_string();

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        use crate::model::Model;

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
                    _ => todo!(),
                }
            }
        }
        
        use crate::model::Entity;
        use std::any::Any;

        impl Entity for #name {
            fn key(&self) -> Option<String> {
                self.key.clone()
            }

            fn set_key(&mut self, key: Option<String>) {
                self.key = key;
            }

            fn type_name(&self) -> String {
                #name_str.to_string()
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn model(&self) -> Model {
                Model::#name(self.clone())
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
