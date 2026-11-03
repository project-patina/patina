//! Language of the messages the backend shows. Without a keyed catalogue: each
//! message carries both of its translations where it is written.
use std::sync::atomic::{AtomicBool, Ordering};

static POLISH: AtomicBool = AtomicBool::new(false);

/// Anything but `pl` means English.
pub fn set(language: &str) {
    POLISH.store(language == "pl", Ordering::Relaxed);
}

pub fn is_polish() -> bool {
    POLISH.load(Ordering::Relaxed)
}

pub fn tr<'a>(pl: &'a str, en: &'a str) -> &'a str {
    if is_polish() { pl } else { en }
}

/// `format!` of the translation for the current language.
#[macro_export]
macro_rules! tr {
    ($pl:literal, $en:literal $(, $arg:expr)* $(,)?) => {
        if $crate::lang::is_polish() { format!($pl $(, $arg)*) } else { format!($en $(, $arg)*) }
    };
}
