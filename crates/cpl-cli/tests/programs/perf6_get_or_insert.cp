// `m.get_or_insert_with(k, f)` runs `f` only for a key the map lacks;
// `m.get_or_insert(k, v)` works `v` out first, like any argument.
use std.seq.{get_or_insert_with}
domain P = 0..2
var ids = Map<Str, P>()
var calls = 0
fn fresh() -> Int:
    calls += 1
    return calls * 10
var next = 0
fn id(name: Str) -> P:
    next = ids.len
    return ids.get_or_insert_with(name, () => P(next))
print(id("a"), id("b"), id("a"), id("b"))
var seen = Map<Int, Int>()
print(seen.get_or_insert_with(1, fresh), seen.get_or_insert_with(1, fresh), seen.get_or_insert_with(2, fresh), calls)
print(seen.get_or_insert(1, fresh()), seen.get_or_insert(3, fresh()), calls)
var nested = Map<Int, List<Int>>()
nested.get_or_insert(3, [4])
print(nested.get_or_insert(3, [9]), nested.len)
