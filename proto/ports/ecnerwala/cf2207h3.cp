// Codeforces 2207 H3: Bowser's Castle (Hard Version).
@!interactive
use std.seq.{reverse, rotate_left, rotate_right}

struct Node:
    l: Int
    r: Int
    is_max: Bool
    ch: ?(Int, Int)

@!multitest
let n: Int = read()
var x = [i + 1 for i in 0..n]
var ops = [Node(i, i + 1, false, none) for i in 0..n]

fn query() -> Int:
    print("?", x)
    let resp: Int = read()
    x.find(resp)!

fn merge(a: Int, b: Int, is_max: Bool) -> Int:
    let id = ops.len
    ops.push(Node(ops[a].l, ops[b].r, is_max, (a, b)))
    id

fn solve(l: Int, r: Int, is_max: Bool, known_m: ?Int) -> List<Int>:
    if r - l == 0: return List<Int>()
    if r - l == 1: return [l]
    let m = known_m ?? query()

    rotate_left(mut x, 1, range: l..=m)
    var l_ch = solve(l, m, is_max, none)
    rotate_right(mut x, 1, range: l..=m)

    var mid = m

    reverse(mut x, range: m..r)
    var r_ch = solve(m + 1, r, not is_max, none)
    reverse(mut x, range: m..r)

    reverse(mut r_ch)
    while let li = l_ch.last, let ri = r_ch.last:
        let (ll, rr) = (ops[li].l, ops[ri].r)
        reverse(mut x, range: ll..rr)
        let loc = query()
        reverse(mut x, range: ll..rr)
        if loc < ops[mid].l:
            mid = merge(mid, r_ch.pop()!, not is_max)
        else:
            mid = merge(l_ch.pop()!, mid, is_max)
    while let rn = r_ch.pop(): mid = merge(mid, rn, not is_max)
    l_ch.push(mid)
    l_ch

let loc_inc = query()
reverse(mut x)
let loc_dec = query()
reverse(mut x)

let is_max = loc_inc >= loc_dec
if not is_max: reverse(mut x)
var top = solve(0, n, is_max, if is_max: loc_inc else: loc_dec)
while top.len >= 2:
    let b = top.pop()!
    let a = top.pop()!
    top.push(merge(a, b, is_max))
let root = top[0]

print("!")
loop:
    let first: Int = read()
    if first == 0: break
    var vals: [0..n]Int = 0
    vals[0] = first
    for i in 1..n: vals[i] = read()

    fn eval(cur: Int) -> Int:
        let node = ops[cur]
        if let (a, b) = node.ch:
            let (va, vb) = (eval(a), eval(b))
            if node.is_max: max(va, vb) else: min(va, vb)
        else:
            vals[cur]

    print(eval(root))
