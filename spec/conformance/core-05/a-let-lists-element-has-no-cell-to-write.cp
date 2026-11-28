// expect: E0401
// rule: 05.6.3
// Writing a field of `ns[0]` writes the cell `ns` lends, so a `let` list, which
// lends none, can't have it written: the fix is `var ns`.
struct N:
    var l: Int
let ns: List<N> = [N(1)]
ns[0].l = 7  // <- here
print(ns[0].l)
