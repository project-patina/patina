use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Błąd sieci: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Błąd dysku: {0}")]
    Io(#[from] std::io::Error),
    #[error("Niepoprawne dane JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Uszkodzone archiwum: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("{0}")]
    Msg(String),
    #[error("Anulowano")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn msg(s: impl Into<String>) -> Self {
        Error::Msg(s.into())
    }
}

// Commands return errors to the UI as plain strings.
impl Serialize for Error {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

#[macro_export]
macro_rules! bail {
    ($($t:tt)*) => { return Err($crate::Error::Msg(format!($($t)*))) };
}
