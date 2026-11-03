use serde::{Serialize, Serializer};

use crate::lang::tr;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{label}: {0}", label = tr("Błąd sieci", "Network error"))]
    Http(#[from] reqwest::Error),
    #[error("{label}: {0}", label = tr("Błąd dysku", "Disk error"))]
    Io(#[from] std::io::Error),
    #[error("{label}: {0}", label = tr("Niepoprawne dane JSON", "Invalid JSON"))]
    Json(#[from] serde_json::Error),
    #[error("{label}: {0}", label = tr("Uszkodzone archiwum", "Damaged archive"))]
    Zip(#[from] zip::result::ZipError),
    #[error("{0}")]
    Msg(String),
    #[error("{}", tr("Anulowano", "Cancelled"))]
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
    ($($t:tt)*) => { return Err($crate::Error::Msg($crate::tr!($($t)*))) };
}
