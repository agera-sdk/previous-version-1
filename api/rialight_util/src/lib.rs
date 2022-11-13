/// Allows functions to accept any string type.
pub trait AnyStringType {
    fn convert(&self) -> &str;
}

impl AnyStringType for &str {
    fn convert(&self) -> &str {
        self
    }
}

impl AnyStringType for String {
    fn convert(&self) -> &str {
        self.as_ref()
    }
}