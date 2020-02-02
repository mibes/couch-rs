use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::{from_value, to_value};
use std::marker::Sized;
use crate::document::Document;

/// Trait that provides methods that can be used to switch between abstract `Document` and concrete `Model` implementors (such as your custom data models)
pub trait Model<T: Serialize + DeserializeOwned + Sized> {
    fn from_document(d: Document) -> T {
        from_value(d.get_data()).unwrap()
    }

    fn to_document(&self) -> Document where Self: Serialize {
        Document::new(to_value(self).unwrap())
    }
}
