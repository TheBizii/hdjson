// All possible JSON values as defined by the RFC-8259 standard.
// https://www.rfc-editor.org/rfc/rfc8259.html#section-3

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(i64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}
