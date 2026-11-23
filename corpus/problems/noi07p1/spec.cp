// NOI 2007 "Social Network": importance of v = sum over pairs s != t (both
// not v) of the fraction of shortest s-t paths through v. Floyd-Warshall that
// also counts shortest paths.
let (n, m) = read<(1..=100, 1..=4500)>()
domain V = 1..=n
var best: [V, V]?(d: Int, ways: Int) = [V, V] (s, t) => (s == t).then((d: 0, ways: 1))
for (a, b, c) in read<[m](a: V, b: V, c: 1..=1000)>():
    best[a, b] = Some((d: c, ways: 1))
    best[b, a] = Some((d: c, ways: 1))
for v in V:
    for s in V if s != v:
        for t in V if t != v and t != s:
            let Some(sv) = best[s, v] else: continue
            let Some(vt) = best[v, t] else: continue
            let via = (d: sv.d + vt.d, ways: sv.ways * vt.ways)
            best[s, t] = match best[s, t]:
                Some(st) if st.d < via.d => Some(st)
                Some(st) if st.d == via.d => Some((d: st.d, ways: st.ways + via.ways))
                _ => Some(via)
fn share(s: V, v: V, t: V) -> F64:
    let Some(sv) = best[s, v] else: return 0.0
    let Some(vt) = best[v, t] else: return 0.0
    let st = best[s, t]!
    if sv.d + vt.d == st.d: F64(sv.ways * vt.ways) / F64(st.ways) else: 0.0
for v in V:
    print(
        [
            (s, t) for s in V for t in V if s != t and s != v and t != v
        ].sum(of: p => share(p.0, v, p.1)),
        digits: 3,
    )
