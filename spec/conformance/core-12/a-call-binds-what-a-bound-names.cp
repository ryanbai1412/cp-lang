// expect: ok, output "3 1 2"
// rule: 12.4
// A call binds what a bound's trait arguments name from the argument it is
// given: `group<S: Seq<T>, ..>` given a `List<Int>` has `T = Int`, so the
// lambda's `x` is an `Int` and the result is a `Map<Int, List<Int>>`.
let g = [1, 2, 1, 3].group(by: x => x % 2)
let odd: List<Int> = g[1]
let counts = [1, 2, 1].tally()
print(odd.len, g[0].len, counts[1])
