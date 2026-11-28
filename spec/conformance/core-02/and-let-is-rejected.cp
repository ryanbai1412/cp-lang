// expect: E0503
// rule: 02.4.5
// has: "`,`"
// A `let` condition is joined to the next one with `,`.
fn find(xs: List<Int>, k: Int) -> ?Int:
    for i in 0..xs.len:
        if xs.get(i) == k: return i
    None
if let i = find([5, 7], 7) and i > 0: print(i) // <- here
