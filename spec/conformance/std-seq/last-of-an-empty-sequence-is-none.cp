// expect: ok, output "YES YES YES"
// rule: 07.4.2
// `last` of an empty List, Range or Str through `Bidir` is `None`.
fn lst<S: Bidir<T>, T>(s: S) -> ?T = s.last
let e: List<Int> = []
print(yes(lst(e) == None), yes(lst(3..3) == None), yes(lst("") == None))
