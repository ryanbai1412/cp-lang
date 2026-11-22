// error: erased
// Synthesis #3 Leap 4: accessing an erased `()` conditional field outside `if const` is rejected.
const struct Cfg:
    lca: Bool = true

const LITE = Cfg(lca: false)

struct Tree<V, const C: Cfg = LITE>:
    depth: [V]Int
    up: if const C.lca: [V]Int else: ()

    fn bad(self, u: V) -> Int = self.up[u]

let n: 1..=10 = read()
domain V = 1..=n
let d: [V]Int = read()
let t = Tree<V, LITE>(depth: d)
print(t.bad(V.first))
