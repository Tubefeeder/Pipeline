use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(FromUiResource)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    if let Data::Struct(struct_data) = data {
        let fields = struct_data.fields;
        let mut named_fields = Vec::with_capacity(fields.len());
        for field in fields {
            if let Some(name) = field.ident {
                named_fields.push(name);
            } else {
                panic!("All fields have to be named to derive FromUiFile");
            }
        }
        let output = quote! {
            impl #ident {
                pub fn from_resource(file: &str) -> Self {
                    let builder = gtk::Builder::from_resource(file);
                    #(
                        let #named_fields = builder.object(stringify!(#named_fields)).expect(&format!("Failed to get {} from the ui file", stringify!(#named_fields)));
                    )*

                    Self {
                        #(
                          #named_fields
                        ),*
                    }
                }
            }
        };
        output.into()
    } else {
        panic!("Can only derive FromUiFile for structs.");
    }
}
