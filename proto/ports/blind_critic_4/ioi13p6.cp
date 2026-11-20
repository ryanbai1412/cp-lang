// IOI '13 P6 — Game (runnable proto/cpc port of ioi13p6_draft.cp)
// Prototype deltas vs v2.1 draft: no `@grader struct`, no struct field defaults, no `.push_id`,
// explicit parallel-array arenas (outer segment tree over rows, dynamic inner segment tree over cols).

let R: 1..=1_000_000_000 = read()
let C: 1..=1_000_000_000 = read()
let N: 1..=250_000 = read()

// Inner dynamic segment tree over [0..C-1] storing compressed leaves (`key`, `val`) or internal gcd.
var in_lc: List<Int> = [-1]
var in_rc: List<Int> = [-1]
var in_key: List<Int> = [-1]
var in_g: List<Int> = [0]

fn new_inner(key: Int, val: Int) -> Int:
    let id = in_g.len
    in_lc.push(-1)
    in_rc.push(-1)
    in_key.push(key)
    in_g.push(val)
    return id

fn inner_update(o: Int, l: Int, r: Int, pos: Int, val: Int) -> Int:
    if o == -1:
        return new_inner(pos, val)
    if l == r:
        in_g[o] = val
        return o
    let k = in_key[o]
    if k != -1:
        if k == pos:
            in_g[o] = val
            return o
        let old_v = in_g[o]
        in_key[o] = -1
        let mid = (l + r) / 2
        if k <= mid:
            let cur_lc = in_lc[o]
            let ch = inner_update(cur_lc, l, mid, k, old_v)
            in_lc[o] = ch
        else:
            let cur_rc = in_rc[o]
            let ch = inner_update(cur_rc, mid + 1, r, k, old_v)
            in_rc[o] = ch
    let mid = (l + r) / 2
    if pos <= mid:
        let cur_lc = in_lc[o]
        let ch = inner_update(cur_lc, l, mid, pos, val)
        in_lc[o] = ch
    else:
        let cur_rc = in_rc[o]
        let ch = inner_update(cur_rc, mid + 1, r, pos, val)
        in_rc[o] = ch
    let lg = if in_lc[o] != -1: in_g[in_lc[o]] else: 0
    let rg = if in_rc[o] != -1: in_g[in_rc[o]] else: 0
    in_g[o] = gcd(lg, rg)
    return o

fn inner_query(o: Int, l: Int, r: Int, ql: Int, qr: Int) -> Int:
    if o == -1 or qr < l or r < ql:
        return 0
    if ql <= l and r <= qr:
        return in_g[o]
    let k = in_key[o]
    if k != -1:
        return if ql <= k and k <= qr: in_g[o] else: 0
    let mid = (l + r) / 2
    let a = inner_query(in_lc[o], l, mid, ql, qr)
    let b = inner_query(in_rc[o], mid + 1, r, ql, qr)
    return gcd(a, b)

// Outer dynamic segment tree over [0..R-1].
var out_lc: List<Int> = [-1]
var out_rc: List<Int> = [-1]
var out_root: List<Int> = [-1]

fn new_outer() -> Int:
    let id = out_root.len
    out_lc.push(-1)
    out_rc.push(-1)
    out_root.push(-1)
    return id

var root = new_outer()

fn outer_update(o: Int, l: Int, r: Int, row: Int, col: Int, val: Int):
    if l == r:
        let old_r = out_root[o]
        let nr = inner_update(old_r, 0, C - 1, col, val)
        out_root[o] = nr
        return
    let mid = (l + r) / 2
    if row <= mid:
        if out_lc[o] == -1:
            let nid = new_outer()
            out_lc[o] = nid
        let lc_id = out_lc[o]
        outer_update(lc_id, l, mid, row, col, val)
    else:
        if out_rc[o] == -1:
            let nid = new_outer()
            out_rc[o] = nid
        let rc_id = out_rc[o]
        outer_update(rc_id, mid + 1, r, row, col, val)
    let lc_id = out_lc[o]
    let rc_id = out_rc[o]
    let lg = if lc_id != -1: inner_query(out_root[lc_id], 0, C - 1, col, col) else: 0
    let rg = if rc_id != -1: inner_query(out_root[rc_id], 0, C - 1, col, col) else: 0
    let old_r = out_root[o]
    let nr = inner_update(old_r, 0, C - 1, col, gcd(lg, rg))
    out_root[o] = nr

fn outer_query(o: Int, l: Int, r: Int, r1: Int, r2: Int, c1: Int, c2: Int) -> Int:
    if o == -1 or r2 < l or r < r1:
        return 0
    if r1 <= l and r <= r2:
        return inner_query(out_root[o], 0, C - 1, c1, c2)
    let mid = (l + r) / 2
    let a = outer_query(out_lc[o], l, mid, r1, r2, c1, c2)
    let b = outer_query(out_rc[o], mid + 1, r, r1, r2, c1, c2)
    return gcd(a, b)

for _ in 0..N:
    let op: Int = read()
    if op == 1:
        let p: Int = read()
        let q: Int = read()
        let k: Int = read()
        outer_update(root, 0, R - 1, p, q, k)
    else:
        let p: Int = read()
        let q: Int = read()
        let u: Int = read()
        let v: Int = read()
        print(outer_query(root, 0, R - 1, p, u, q, v))
