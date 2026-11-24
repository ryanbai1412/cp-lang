// expect: ok, output "YES YES YES YES\n2 4 6 | 3 2 1"
// rule: 08.1.2
// `filter`, `map`, `rev` and `items` are std view structs that keep their source: `.src` hands back the same sequence, and walking the view reads through it.
use std.seq.{filter, map, items}
let a = [1, 2, 3]
let m = map(a, x => x * 2)
let r = a.rev
print(yes(filter(a, x => x > 1).src == a), yes(m.src == a), yes(r.src == a), yes(items(a).src == a))
var walked = ""
for x in m: walked += "{x} "
walked += "|"
for x in r: walked += " {x}"
print(walked)
