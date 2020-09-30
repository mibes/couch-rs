use crate::document::TypedCouchDocument;
use serde::ser::Serialize;
use serde_json::{from_value, to_value, Value};

/// Trait that provides methods that can be used to switch between abstract `Document` and concrete `Model` implementors (such as your custom data models)
pub trait Model<T: TypedCouchDocument> {
    fn from_document(d: Value) -> T {
        from_value(d).unwrap()
    }

    fn to_document(&self) -> Value
    where
        Self: Serialize,
    {
        to_value(self).unwrap()
    }
}
