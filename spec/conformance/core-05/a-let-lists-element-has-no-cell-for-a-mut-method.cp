// expect: E0401
// rule: 05.6.3
// A `mut self` method on `ns[0]` changes the cell `ns` lends, so a `let` list can't lend it.
struct N:
    var l: Int
    fn bump(mut self):
        l += 1
let ns: List<N> = [N(1)]
ns[0].bump()  // <- here
print(ns[0].l)
