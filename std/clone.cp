//! `Clone`: the explicit deep copy (R05.2.2), and the impls of the types that copy by value.
//! Core re-exports it, so no module imports it.
//!
//! start: `Clone`, `Clone.clone`
//! example: `let xs = [1, 2]; var ys = xs.clone(); ys.push(3); xs.len == 2`

/// `x.clone()`: the explicit deep copy, O(size) (R05.2.2). Tuples, records and structs whose
/// fields all have it get it field by field unless they write their own (R08.1.6).
@derive
pub trait Clone:
    /// A deep copy of `self`: changing one never changes the other.
    fn clone(self) -> Self

impl Clone for Int:
    fn clone(a) = a
impl Clone for Char:
    fn clone(a) = a
impl Clone for U64:
    fn clone(a) = a
impl Clone for I128:
    fn clone(a) = a
impl Clone for F64:
    fn clone(a) = a
impl Clone for Bool:
    fn clone(a) = a
impl<D: Domain> Clone for D:
    fn clone(a) = a
