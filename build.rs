use std::{
    collections::BTreeMap,
    env,
    io::{self, Write},
    path::Path,
    process::Command,
};

use heck::{CamelCase, SnakeCase};
use proc_macro2::Literal;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Spec {
    tables: BTreeMap<String, TableSpec>,
}

#[derive(Deserialize)]
pub struct TableSpec {
    columns: Vec<ColumnSpec>,
}

#[derive(Deserialize)]
pub struct ColumnSpec {
    name: String,
    ty: ValueType,
    nullable: bool,
}

#[derive(Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    /// The NULL value
    Nothing,
    /// A 32-bit signed integer
    Integer,
    /// A 32-bit IEEE floating point number
    Float,
    /// A long string
    Text,
    /// A boolean
    Boolean,
    /// A 64 bit integer
    BigInt,
    /// An (XML?) string
    VarChar,
}

fn run() -> Result<(), io::Error> {
    let data = include_str!("spec.json");
    let spec: Spec = serde_json::from_str(data).unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut cspecs = Vec::with_capacity(spec.tables.len());
    let mut tables = Vec::with_capacity(spec.tables.len());
    let mut rows = Vec::with_capacity(spec.tables.len());

    let field_into_nothing = quote!(field_into_nothing);

    for (name, tspec) in &spec.tables {
        let t = name.to_camel_case();
        let strname = Literal::string(name);
        let tname = format_ident!("{}Table", t);
        let rname = format_ident!("{}Row", t);
        let csname = format_ident!("{}Column", t);

        let ccount = tspec.columns.len();
        let mut cslist = Vec::with_capacity(ccount);
        let mut col_to_string_list = Vec::with_capacity(ccount);
        let mut col_serialize_list = Vec::with_capacity(ccount);
        let mut exlist = Vec::with_capacity(ccount);
        //let mut ser_stmts = Vec::with_capacity(ccount);
        let mut cmatch = Vec::with_capacity(ccount);

        for cspec in &tspec.columns {
            let cn = &cspec.name;
            let c = cn.to_snake_case();
            let cf = cn.to_camel_case();
            let c = match c.as_str() {
                "type" => String::from("r#type"),
                "static" => String::from("r#static"),
                _ => c,
            };
            let cname = format_ident!("{}", c);
            let cfname = format_ident!("{}", cf);
            let doc = format!("Index of column `{}`", &cn);
            cslist.push(quote! {
                #[doc = #doc]
                #cfname
            });
            col_to_string_list.push(quote! {
                Self::#cfname => #cn
            });
            /*ser_stmts.push(quote! {
                s.serialize_field(#cn, &self.#cname())?;
            });*/

            let doc = format!("Get the data in column `{}`", &cspec.name);
            let (return_type, map_fn) = match &cspec.ty {
                ValueType::Nothing => (quote!(()), field_into_nothing.clone()),
                ValueType::Integer => (quote!(i32), quote!(Field::into_opt_integer)),
                ValueType::Float => (quote!(f32), quote!(Field::into_opt_float)),
                ValueType::Text => (quote!(&'a Latin1Str), quote!(Field::into_opt_text)),
                ValueType::Boolean => (quote!(bool), quote!(Field::into_opt_boolean)),
                ValueType::BigInt => (quote!(i64), quote!(Field::into_opt_big_int)),
                ValueType::VarChar => (quote!(&'a Latin1Str), quote!(Field::into_opt_varchar)),
            };

            col_serialize_list.push(quote! {
                Self::#cfname => s.serialize_field(#cn, &((#map_fn)(value)))
            });

            let columns = quote!(super::columns::#csname);
            let err = Literal::string(&format!("Missing column {} in {}", cn, t));
            let f = if cspec.nullable {
                quote! {
                    #[doc = #doc]
                    pub fn #cname(&self) -> Option<#return_type> {
                        if let Some(index) = self.table.get_col(#columns::#cfname) {
                            self.row.field_at(index).and_then(#map_fn)
                        } else {
                            ::log::warn!(#err);
                            None
                        }
                    }
                }
            } else {
                // Fix some clippy lints
                let ret = if cspec.ty == ValueType::Nothing {
                    quote!()
                } else {
                    quote!( -> #return_type)
                };
                // FIXME: impl Default for &Latin1Str upstream
                let default = if matches!(cspec.ty, ValueType::Text | ValueType::VarChar) {
                    quote!(EMPTY_L1_STR)
                } else {
                    quote!(Default::default())
                };
                quote! {
                    #[doc = #doc]
                    pub fn #cname(&self) #ret {
                        let index = self.table.get_col(#columns::#cfname).expect(#err);
                        let value = self.row.field_at(index).expect("defined field missing, FDB corrupt");
                        #map_fn(value).unwrap_or_else(|| {
                            let pk = self.row.field_at(0).unwrap_or(Field::Nothing);
                            log::warn!("Missing non-nullable field {} in {} in row {:?}", #cn, #t, pk);
                            #default
                        })
                    }
                }
            };
            exlist.push(f);

            let b = Literal::byte_string(cn.as_bytes());
            cmatch.push(quote! {
                #b => Some(super::columns::#csname::#cfname)
            });
        }

        let doc = format!("Columns in table `{}`\n\nSee also: [`{0}.html>", &name,);
        cspecs.push(quote! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[doc = #doc]
            pub enum #csname {
                #(#cslist),*
            }

            impl crate::TypedColumn for #csname {
                fn to_static_str(&self) -> &'static str {
                    match self {
                        #(#col_to_string_list),*
                    }
                }

                fn serialize_struct_field<S: ::serde::ser::SerializeStruct>(&self, s: &mut S, value: Field) -> Result<(), S::Error> {
                    match self {
                        #(#col_serialize_list),*
                    }
                }
            }
        });

        let doc = format!(
            "## Table `{0}`\n\nSee also: [`{1}`][super::tables::{1}]",
            &name, tname
        );
        tables.push(quote! {
            #[doc = #doc]
            #[derive(Clone)]
            pub struct #tname<'a> {
                inner: Table<'a>,
                pub(crate) col: BTreeMap<super::columns::#csname, usize>,
            }

            impl<'a> TypedTable<'a> for #tname<'a> {
                type Column = super::columns::#csname;
                const NAME: &'static str = #strname;
                fn as_raw(&self) -> Table<'a> {
                    self.inner
                }

                fn new(inner: Table<'a>) -> Self {
                    let mut col = BTreeMap::new();
                    for (i, c) in inner.column_iter().enumerate() {
                    	let key = match c.name_raw().as_bytes() {
                    		#(#cmatch),*,
                    		_ => None
                    	};
                    	if let Some(key) = key {
	                    	col.insert(key, i);
						}
                    }
                    Self { inner, col }
                }
            }

            impl<'a> #tname<'a> {
            	/// Get the "real" index of the well-known column
                pub fn get_col(&self, col: super::columns::#csname) -> Option<usize> {
                    self.col.get(&col).copied()
                }

				/// Iterate over all rows
                pub fn row_iter<'b>(&'b self) -> crate::RowIter<'a, 'b, super::rows::#rname<'a, 'b>> {
					crate::RowIter::new(self)
                }

				/// Iterate over all rows that have a specific key
                pub fn key_iter<'b: 'a>(&'b self, key: i32) -> impl Iterator<Item = super::rows::#rname<'a, 'b>> {
					let hash = key as usize % self.as_raw().bucket_count();
                    self.as_raw()
                        .bucket_at(hash)
                        .unwrap()
                        .row_iter()
                        .filter(move |row| row.field_at(0) == Some(Field::Integer(key)))
                        .map(move |inner| <super::rows::#rname as TypedRow<'a,'b>>::new(inner, self))
                }
            }
        });

        let doc = format!(
            "## Row of the  `{}` table\n\n See also: [`{1}`][`super::tables::{1}`]",
            &name, tname
        );
        rows.push(quote! {
            #[doc = #doc]
            #[derive(Copy, Clone)]
            pub struct #rname<'a, 'b> {
                row: Row<'a>,
                table: &'b super::tables::#tname<'a>,
            }

            impl<'a, 'b> crate::TypedRow<'a, 'b> for #rname<'a, 'b> {
                type Table = super::tables::#tname<'a>;

                fn new(row: Row<'a>, table: &'b Self::Table) -> Self {
                    Self { row, table }
                }
            }

            impl<'a, 'b> #rname<'a, 'b> {
                #(#exlist)*
            }

            impl<'a, 'b> serde::Serialize for #rname<'a, 'b> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer {
                    let mut s = serializer.serialize_struct(#name, #ccount)?;
                    for (col, index) in &self.table.col {
                        col.serialize_struct_field(&mut s, self.row.field_at(*index).unwrap_or_else(|| panic!("Missing known column {:?} in {}", col, #t)))?;
                    }
                    //#(#ser_stmts)*
                    s.end()
                }
            }
        });
    }

    let field_into_nothing_impl = quote! {
        fn #field_into_nothing(_: Field) -> Option<()> {
            Some(())
        }
    };

    let columns = quote! {
        use ::assembly_fdb::mem::Field;
        #field_into_nothing_impl

        #(#cspecs)*
    };

    let tables = quote! {
        use assembly_fdb::mem::{Table, Field};
        use std::collections::BTreeMap;
        use crate::{TypedTable, TypedRow};

        #(#tables)*
    };

    let rows = quote! {
        use latin1str::Latin1Str;
        use assembly_fdb::mem::{Field, Row};
        use serde::ser::SerializeStruct;
        use crate::TypedColumn;

        const EMPTY_L1_STR: &Latin1Str = unsafe { Latin1Str::from_bytes_unchecked(&[]) };
        #field_into_nothing_impl

        #(#rows)*
    };

    let out_path = Path::new(&out_dir);
    let columns_file = out_path.join("columns.rs");
    let tables_file = out_path.join("tables.rs");
    let rows_file = out_path.join("rows.rs");

    let (c, t, r) = (
        columns_file.display().to_string(),
        tables_file.display().to_string(),
        rows_file.display().to_string(),
    );
    let generated = quote! {
        #[path = #c]
        /// All column types
        pub mod columns;
        #[path = #t]
        /// All table types
        pub mod tables;
        #[path = #r]
        /// All row types
        pub mod rows;
    };

    let generated_file = out_path.join("generated.rs");
    std::fs::write(&columns_file, format!("{}", columns))?;
    std::fs::write(&tables_file, format!("{}", tables))?;
    std::fs::write(&rows_file, format!("{}", rows))?;
    std::fs::write(&generated_file, format!("{}", generated))?;

    match Command::new("rustfmt")
        .arg(&columns_file)
        .arg(&tables_file)
        .arg(&rows_file)
        .arg(&generated_file)
        .spawn()
    {
        Ok(mut p) => {
            p.wait()?;
        }
        Err(e) => {
            println!("cargo:warning=rustfmt: {}", e);
        }
    }

    if option_env!("CARGO_PRIMARY_PACKAGE").is_some() {
        if let Ok(path) = std::env::var("GITHUB_OUTPUT") {
            let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
            writeln!(file, "crate_version={}", env!("CARGO_PKG_VERSION"))?;
            writeln!(file, "crate_name={}", env!("CARGO_PKG_NAME"))?;
        }
    }

    Ok(())
}

fn main() {
    run().unwrap()
}
