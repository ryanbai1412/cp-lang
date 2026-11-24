// expect: ok
// tool: fix
// rewritten: "print(ps.bsearch(p => not (p < 3)), ps.bsearch(x => not small(x)))"
// `cpl fix` turns C++'s and Rust's `a.partition_point(p)`, the first position where `p` turns false, into `bsearch` of the negated test.
fn small(x: Int) -> Bool = x < 5
let ps = [1, 3, 5]
print(ps.partition_point(p => p < 3), ps.partition_point(small))
