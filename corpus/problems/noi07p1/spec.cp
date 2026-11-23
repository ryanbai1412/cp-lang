// NOI 2007 "Social Network": importance of v = sum over pairs s != t (both
// not v) of the fraction of shortest s-t paths through v. Floyd-Warshall that
// also counts shortest paths, multiplying ways only on a tie or a win.
let (n, m) = read<(1..=100, 1..=4500)>()
domain V = 1..=n
var best: [V, V]?(d: Int, ways: Int) = [V, V] (s, t) => if s == t: Some((d: 0, ways: 1)) else: None
for (a, b, c) in read<[m](a: V, b: V, c: 1..=1000)>():
    for (x, y) in [(a, b), (b, a)]:
        best[x, y] = Some((d: c, ways: 1))
for v in V:
    for s in V if s != v:
        let Some(sv) = best[s, v] else: continue
        for t in V if t != v and t != s:
            let Some(vt) = best[v, t] else: continue
            let d = sv.d + vt.d
            best[s, t] = match best[s, t]:
                Some(st) if st.d < d => Some(st)
                Some(st) if st.d == d => Some((d, ways: st.ways + sv.ways * vt.ways))
                _ => Some((d, ways: sv.ways * vt.ways))
fn share(s: V, v: V, t: V) -> F64:
    let (sv, vt, st) = (best[s, v]!, best[v, t]!, best[s, t]!)
    if sv.d + vt.d == st.d: F64(sv.ways * vt.ways) / F64(st.ways) else: 0.0
for v in V:
    print(
        [(s, t) for s in V for t in V if s != t and s != v and t != v].sum(of: p => share(p.0, v, p.1)),
        digits: 3,
    )
