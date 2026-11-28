//! `Default`: the value a `Map`'s absent key starts at under `m[k] op= v` (R07.12.9).
use std.list.{List, Str}

/// The value `T.default()` a `Map`'s absent key starts at under `m[k] op= v` (R07.12.9).
pub trait Default:
    fn default() -> Self
impl Default for Int:
    fn default() -> Int = 0
impl Default for U64:
    fn default() -> U64 = U64(0)
impl Default for I128:
    fn default() -> I128 = I128(0)
impl Default for Bool:
    fn default() -> Bool = false
impl Default for Str:
    fn default() -> Str = ""
impl<T> Default for List<T>:
    fn default() -> List<T> = []
