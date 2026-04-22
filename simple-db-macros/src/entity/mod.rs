use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitStr, Type};

struct ParsedField {
    ident: Ident,
    ty: Type,
    column: String,
    is_primary_key: bool,
    is_ignored: bool,
}

/// Entry point called from `lib.rs`.
pub fn derive(input: DeriveInput) -> TokenStream {
    let collection = parse_collection_name(&input);
    let fields = parse_fields(&input);

    if fields.iter().filter(|f| f.is_primary_key).count() == 0 {
        panic!("DbEntity derive requires at least one field with #[db(primary_key)]");
    }

    generate_impl(&input.ident, &collection, &fields)
}

fn parse_collection_name(input: &DeriveInput) -> String {
    for attr in &input.attrs {
        if !attr.path().is_ident("db") {
            continue;
        }
        let mut collection: Option<String> = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("collection") {
                let value: LitStr = meta.value()?.parse()?;
                collection = Some(value.value());
            }
            Ok(())
        });
        if let Some(c) = collection {
            return c;
        }
    }
    panic!("DbEntity derive requires #[db(collection = \"table_name\")] on the struct");
}

fn parse_fields(input: &DeriveInput) -> Vec<ParsedField> {
    let Data::Struct(data) = &input.data else {
        panic!("DbEntity can only be derived for structs");
    };
    let Fields::Named(named) = &data.fields else {
        panic!("DbEntity requires named fields");
    };

    named.named.iter().map(|field| {
        let ident = field.ident.clone().unwrap();
        let ty = field.ty.clone();
        let mut column = ident.to_string();
        let mut is_primary_key = false;
        let mut is_ignored = false;

        for attr in &field.attrs {
            if !attr.path().is_ident("db") {
                continue;
            }
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("primary_key") {
                    is_primary_key = true;
                } else if meta.path.is_ident("ignore") {
                    is_ignored = true;
                } else if meta.path.is_ident("column") {
                    let value: LitStr = meta.value()?.parse()?;
                    column = value.value();
                }
                Ok(())
            });
        }

        ParsedField { ident, ty, column, is_primary_key, is_ignored }
    }).collect()
}

fn generate_impl(struct_ident: &Ident, collection: &str, fields: &[ParsedField]) -> TokenStream {
    let pk_entries = fields.iter().filter(|f| f.is_primary_key).map(|f| {
        let ident = &f.ident;
        let col = &f.column;
        quote! {
            (#col, ::simple_db::types::DbValue::from(self.#ident.clone()))
        }
    });

    let to_db_entries = fields.iter().filter(|f| !f.is_ignored).map(|f| {
        let ident = &f.ident;
        let col = &f.column;
        quote! {
            (#col, ::simple_db::types::DbValue::from(self.#ident.clone()))
        }
    });

    let from_db_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        if f.is_ignored {
            quote! { #ident: ::std::default::Default::default() }
        } else {
            let col = &f.column;
            let ty = &f.ty;
            quote! {
                #ident: row.get_by_name(#col)
                    .and_then(|v| {
                        <#ty as ::std::convert::TryFrom<::simple_db::types::DbValue>>::try_from(v).ok()
                    })
                    .unwrap_or_default()
            }
        }
    });

    quote! {
        #[automatically_derived]
        impl ::simple_db::DbEntityTrait for #struct_ident {
            fn collection_name() -> &'static str {
                #collection
            }

            fn primary_key(&self) -> ::std::vec::Vec<(&'static str, ::simple_db::types::DbValue)> {
                vec![#(#pk_entries),*]
            }

            fn to_db(&self) -> ::std::vec::Vec<(&'static str, ::simple_db::types::DbValue)> {
                vec![#(#to_db_entries),*]
            }

            fn from_db(row: &dyn ::simple_db::types::DbRow) -> Self {
                Self {
                    #(#from_db_fields),*
                }
            }
        }
    }
}
