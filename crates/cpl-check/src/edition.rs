//! Which edition of the language a program is checked and run as. CP Lang
//! 0.3 (`spec/`, with `spec/std` as its standard library) replaces 0.2
//! (`stdlib/`) and is the default; 0.2 stays selectable only while its
//! tests migrate, and is deleted with them.

use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edition {
    V02,
    V03,
}

/// The edition for this process: 0.3 unless `CPL_EDITION=0.2` (and
/// `CPL_SPEC` is unset). Read once.
pub fn edition() -> Edition {
    static EDITION: OnceLock<Edition> = OnceLock::new();
    *EDITION.get_or_init(|| {
        if std::env::var_os("CPL_SPEC").is_some() {
            return Edition::V03;
        }
        match std::env::var("CPL_EDITION").as_deref() {
            Ok("0.2") => Edition::V02,
            _ => Edition::V03,
        }
    })
}

pub fn is_v03() -> bool {
    edition() == Edition::V03
}
