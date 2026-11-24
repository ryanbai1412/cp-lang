// Source: APIO 2007 P1 (Mobiles), https://dmoj.ca/problem/apio07p1
// limits: 1000 ms, 32 MB
// Problem: a binary tree of n <= 1e5 rods (children of rod i numbered above i, -1 a toy).
//   A swap exchanges a rod's left and right subtrees. Print the fewest swaps after which all
//   toy depths differ by at most one and deeper toys lie left of shallower ones, or -1.
// Solution: bottom-up, a subtree's (deepest, shallowest) toy depth decides it: keep the
//   children if the left's shallowest >= the right's deepest, swap (count it) if the mirrored
//   test holds, else -1; at the root check deepest - shallowest <= 1. O(n).
// Bounds: n: 1..=1e5
let n: 1..=1e5 = read()
domain Bar = 1..=n
let kids: [Bar](l: ?Bar, r: ?Bar) = read(none: -1)

struct Span(deep: Int, shallow: Int, swaps: Int)

// the toy depths under u and the swaps that sort them, or none if no swaps can
fn span(u: ?Bar, d: Int) -> ?Span:
    let v = u else:
        return Span(d, d, 0)
    let l = span(kids[v].l, d + 1) else: return None
    let r = span(kids[v].r, d + 1) else: return None
    let swaps = l.swaps + r.swaps
    if l.shallow >= r.deep:
        return Span(l.deep, r.shallow, swaps)
    if r.shallow >= l.deep:
        return Span(r.deep, l.shallow, swaps + 1)
    None

fn answer() -> ?Int:
    let s = span(Bar.first!, 0) else: return None
    if s.deep - s.shallow > 1:
        return None
    s.swaps

print(answer() ?? -1)
