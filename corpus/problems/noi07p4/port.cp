// Runnable port (proto/cp.py lib-run) of statement-first noi07p4.cp
// original: corpus/submissions/5032746.cpp

let (n, c_max): (Int, Int) = read()
domain Pos = 0..n
let init_col: [Pos]Int = read()
let q: Int = read()

struct Run:
    empty: Bool
    lcol: Int
    rcol: Int
    runs: Int

let run_monoid = Monoid<Run>(
    id: Run(true, 0, 0, 0),
    op: (a, b) => if a.empty: b else: if b.empty: a else: Run(
        false,
        a.lcol,
        b.rcol,
        a.runs + b.runs - Int(a.rcol == b.lcol),
    ),
)

let paint_action = Action<Run, ?Int>(
    id: None,
    compose: (f, g) => if f == none: g else: f,
    apply: (f, x, len) => if f == None or x.empty: x else: Run(false, f!, f!, 1),
    over: run_monoid,
)

let init = [Pos] p => Run(false, init_col[p], init_col[p], 1)
var st = LazySegTree<Pos, Run, ?Int>(paint_action, init)

var dir = 1
var off = 0

fn phys(one_based: Int) -> Pos:
    let x = one_based - 1
    Pos((dir * x + off) % n)

for _ in 0..q:
    let op: Str = read()
    if op == "R":
        let k: Int = read()
        off = (off - dir * k) % n
    elif op == "F":
        dir = -dir
    elif op == "S":
        let (i, j): (Int, Int) = read()
        let (pi, pj) = (phys(i), phys(j))
        if pi != pj:
            let ci = st.get(pi).lcol
            let cj = st.get(pj).lcol
            st.set(pi, Run(false, cj, cj, 1))
            st.set(pj, Run(false, ci, ci, 1))
    elif op == "P":
        let (i, j, x): (Int, Int, Int) = read()
        var (l, r) = (phys(i), phys(j))
        if dir == -1: swap(mut l, mut r)
        if l.to_idx <= r.to_idx:
            st.apply(l..=r, x)
        else:
            st.apply(l.., x)
            st.apply(..=r, x)
    elif op == "C":
        let all = st.fold(Pos.all)
        let cyc = if all.runs > 1 and all.lcol == all.rcol: all.runs - 1 else: all.runs
        print(cyc)
    else:
        let (i, j): (Int, Int) = read()
        var (l, r) = (phys(i), phys(j))
        if dir == -1: swap(mut l, mut r)
        var res = run_monoid.id
        if l.to_idx <= r.to_idx:
            res = st.fold(l..=r)
        else:
            res = run_monoid.op(st.fold(l..), st.fold(..=r))
        print(res.runs)
