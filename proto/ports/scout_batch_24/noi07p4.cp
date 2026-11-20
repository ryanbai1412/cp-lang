// NOI '07 P4 - Necklace Factory (DMOJ: noi07p4, 20pt, TL 1.0s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Cyclic necklace of n <= 5e5 beads with colors in 1..=c_max.
// q <= 5e5 operations:
//   R k      : rotate clockwise by k
//   F        : flip across axis through position 1
//   S i j    : swap beads at logical positions i and j
//   P i j x  : paint clockwise arc i -> j (inclusive) color x
//   C        : total color segments around whole necklace
//   CS i j   : color segments along clockwise arc i -> j (inclusive)
//
// Track global dihedral transform as physical(x) = (dir * x + off) % n (0-based),
// backed by a LazySegTree over a non-commutative color-run Monoid with range-assign.
use std.algebra.{Semigroup, Monoid, Action, opt}
use std.lazy.{LazySegTree}

let (n, c_max): (1..=5e5, 1..=5e5) = read()
domain Bead = 0..n
let init_col: [Bead]Int = read()
let q: 1..=5e5 = read()

// WISH A4: Heterogeneous command tokens (`R`, `F`, `S`, `P`, `C`, `CS`) with
// different arities and 1-based positions require per-variant `@tag` decoding
// and manual 1-to-0 conversion; `CS` vs `C` prefix overlap must match full tokens.
enum Cmd:
    @tag("R") Rot(k: 0..n)
    @tag("F") Flip
    @tag("S") Swap(i: 1..=n, j: 1..=n)
    @tag("P") Paint(i: 1..=n, j: 1..=n, x: 1..=c_max)
    @tag("C") CountAll
    @tag("CS") CountSub(i: 1..=n, j: 1..=n)

let cmds: [0..q]Cmd = read()

struct Run(lcol: Int, rcol: Int, runs: Int)

// WISH A5: `opt(Semigroup)` wraps `Run` in `?Run` cleanly for the identity, but
// writing `Action<?Run, ?Int>` still requires manual `none` unwrapping in both
// `compose` and `apply` (5 lines of boilerplate for a trivial range-assign tag).
const RunMonoid = opt(Semigroup<Run>(
    (a, b) => Run(a.lcol, b.rcol, a.runs + b.runs - if a.rcol == b.lcol: 1 else: 0)
))

const PaintAction = Action<?Run, ?Int>(
    id: none,
    compose: (f, g) => if f == none: g else: f,
    apply: (f, x) => match (f, x):
        (none, _) => x
        (_, none) => none
        (c, _) => Run(c, c, 1)
)

fn solve() -> List<Int>:
    var st = LazySegTree<Bead, ?Run, ?Int>.from(
        RunMonoid,
        PaintAction,
        [Bead] i => Run(init_col[i], init_col[i], 1)
    )
    var dir = 1
    var off = 0

    fn phys(pos1: Int) -> Bead:
        Bead((dir * (pos1 - 1) + off) % n)

    // Physical arc endpoints `(a, b)` for clockwise logical arc `i -> j`:
    // if `dir == -1`, reversing the arc leaves run count invariant, so walk `phys(j) -> phys(i)`.
    fn arc(i: Int, j: Int) -> (Bead, Bead):
        if dir == 1: (phys(i), phys(j)) else: (phys(j), phys(i))

    var out: List<Int> = []
    for cmd in cmds:
        match cmd:
            Rot(k) =>
                off = (off - dir * k) % n
            Flip =>
                dir = -dir
            Swap(i, j) =>
                let (pi, pj) = (phys(i), phys(j))
                let ci = st.get(pi)!.lcol
                let cj = st.get(pj)!.lcol
                st.set(pi, Run(cj, cj, 1))
                st.set(pj, Run(ci, ci, 1))
            Paint(i, j, x) =>
                let (a, b) = arc(i, j)
                if a <= b:
                    st.apply(a..=b, x)
                else:
                    st.apply(a..=Bead.last, x)
                    st.apply(Bead.first..=b, x)
            CountAll =>
                let all = st.fold(Bead.first..=Bead.last)!
                let ans = if all.runs > 1 and all.lcol == all.rcol: all.runs - 1 else: all.runs
                out.push(ans)
            CountSub(i, j) =>
                let (a, b) = arc(i, j)
                let seg = if a <= b:
                    st.fold(a..=b)!
                else:
                    RunMonoid.op(st.fold(a..=Bead.last), st.fold(Bead.first..=b))!
                out.push(seg.runs)
    out

for ans in solve(): print(ans)
