pub mod parse_error;
pub mod runtime_error;

pub trait ErrorFormat {
    fn error_format(&self) -> String;
}
