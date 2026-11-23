// Runnable port (proto/cp.py lib-run) of statement-first dpw.cp
// original: corpus/submissions/4133460.cpp

let (n, m): (Int, Int) = read()
domain Pos = 0..=n
domain Rule = 0..m
let rules: [Rule](l: Int, r: Int, a: Int) = read()

var by_r: [Pos]List<(l: Pos, a: Int)> = [Pos] _ => []
for k in Rule:
    by_r[Pos(rules[k].r)].push((l: Pos(rules[k].l), a: rules[k].a))

var st = SegTree<Pos, Int, Int>(max, fill: 0, act: add)
st.set(Pos(0), 0)
for i in Pos(1)..=Pos(n):
    let best_prev = st.fold(Pos(0)..i)
    st.set(i, best_prev)
    for seg in by_r[i]:
        st.apply(seg.l..=i, seg.a)
print(st.fold(Pos(0)..=Pos(n)))
