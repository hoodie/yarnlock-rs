pub use failure::Error;
use indent_tokenizer;

/// Wraps `ident_tokenizer::Error`
#[derive(Debug, Fail)]
#[fail(display = "invalid indentation in line {:?}", _0)]
pub struct IndentationFail(pub indent_tokenizer::Error);
