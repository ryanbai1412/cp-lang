// expect: ok, output "2\n4\n6"
// rule: std.seq.map
// `map` of a list is a `Mapped<List<Int>, Int, Int>`; its `Indexed` impl's bound `S: Indexed<D, T>` is matched onto its own source, which terminates.
use std.seq.{map}
for x in map([1, 2, 3], x => x * 2): print(x)
