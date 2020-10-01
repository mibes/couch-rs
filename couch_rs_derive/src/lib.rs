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
            fn get_id(&self) -> couch_rs::Cow<str> {
                couch_rs::Cow::from(&self._id)
            }

            fn get_rev(&self) -> couch_rs::Cow<str> {
                couch_rs::Cow::from(&self._rev)
            }

            fn set_id(&mut self, id: &str) {
                self._id = id.to_string();
            }

            fn set_rev(&mut self, rev: &str) {
                self._rev = rev.to_string();
            }

            fn merge(&mut self, other: &Self) {
                self.set_id(&other.get_id());
                self.set_rev(&other.get_rev());
            }
        }
    };

    gen.into()
}
