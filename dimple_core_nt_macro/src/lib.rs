use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ModelSupport)]
pub fn derive_model_support(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let name_str = name.to_string();

    // https://docs.rs/quote/latest/quote/macro.quote.html
    let stream = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let from_row = fields.named.iter().map(|f| {
                        let field_name = &f.ident;
                        let field_name_str = field_name.as_ref().unwrap().to_string();
                        
                        quote! {
                            #field_name: row.get(#field_name_str).unwrap()
                        }
                    });

                    let diffs = fields.named.iter().map(|f| {
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

                    let apply_diffs = fields.named.iter().map(|f| {
                        let field_name = &f.ident;
                        let field_name_str = field_name.as_ref().unwrap().to_string();
                        
                        quote! {
                            if &field == #field_name_str {
                                self.#field_name = ChangeLogValue::from(change.value.clone()).into();
                            }
                        }
                    });

                    let columns = fields.named.iter()
                        .map(|f| f.ident.clone().unwrap().to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    let column_positions = fields.named.iter().enumerate()
                        .map(|(i, f)| format!("?{}", i + 1))
                        .collect::<Vec<_>>()
                        .join(",");
                    let params = fields.named.iter().enumerate()
                        .map(|(i, f)| {
                            let field_name = &f.ident;
                            quote! {
                                &self.#field_name
                            }
                        });
                    let upsert = quote! {
                        let sql = format!("INSERT OR REPLACE INTO {} ({}) VALUES ({})", #name_str, #columns, #column_positions);
                        conn.execute(&sql, (#(#params,)*)).unwrap();
                    };

                    quote! {
                        use super::ChangeLogValue;

                        impl FromRow for #name {
                            fn from_row(row: &Row) -> Self {
                                Self {
                                    #(#from_row,)*
                                }
                            }
                        }

                        impl Model for #name {
                            fn table_name() -> String {
                                #name_str.to_string()
                            }
                        
                            fn key(&self) -> Option<String> {
                                self.key.clone()
                            }
                            
                            fn upsert(&self, conn: &rusqlite::Connection) {
                                #upsert
                            }
                            
                            fn set_key(&mut self, key: Option<String>) {
                                self.key = key.clone();
                            }
                            
                            fn log_changes() -> bool {
                                true
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

