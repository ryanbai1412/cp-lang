// Synthesis #3 Leap 4: `const struct` POD configs + `if const` compile-time branching & field erasure.
const struct TreeCfg:
    lca: Bool = true
    euler: Bool = true
    stride: Int = 1

const DEFAULT = TreeCfg()
const LITE = TreeCfg(lca: false, euler: false, stride: 2)

struct Rooted<V, const C: TreeCfg = DEFAULT>:
    depth: [V]Int
    up: if const C.lca: [V]Int else: ()
    tin: if const C.euler: [V]Int else: ()

    fn tag(self) -> Int:
        let base = if const C.lca: 100 else: 0
        if const C.euler:
            return base + C.stride + self.tin[V.first]
        else:
            // Dead branch check: `self.up` and `self.tin` are erased `()` here, so only this branch is type-checked!
            return base + C.stride + self.depth[V.first]

    fn score(self, u: V) -> Int:
        if const C.lca:
            return self.depth[u] + self.up[u]
        else:
            return self.depth[u] * C.stride

let n: 1..=10 = read()
domain V = 1..=n
let d: [V]Int = read()
let up_arr: [V]Int = read()
let tin_arr: [V]Int = read()

let full = Rooted<V>(depth: d, up: up_arr, tin: tin_arr)
let lite = Rooted<V, LITE>(depth: d)

print(full.tag(), full.score(V.last), lite.tag(), lite.score(V.last))
