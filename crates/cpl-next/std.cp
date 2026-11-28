// Core: what every module names unimported, each a `pub use` of an ordinary std module. A
// library declares nothing here that it could not declare in its own file.
use std.ops.{Eq, Ord, Add, Sub, Mul, Div, Rem, Pow, Neg, BitXor, BitOr, BitAnd, Contains, Zero, One, FromInt, Bounded}
/// The traits of `==`, `<`, `+` and the other operators, and of `T.zero`, `T.one` (R08.1.5).
pub use std.ops.{Eq, Ord, Add, Sub, Mul, Div, Rem, Pow, Neg, BitXor, BitOr, BitAnd, Contains, Zero, One, FromInt, Bounded}
use std.clone.{Clone}
/// `x.clone()` (R05.2.2).
pub use std.clone.{Clone}
use std.convert.{Into}
/// `T(x)` (R04.2.28).
pub use std.convert.{Into}
use std.domains.{Domain, Ordered, Index, Cut, RangeEnds}
/// Domains and their cuts (R04.2).
pub use std.domains.{Domain, Ordered, Index, Cut, RangeEnds}
use std.seq.{IndexGet, IndexMut, IndexedMut, Keyed, Positional, Indexed, Slice, Seq, Sized}
/// The indexed writes and defaults every module names unimported (R12.4.6, R07.12.9).
pub use std.seq.{IndexMut, IndexedMut, IndexSet}
pub use std.default.{Default}
/// The maps and sets every module names unimported (R07.2.4).
pub use std.map.{Map, Set}
use std.list.{List, Array, Str}
/// The lists, arrays and strings every module names unimported (R02.3, R04.8).
pub use std.list.{List, Array, Str}
pub use std.io.{io, yes}
use std.optional.{Optional}
/// `?T` and `Some(v)` every module names unimported (R02.4).
pub use std.optional.{Optional, Some}
use std.tuple.{Tuple1, Tuple2, Tuple3, Tuple4}
/// Tuples every module names unimported (R02.5).
pub use std.tuple.{Tuple1, Tuple2, Tuple3, Tuple4}
use std.ranges.{Range, range_of}
/// `lo..hi` and the `[]` of a run of keys (R01.8.4, R07.4.6).
pub use std.ranges.{Range, slice, slice_ends, sub_range, sub_range_ends}
use std.raw.{raw_empty, raw_new, raw_get, raw_set, raw_take, raw_cap, check_key, check_range}
/// Printing, reading, the random stream and `swap`, which every module names unimported.
pub use std.show.{Show, Pack, print, write}
pub use std.read.{Read, Input, read, read_line}
pub use std.stream.{rand}
pub use std.mem.{swap}

/// `max(a, b, ..)` and `min(a, b, ..)`.
pub use std.ops.{max, min, Alike}
/// The trap of an `Int` converted to an enum or cut it is no value of (R02.7, R04.2.17).
pub use std.domains.{check_value}
