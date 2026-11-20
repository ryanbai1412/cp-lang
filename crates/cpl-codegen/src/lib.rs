//! Lowering of a checked program to a single C++17 file.

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_is_named() {
        assert_eq!(super::NAME, "cpl-codegen");
    }
}
