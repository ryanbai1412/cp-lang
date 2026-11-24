// expect: ok, output "YES YES YES YES NO\n3 1\n2 0\n1 1\nYES NO 1 1"
// rule: 05.3.5
// `m[k].insert(x)` and `.remove(x)` change the set in the entry `k` finds by the key's own `Ord`, here reversed, and by a key's fields other than its `@cache` ones.
struct W(k: Int)
impl Ord for W:
    fn lt(a: W, b: W) -> Bool = b.k < a.k

struct C:
    @cache memo: List<Int>
    k: Int

fn go(n: Int):
    var mw: Map<W, Set<Int>> = [W(k: 1): Set(), W(k: 2): Set(), W(k: 3): Set()]
    let a = mw[W(k: 1)].insert(n)
    let b = mw[W(k: 1)].insert(n + 1)
    let c = mw[W(k: 3)].insert(n)
    let d = mw[W(k: 1)].remove(n)
    let e = mw[W(k: 1)].remove(n)
    print(yes(a), yes(b), yes(c), yes(d), yes(e))
    for w in mw.keys:
        print(w.k, mw[w].len)
    var mc: Map<C, Set<Int>> = [C(memo: [1], k: 2): Set()]
    let f = mc[C(memo: [], k: 2)].insert(n)
    let g = mc[C(memo: [7], k: 2)].insert(n)
    print(yes(f), yes(g), mc.len, mc[C(memo: [], k: 2)].len)

go(5)
