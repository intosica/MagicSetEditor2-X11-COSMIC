use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error on {path:?}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },

    #[error("invalid .mse-set archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("archive is missing the `set` data file")]
    MissingSetFile,

    #[error("set data is not valid UTF-8")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("parse error at line {line}: {msg}")]
    Parse { line: usize, msg: String },

    #[error("unknown game or stylesheet: {0}")]
    UnknownTemplate(String),

    #[error("render error: {0}")]
    Render(String),

    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
}
