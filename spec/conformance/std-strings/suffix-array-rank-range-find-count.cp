// expect: ok, output "1 3\n3 1\n2 4 4"
// rule: 08.1.1
use std.strings.{SuffixArray}
let sa = SuffixArray("banana")
let r = sa.rank_range("ana")
print(r.start, r.end)
print(sa.find_all("ana").collect())
let miss = sa.rank_range("c")
print(sa.count("an"), miss.start, miss.end)
