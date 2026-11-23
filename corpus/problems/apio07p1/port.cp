// Source: APIO 2007 P1 (Mobiles), https://dmoj.ca/problem/apio07p1
// Problem: a binary tree of n <= 1e5 rods (children of rod i numbered above i, -1 a toy).
//   A swap exchanges a rod's left and right subtrees. Print the fewest swaps after which all
//   toy depths differ by at most one and deeper toys lie left of shallower ones, or -1.
// Solution: bottom-up, a subtree's (deepest, shallowest) toy depth decides it: keep the
//   children if the left's shallowest >= the right's deepest, swap (count it) if the mirrored
//   test holds, else -1; at the root check deepest - shallowest <= 1. O(n).
// cp: 0.1 8ef64b9+dirty
let n: 1..=1e5 = read()
domain Bar = 1..=n
type Kids = (l: ?Bar, r: ?Bar)

fn is_forest(k: [Bar]Kids) -> Bool:
    var parents: [Bar]Int = 0
    for v in Bar:
        if let c = k[v].l:
            if c <= v:
                return false
            parents[c] += 1
        if let c = k[v].r:
            if c <= v:
                return false
            parents[c] += 1
    all(parents[v] <= 1 for v in Bar)

// random binary tree rooted at Bar.first!, for stress tests
fn random_mobile() -> [Bar]Kids:
    var kids: [Bar]Kids = [Bar] _ => (l: None, r: None)
    var free: List<(Bar, Bool)> = [(Bar.first!, true), (Bar.first!, false)]
    for v in Bar:
        if v != Bar.first!:
            let i = rand(0, free.len - 1)
            let (u, left) = free[i]
            free[i] = free.last!
            free.pop()
            if left: kids[u].l = v else: kids[u].r = v
            free.push((v, true))
            free.push((v, false))
    kids

let kids: [Bar]Kids = read(gen: random_mobile, none: -1)
assert is_forest(kids)

var swaps = 0

// toy depths under u as (deepest, shallowest), or none if no swaps can sort them
fn span(u: ?Bar, d: Int) -> ?(Int, Int):
    let v = u else:
        return (d, d)
    let (ld, ls) = span(kids[v].l, d + 1) else: return None
    let (rd, rs) = span(kids[v].r, d + 1) else: return None
    if ls >= rd:
        return (ld, rs)
    if rs >= ld:
        swaps += 1
        return (rd, ls)
    return None

fn answer() -> ?Int:
    let (deep, shallow) = span(Bar.first!, 0) else: return None
    if deep - shallow > 1:
        return None
    return swaps

print(answer() ?? -1)
