//! Diagnostics: source spans, stable error codes, and rendering shared by every tool.

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_is_named() {
        assert_eq!(super::NAME, "cpl-diag");
    }
}
