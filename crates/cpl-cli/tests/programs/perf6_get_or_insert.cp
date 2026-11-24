// `m.get_or_insert(k, v)` works `v` out first, like any argument, and stores it
// only for a key the map lacks.
var calls = 0
fn fresh() -> Int:
    calls += 1
    return calls * 10
var seen = Map<Int, Int>()
print(seen.get_or_insert(1, fresh()), seen.get_or_insert(1, fresh()), seen.get_or_insert(2, fresh()), calls)
print(seen.get_or_insert(1, fresh()), seen.get_or_insert(3, fresh()), calls)
var nested = Map<Int, List<Int>>()
nested.get_or_insert(3, [4])
print(nested.get_or_insert(3, [9]), nested.len)
