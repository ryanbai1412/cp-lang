// Core: what every module names unimported, each a `pub use` of an ordinary std module. A
// library declares nothing here that it could not declare in its own file.
use std.ops.{Eq, Ord, Add, Sub, Mul, Div, Rem, Neg, BitXor, BitOr, BitAnd, Contains, Zero, One}
/// The traits of `==`, `<`, `+` and the other operators, and of `T.zero`, `T.one` (R08.1.5).
pub use std.ops.{Eq, Ord, Add, Sub, Mul, Div, Rem, Neg, BitXor, BitOr, BitAnd, Contains, Zero, One}
use std.clone.{Clone}
/// `x.clone()` (R05.2.2).
pub use std.clone.{Clone}
use std.convert.{Into}
/// `T(x)` (R04.2.28).
pub use std.convert.{Into}
use std.domains.{Domain, Cut}
/// Domains and their cuts (R04.2).
pub use std.domains.{Domain, Cut}
use std.seq.{IndexGet, IndexMut, IndexedMut, Keyed, Positional, Indexed, Slice, Seq, Sized}
/// The indexed writes and defaults every module names unimported (R12.4.6, R07.12.9).
pub use std.seq.{IndexMut, IndexedMut, IndexSet}
pub use std.default.{Default}
/// The maps and sets every module names unimported (R07.2.4).
pub use std.map.{Map, Set}
use std.list.{List, Array, Str}
/// The lists, arrays and strings every module names unimported (R02.3, R04.8).
pub use std.list.{List, Array, Str, yes}
use std.optional.{Optional}
/// `?T` and `Some(v)` every module names unimported (R02.4).
pub use std.optional.{Optional, some}
use std.tuple.{Tuple1, Tuple2, Tuple3, Tuple4}
/// Tuples every module names unimported (R02.5).
pub use std.tuple.{Tuple1, Tuple2, Tuple3, Tuple4}
use std.ranges.{Range, range_of}
/// `lo..hi` and the `[]` of a run of keys (R01.8.4, R07.4.6).
pub use std.ranges.{Range, slice, slice_ends, sub_range, sub_range_ends}
use std.raw.{raw_empty, raw_new, raw_get, raw_set, raw_take, raw_cap, check_key, check_range}

/// Traps R0102 unless `ok`: a value its type doesn't have, as an `Int` converted to an enum that
/// is no variant's value (R02.7), or to a cut past a domain's ends (R04.2.17).
@variant_check
pub fn check_value(ok: Bool) = @intrinsic("value_check")

fn read_array<D: Domain>() -> [D]Int:
    var a: [D]Int = 0
    for v in D: a[v] = read()
    a

fn write_list(xs: List<Int>):
    var first = true
    for x in xs:
        if not first: write(" ")
        write(x)
        first = false

fn write_array<D: Domain>(xs: [D]Int):
    var first = true
    for x in xs:
        if not first: write(" ")
        write(x)
        first = false

/// The largest of two or more values, the first of equal ones (R11.3.3).
pub fn max<T: Ord, Ts: Alike<T>>(a: T, b: T, ..more: Ts) -> T = more.max_with(if b > a: b else: a)

/// The smallest of two or more values, the first of equal ones (R11.3.3).
pub fn min<T: Ord, Ts: Alike<T>>(a: T, b: T, ..more: Ts) -> T = more.min_with(if b < a: b else: a)

/// A tuple whose fields are all `T`: what `max` and `min` take past their first two values.
pub trait Alike<T>:
    /// The largest of `m` and the fields, the first of equal ones.
    fn max_with(self, m: T) -> T
    /// The smallest of `m` and the fields, the first of equal ones.
    fn min_with(self, m: T) -> T
impl<T: Ord> Alike<T> for ():
    fn max_with(self, m: T) -> T = m
    fn min_with(self, m: T) -> T = m
