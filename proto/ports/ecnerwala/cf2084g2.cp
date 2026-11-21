@!multitest
use std.rand.Rng

struct SlopeNode(l: ?Int, r: ?Int, pri: U64, sz: Int, slope: Int, add: Int, step: Int)

let n: 1..=500_000 = read()
domain Pos = 1..=n
let a: [Pos]?(1..=n) = read(none: 0)
let half = n / 2

var fixed_even: [1..=n]?Bool = none
for i in Pos:
    if let v = a[i]:
        fixed_even[v] = i.label % 2 == 0

var rng = Rng(48)
var nodes: List<SlopeNode> = []
var root: ?Int = none

fn size_of(u: ?Int) -> Int = if let i = u: nodes[i].sz else: 0

fn apply_node(u: ?Int, add: Int, step: Int):
    if let i = u:
        nodes[i].slope += add + size_of(nodes[i].l) * step
        nodes[i].add += add
        nodes[i].step += step

fn push(i: Int):
    let add = nodes[i].add
    let step = nodes[i].step
    if add != 0 or step != 0:
        apply_node(nodes[i].l, add, step)
        apply_node(nodes[i].r, add + (1 + size_of(nodes[i].l)) * step, step)
        nodes[i].add = 0
        nodes[i].step = 0

fn pull(i: Int):
    nodes[i].sz = 1 + size_of(nodes[i].l) + size_of(nodes[i].r)

// Split into (>= val) and (< val) in descending slope order.
fn split_desc(u: ?Int, val: Int) -> (?Int, ?Int):
    let i = u ?? return (none, none)
    push(i)
    if val >= nodes[i].slope:
        let (l, r) = split_desc(nodes[i].l, val)
        nodes[i].l = r
        pull(i)
        (l, i)
    else:
        let (l, r) = split_desc(nodes[i].r, val)
        nodes[i].r = l
        pull(i)
        (i, r)

fn insert_node(u: ?Int, idx: Int, val: Int, pri: U64) -> Int:
    let i = u ?? return idx
    if pri < nodes[i].pri:
        let (l, r) = split_desc(i, val)
        nodes[idx].l = l
        nodes[idx].r = r
        pull(idx)
        return idx
    push(i)
    if val >= nodes[i].slope:
        nodes[i].l = insert_node(nodes[i].l, idx, val, pri)
    else:
        nodes[i].r = insert_node(nodes[i].r, idx, val, pri)
    pull(i)
    i

fn gain(k: Int, c: Int) -> Int = (2 * c - k) * (2 * (half - c) - (n - k))

var c0 = 0
var v0 = 0
for k in 0..n:
    let a0 = gain(k, c0)
    let a1 = gain(k, c0 + 1)
    v0 += a0
    apply_node(root, a1 - a0, -8)
    if let is_even = fixed_even[k + 1]:
        c0 += Int(is_even)
    else:
        let idx = nodes.len
        let pri = rng.next()
        nodes.push(SlopeNode(none, none, pri, 1, 0, 0, 0))
        root = insert_node(root, idx, 0, pri)

var take = half - c0
var best = v0

fn collect(u: ?Int):
    if let i = u:
        if take <= 0: return
        push(i)
        collect(nodes[i].l)
        if take > 0:
            best += nodes[i].slope
            take -= 1
            collect(nodes[i].r)

collect(root)
print((best + n * (n + 1) * (n + 1) / 2) / 2)
