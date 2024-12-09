pub mod bi_core;
pub mod parser;
pub mod validator;
pub mod writer;

pub mod prelude {
    pub use crate::bi_core::types::BiField;
    pub use crate::bi_core::error::BiError;
    pub use crate::parser::reader::BiReader;
}
