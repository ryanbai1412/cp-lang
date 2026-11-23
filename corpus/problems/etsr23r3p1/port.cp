// Runnable port (proto/cp.py lib-run) of etsr23r3p1.cp
// original: corpus/submissions/5527625.cpp

struct Cell:
    lo_orig: Int
    lo_val: Int
    hi_orig: Int
    hi_val: Int

struct Op:
    has_set: Bool
    set_val: Int
    mul: Int
    m: Int
    b: Int

fn eval_op(op: Op, orig: Int, val: Int) -> Int:
    let base = if op.has_set: op.set_val else: val
    base * op.mul + op.m * orig + op.b

fn compose_op(newer: Op, older: Op) -> Op:
    if newer.has_set: newer
    else: Op(
        older.has_set,
        older.set_val,
        older.mul * newer.mul,
        older.m * newer.mul + newer.m,
        older.b * newer.mul + newer.b,
    )

fn apply_op(op: Op, c: ?Cell, _len: Int) -> ?Cell:
    if c == None: return None
    let x = c!
    Cell(x.lo_orig, eval_op(op, x.lo_orig, x.lo_val), x.hi_orig, eval_op(op, x.hi_orig, x.hi_val))

fn merge_cell(a: ?Cell, b: ?Cell) -> ?Cell:
    if a == None: return b
    if b == None: return a
    Cell(a!.lo_orig, a!.lo_val, b!.hi_orig, b!.hi_val)

let (n, ql, qr): (Int, Int, Int) = read()
var op_t: List<Int> = []
var op_x: List<Int> = []
for _ in 0..n:
    let (t, x): (Int, Int) = read()
    op_t.push(t)
    op_x.push(x)

let q: Int = read()
var arr: List<Int> = []
for _ in 0..q:
    let a: Int = read()
    arr.push(a)

let sraw = arr.sorted()
var uniq: List<Int> = []
for v in sraw:
    if uniq.len == 0 or uniq.last! != v:
        uniq.push(v)

let ulen = uniq.len
domain U = 0..ulen
let m_cell = Monoid<?Cell>(None, merge_cell)
let act_cell = Action<?Cell, Op>(Op(false, 0, 1, 0, 0), compose_op, apply_op, over: m_cell)
var st = SegTree(
    act_cell.over,
    [U] u => Some(Cell(uniq[u.ord], uniq[u.ord], uniq[u.ord], uniq[u.ord])),
    act: act_cell,
)

for k in 0..n:
    let t = op_t[k]
    let x = op_x[k]
    var tag = Op(false, 0, 1, x, 0)
    if t == 1: tag = Op(false, 0, 1, 0, x)
    elif t == 2: tag = Op(false, 0, 1, 0, -x)
    elif t == 3: tag = Op(false, 0, x, 0, 0)
    st.apply(U.all, tag)

    let left_end = st.right_while(U.first!.l, c => if c == None: true else: c!.hi_val < ql)
    st.apply(U.first!.l..left_end, Op(true, ql, 1, 0, 0))

    let right_start = st.left_while(U.last!.r, c => if c == None: true else: c!.lo_val > qr)
    st.apply(right_start..U.last!.r, Op(true, qr, 1, 0, 0))

var idx_of = Map<Int, Int>()
for i in 0..ulen:
    idx_of[uniq[i]] = i

for a in arr:
    print(st.get(U(idx_of[a]))!.lo_val)
