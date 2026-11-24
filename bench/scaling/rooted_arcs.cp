// let: k = n; depth = n; children = n; size = n; path = n
let n: Int = read()
domain V = 0..n
domain E = 0..n - 1
let es: [E](u: V, v: V) = [E] e => (u: V(0), v: V(e.ord + 1))
let st = Rooted(Graph.undirected(es), root: V(0))
let ex_f: V -> Int = c => c.ord // within: tree.Rooted.excl_fold
var s = 0
for _ in 0..40:
    for c in st.excl_fold(V(0), +, ex_f): s += c.rest // timed: tree.Rooted.excl_fold
    for c in st.light(V(0)): s += c.ord // timed: tree.Rooted.light
for _ in 0..16:
    for c in st.children(V(0)): // timed: tree.Rooted.children
        s += c.ord // timed: tree.Rooted.children
    for a in st.child_arcs(V(0)): // timed: tree.Rooted.child_arcs
        s += a.to.ord // timed: tree.Rooted.child_arcs
print(s)
