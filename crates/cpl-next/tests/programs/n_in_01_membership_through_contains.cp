// expect: ok, output "4 YES YES YES NO\nYES NO YES NO\nNO YES\nNO YES YES"
// `x in s` is `s`'s `Contains` impl: core's for a range, std's for a set or a map, a user's own;
// `x in V` on a domain type asks whether integer `x` is one of its positions.
use std.map.{Set, Map}
domain V = 1..=6
struct Evens(n: Int)
impl Contains<Int> for Evens:
    fn contains(self, x: Int) -> Bool = x % 2 == 0 and 0 <= x and x < 2 * n
let r = V(2)..=V(5)
var c = 0
for v in V:
    if v in r: c += 1
print(c, yes(3 in 0..5), yes(7 not in (0..10).step(2)), yes(6 in (0..10).step(2)), yes(4 in (9..0).step(1)))
var s = Set<Int>()
s.insert(3)
var m = Map<Int, Int>()
m[5] = 1
print(yes(3 in s), yes(4 in s), yes(5 in m), yes(s.contains(9)))
print(yes(7 in Evens(4)), yes(6 in Evens(4)))
print(yes(0 in V), yes(6 in V), yes(7 not in V))
