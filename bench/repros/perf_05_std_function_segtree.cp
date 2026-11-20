// Perf Repro #5 (P1 — #12 ioi07p3 3.55x, #14 cco21p5 3.10x):
// `LazySegTree` / `SegTree` and higher-order callbacks emit `std::function<...>`
// type-erased indirect calls on every segment tree node visit (millions of un-inlined
// indirect jumps blocking g++ `-O2` inlining).
// Fix: monomorphize callable parameters as C++ templates (`template <class F>`) or
// stateless functor structs so g++ `-O2` inlines them completely.

use std.lazy.{LazySegTree, AddMin}

let n: Int = read()
domain H = 0..n
var init: [H]Int = 0
var st = LazySegTree<H, Int, Int>(AddMin, init)
var ans = 0
for i in 0..n:
    st.apply(H(0)..=H(i), 1)
    ans += st.get(H(i))
print(ans)
