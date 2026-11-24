// expect: ok, output "2 2 2"
// rule: 08.1.6
// A struct whose fields all have `Hash` derives it field by field, a range field too.
use std.strings.{suffix_tree_edges}
struct E(a: Int, r: Range<Int>)
let es = [E(1, 0..2), E(1, 0..2), E(2, 1..3)]
let edges = suffix_tree_edges("aa")
print(es.distinct().len, HashSet(es).len, (edges + edges).distinct().len)
