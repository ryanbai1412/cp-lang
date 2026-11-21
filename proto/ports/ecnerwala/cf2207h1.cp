@!interactive

struct Op:
    l: Int
    r: Int
    is_max: Bool
    c0: Int
    c1: Int

fn ask(x: List<Int>) -> Int:
    print("?", [v + 1 for v in x])
    let resp = read::<Int>() - 1
    first(i for i in 0..x.len if x[i] == resp)!

fn rev_sub(mut x: List<Int>, l: Int, r: Int):
    var (i, j) = (l, r - 1)
    while i < j:
        (x[i], x[j]) = (x[j], x[i])
        i += 1
        j -= 1

fn rot_left1(mut x: List<Int>, l: Int, r: Int):
    if r - l <= 1: return
    let first_v = x[l]
    for i in l..r - 1: x[i] = x[i + 1]
    x[r - 1] = first_v

fn rot_right1(mut x: List<Int>, l: Int, r: Int):
    if r - l <= 1: return
    let last_v = x[r - 1]
    for i in (l + 1..r).rev: x[i] = x[i - 1]
    x[l] = last_v

fn merge_op(mut ops: List<Op>, a: Int, b: Int, is_max: Bool) -> Int:
    let id = ops.len
    ops.push(Op(ops[a].l, ops[b].r, is_max, a, b))
    id

fn solve(
    mut x: List<Int>,
    mut ops: List<Op>,
    l: Int,
    r: Int,
    is_max: Bool,
    known_m: ?Int,
) -> List<Int>:
    if r - l == 0: return List<Int>()
    if r - l == 1:
        ops[l] = Op(l, r, false, -1, -1)
        return [l]
    let m = known_m ?? ask(x)
    rot_left1(mut x, l, m + 1)
    var l_ch = solve(mut x, mut ops, l, m, is_max, none)
    rot_right1(mut x, l, m + 1)
    var mid_ch = solve(mut x, mut ops, m, m + 1, is_max, none)[0]
    rev_sub(mut x, m, r)
    var r_ch = solve(mut x, mut ops, m + 1, r, not is_max, none)
    rev_sub(mut x, m, r)
    r_ch.reverse()
    while not l_ch.is_empty and not r_ch.is_empty:
        let (lb, rb) = (ops[l_ch.last!].l, ops[r_ch.last!].r)
        rev_sub(mut x, lb, rb)
        let loc = ask(x)
        rev_sub(mut x, lb, rb)
        if loc < ops[mid_ch].l:
            mid_ch = merge_op(mut ops, mid_ch, r_ch.pop()!, not is_max)
        else:
            mid_ch = merge_op(mut ops, l_ch.pop()!, mid_ch, is_max)
    while not r_ch.is_empty:
        mid_ch = merge_op(mut ops, mid_ch, r_ch.pop()!, not is_max)
    l_ch.push(mid_ch)
    l_ch

fn eval_tree(ops: List<Op>, vals: List<Int>, n: Int, cur: Int) -> Int:
    if cur < n: return vals[cur]
    let a = eval_tree(ops, vals, n, ops[cur].c0)
    let b = eval_tree(ops, vals, n, ops[cur].c1)
    if ops[cur].is_max: max(a, b) else: min(a, b)

@!multitest
let n: Int = read()
var x = [i for i in 0..n]
var ops = [Op(0, 0, false, -1, -1) for _ in 0..n]
let loc_incr = ask(x)
x.reverse()
let loc_decr = ask(x)
x.reverse()
let is_max = loc_incr >= loc_decr
if not is_max: x.reverse()
var top = solve(mut x, mut ops, 0, n, is_max, some(if is_max: loc_incr else: loc_decr))
while top.len >= 2:
    let b = top.pop()!
    let a = top.pop()!
    top.push(merge_op(mut ops, a, b, is_max))
let root = top[0]
print("!")
while true:
    let first_v: Int = read()
    if first_v == 0: break
    var vals = [first_v]
    for _ in 1..n: vals.push(read::<Int>())
    print(eval_tree(ops, vals, n, root))
