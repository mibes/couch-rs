#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(CouchDocument, attributes(serde))]
pub fn derive_couch_doc(input: TokenStream) -> TokenStream {
    impl_derive_couch_doc(&syn::parse(input).unwrap())
}

fn impl_derive_couch_doc(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let gen = quote! {
        impl TypedCouchDocument for #name {
            fn get_id(&self) -> &str {
                &self._id
            }

            fn get_rev(&self) -> &str {
                &self._rev
            }

            fn set_rev(&mut self, rev: &str) {
                self._rev = rev.to_string();
            }

            fn merge_rev(&mut self, other: Self) {
                self._rev = other.get_rev().to_string();
            }
        }
    };

    gen.into()
}
