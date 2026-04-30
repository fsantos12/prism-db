use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitStr, Type};

struct Field {
    ident: Ident,
    r#type: Type,
    column_name: String,
    is_primary_key: bool,
    is_ignored: bool,
}

pub fn derive(input: DeriveInput) -> TokenStream {
    let table_name = parse_table_name(&input);
    let fields = parse_fields(&input);

    if fields.iter().filter(|f| f.is_primary_key).count() == 0 {
        panic!("DbEntity derive requires at least one field with #[db(primary_key)]");
    }

    generate_impl(&input.ident, &table_name, &fields)
}

fn parse_table_name(input: &DeriveInput) -> String {
    for attr in &input.attrs {
        if !attr.path().is_ident("db") {
            continue;
        }

        let mut table_name: Option<String> = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                let value: syn::LitStr = meta.value()?.parse()?;
                table_name = Some(value.value());
            }
            Ok(())
        });

        if let Some(name) = table_name {
            return name;
        }
    }
    panic!("No table name specified for entity. Please use #[db(table = \"table_name\")] on the struct.");
}

fn parse_fields(input: &DeriveInput) -> Vec<Field> {
    let Data::Struct(data) = &input.data else {
        panic!("DbEntity can only be derived for structs");
    };

    let Fields::Named(named) = &data.fields else {
        panic!("DbEntity requires named fields");
    };

    named.named.iter().map(|field| {
        let ident = field.ident.clone().unwrap();
        let r#type = field.ty.clone();
        let mut column_name = ident.to_string();
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
                    column_name = value.value();
                }
                Ok(())
            });
        }

        Field { ident, r#type, column_name, is_primary_key, is_ignored }
    }).collect()
}

fn generate_impl(struct_ident: &Ident, table: &str, fields: &[Field]) -> TokenStream {
    let pk_fields: Vec<&Field> = fields.iter().filter(|f| f.is_primary_key).collect();
    let valid_fields: Vec<&Field> = fields.iter().filter(|f| !f.is_ignored).collect();

    let pks = pk_fields.iter().map(|f| {
        let ident = &f.ident;
        let col_name = &f.column_name;
        quote! {
            (#col_name, ::prism_db_core::types::DbValue::from(self.#ident.clone()))
        }
    });

    let to_db_fields = valid_fields.iter().map(|f| {
        let ident = &f.ident;
        let col_name = &f.column_name;
        quote! {
            (#col_name, ::prism_db_core::types::DbValue::from(self.#ident.clone()))
        }
    });

    let from_db_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        if f.is_ignored {
            quote! {
                #ident: ::std::default::Default::default()
            }
        } else {
            let col_name = &f.column_name;
            let ty = &f.r#type;
            quote! {
                #ident: row.get_by_name(#col_name)
                    .unwrap_or_else(|| panic!("Column '{}' not found", #col_name))
                    .cast::<#ty>()
                    .unwrap_or_else(|| panic!("Failed to cast column '{}'", #col_name))
            }
        }
    });

    quote! {
        #[::async_trait::async_trait]
        impl ::prism_db_orm::DbEntityTrait for #struct_ident {
            fn table_name() -> &'static str {
                #table
            }

            fn primary_key(&self) -> ::std::vec::Vec<(&'static str, ::prism_db_core::types::DbValue)> {
                ::std::vec![
                    #(#pks),*
                ]
            }

            fn to_db(&self) -> ::std::vec::Vec<(&'static str, ::prism_db_core::types::DbValue)> {
                ::std::vec![
                    #(#to_db_fields),*
                ]
            }

            fn from_db(row: &dyn ::prism_db_core::types::DbRow) -> Self {
                Self {
                    #(#from_db_fields),*
                }
            }
        }
    }
}