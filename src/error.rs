#[derive(Fail, Debug)]
#[fail(display = "Custom error: {}", _0)]
pub struct SofaError(pub String);
