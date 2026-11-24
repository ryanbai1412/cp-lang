// expect: ok, output "3 4 8"
use std.bag.{SortedSet}
var st: SortedSet<Int> = SortedSet()
for x in [1, 4, 6, 8, 9]: st.add(x)
let xs = st.range(4..9).collect()
print(xs.len, xs[0], xs[2])
