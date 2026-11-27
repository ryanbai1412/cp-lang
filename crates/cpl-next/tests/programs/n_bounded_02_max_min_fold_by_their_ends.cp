// expect: ok, output "1 9 7 2\n-9223372036854775807 c"
// rule: R11.3.3
// `max`/`min` over a bounded `T` are monoids with `T.MIN`/`T.MAX` as identity (decision 602): an
// empty fold over a range type gives the range's own ends, not `Int`'s.
type D = 1..=9
let e: List<D> = []
let xs: List<D> = [7, 2, 5]
print(e.fold(max), e.fold(min), xs.fold(max), xs.fold(min))
let ns: List<Int> = []
let cs = ['a', 'c', 'b']
print(ns.fold(max), cs.fold(max))
