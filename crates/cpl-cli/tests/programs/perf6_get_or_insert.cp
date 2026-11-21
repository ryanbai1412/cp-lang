// `m.get_or_insert(k, init)` runs `init` only for a key the map lacks.
domain P = 0..2
var ids = Map<Str, P>()
var calls = 0
fn fresh() -> Int:
    calls += 1
    return calls * 10
fn id(name: Str) -> P:
    return ids.get_or_insert(name, P(ids.len))
print(id("a"), id("b"), id("a"), id("b"))
var seen = HashMap<Int, Int>()
print(seen.get_or_insert(1, fresh()), seen.get_or_insert(1, fresh()), seen.get_or_insert(2, fresh()), calls)
var nested = Map<Int, List<Int>>()
nested.get_or_insert(3, [4])
print(nested.get_or_insert(3, [9]), nested.len)
