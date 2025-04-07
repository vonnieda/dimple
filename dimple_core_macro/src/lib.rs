use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields};

fn has_ignore_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("model_ignore")
    })
}

#[proc_macro_attribute]
pub fn model_ignore(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Simply return the item unchanged - this is just a marker attribute
    item
}

#[proc_macro_derive(ModelSupport, attributes(model_ignore))]
pub fn derive_model_support(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let name_str = name.to_string();

    // https://docs.rs/quote/latest/quote/macro.quote.html
    let stream = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Filter out ignored fields
                    let active_fields = fields.named.iter()
                        .filter(|f| !has_ignore_attr(&f.attrs));

                    let from_row = active_fields.clone().map(|f| {
                        let field_name = &f.ident;
                        let field_name_str = field_name.as_ref().unwrap().to_string();
                        
                        quote! {
                            #field_name: row.get(#field_name_str).unwrap()
                        }
                    });

                    let diffs = active_fields.clone().map(|f| {
                        let field_name = &f.ident;
                        let field_name_str = field_name.as_ref().unwrap().to_string();
                        
                        quote! {
                            if self.#field_name != other.#field_name {
                                diff.push(ChangeLog { model: #name_str.to_string(), 
                                    op: "set".to_string(), field: Some(#field_name_str.to_string()), 
                                    value: ChangeLogValue::from(other.#field_name.clone()).val, 
                                    ..Default::default() });
                            }
                        }
                    });

                    let apply_diffs = active_fields.clone().map(|f| {
                        let field_name = &f.ident;
                        let field_name_str = field_name.as_ref().unwrap().to_string();
                        
                        quote! {
                            if &field == #field_name_str {
                                self.#field_name = ChangeLogValue::from(change.value.clone()).into();
                            }
                        }
                    });

                    let columns = active_fields.clone()
                        .map(|f| f.ident.clone().unwrap().to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    let column_positions = active_fields.clone().enumerate()
                        .map(|(i, f)| format!("?{}", i + 1))
                        .collect::<Vec<_>>()
                        .join(",");
                    let params = active_fields.clone().enumerate()
                        .map(|(i, f)| {
                            let field_name = &f.ident;
                            quote! {
                                &self.#field_name
                            }
                        });
                    let upsert = quote! {
                        let sql = format!("INSERT OR REPLACE INTO {} ({}) VALUES ({})", #name_str, #columns, #column_positions);
                        conn.execute(&sql, params!(#(#params,)*)).unwrap();
                    };

                    let params = active_fields.clone().enumerate()
                        .map(|(i, f)| {
                            let field_name = &f.ident;
                            quote! {
                                &self.#field_name
                            }
                        });
                    let insert = quote! {
                        let sql = format!("INSERT INTO {} ({}) VALUES ({})", #name_str, #columns, #column_positions);
                        conn.execute(&sql, params!(#(#params,)*)).unwrap();
                    };

                    let params = active_fields.clone().enumerate()
                        .map(|(i, f)| {
                            let field_name = &f.ident;
                            quote! {
                                &self.#field_name
                            }
                        });
                    let update = quote! {
                        let sql = format!("UPDATE {} SET ({}) = ({}) WHERE key = ?1", #name_str, #columns, #column_positions);
                        conn.execute(&sql, params!(#(#params,)*)).unwrap();
                    };
            
                    quote! {
                        use crate::model::{ChangeLogValue, ChangeLog, Diff, FromRow, Model, LibraryModel};
                        use rusqlite::Row;
                        use rusqlite::params;
                        use std::any::Any;

                        impl FromRow for #name {
                            fn from_row(row: &Row) -> Self {
                                Self {
                                    #(#from_row,)*
                                    ..Default::default()
                                }
                            }
                        }

                        impl LibraryModel for #name {
                            fn upsert(&self, conn: &rusqlite::Connection) {
                                #upsert
                            }
                            
                            fn insert(&self, conn: &rusqlite::Connection) {
                                #insert
                            }

                            fn update(&self, conn: &rusqlite::Connection) {
                                #update
                            }
                        }

                        impl Model for #name {
                            fn key(&self) -> Option<String> {
                                self.key.clone()
                            }
                            
                            fn set_key(&mut self, key: Option<String>) {
                                self.key = key.clone();
                            }
                            
                            fn type_name(&self) -> String {
                                #name_str.to_string()
                            }
                        
                            fn as_any(&self) -> &dyn Any {
                                self
                            }
                        }    

                        impl Diff for #name {
                            fn diff(&self, other: &Self) -> Vec<ChangeLog> {
                                let mut diff = vec![];
                                #(#diffs)*
                                diff
                            }
                            
                            fn apply_diff(&mut self, diff: &[ChangeLog]) {
                                for change in diff {
                                    if change.op == "set" {
                                        if let Some(field) = change.field.clone() {
                                            #(#apply_diffs)*
                                        }
                                    }
                                }
                            }        
                        }
                    }
                },
                _ => quote! {}
            }
        },
        _ => quote! {}
    };    

    TokenStream::from(stream)
}

