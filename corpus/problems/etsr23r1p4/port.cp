// Runnable port (proto/cp.py lib-run) of etsr23r1p4.cp
// original: corpus/submissions/5495163.cpp

struct Node:
    mn: Int
    mx: Int
    sm: Int

struct Tag:
    has_set: Bool
    set_val: Int
    add: Int

fn merge_node(a: Node, b: Node) -> Node:
    Node(min(a.mn, b.mn), max(a.mx, b.mx), a.sm + b.sm)

fn compose_tag(old: Tag, newer: Tag) -> Tag:
    if newer.has_set: newer
    else: Tag(old.has_set, old.set_val, old.add + newer.add)

fn apply_tag(tag: Tag, node: Node, len: Int) -> Node:
    if tag.has_set:
        let v = tag.set_val + tag.add
        Node(v, v, len * v)
    else:
        Node(node.mn + tag.add, node.mx + tag.add, node.sm + len * tag.add)

let (n, q): (Int, Int) = read()
domain I = 1..=n
let init: [I]Int = read()

var t: List<Node> = List.filled(4 * n + 4, Node(0, 0, 0))
var lz: List<Tag> = List.filled(4 * n + 4, Tag(false, 0, 0))

fn build(o: Int, l: Int, r: Int):
    if l == r:
        let v = init[I(l + 1)]
        t[o] = Node(v, v, v)
        return
    let mid = (l + r) / 2
    build(2 * o, l, mid)
    build(2 * o + 1, mid + 1, r)
    t[o] = merge_node(t[2 * o], t[2 * o + 1])

fn put(o: Int, l: Int, r: Int, tag: Tag):
    t[o] = apply_tag(tag, t[o], r - l + 1)
    lz[o] = compose_tag(lz[o], tag)

fn push(o: Int, l: Int, r: Int):
    let mid = (l + r) / 2
    put(2 * o, l, mid, lz[o])
    put(2 * o + 1, mid + 1, r, lz[o])
    lz[o] = Tag(false, 0, 0)

fn range_add(o: Int, l: Int, r: Int, ql: Int, qr: Int, x: Int):
    if ql <= l and r <= qr:
        put(o, l, r, Tag(false, 0, x))
        return
    push(o, l, r)
    let mid = (l + r) / 2
    if ql <= mid: range_add(2 * o, l, mid, ql, qr, x)
    if mid < qr: range_add(2 * o + 1, mid + 1, r, ql, qr, x)
    t[o] = merge_node(t[2 * o], t[2 * o + 1])

fn range_div(o: Int, l: Int, r: Int, ql: Int, qr: Int, x: Int):
    if ql <= l and r <= qr:
        let cmn = t[o].mn / x
        let cmx = t[o].mx / x
        if cmn == cmx:
            put(o, l, r, Tag(true, cmn, 0))
            return
        elif t[o].mx - t[o].mn == 1:
            put(o, l, r, Tag(false, 0, cmx - t[o].mx))
            return
    push(o, l, r)
    let mid = (l + r) / 2
    if ql <= mid: range_div(2 * o, l, mid, ql, qr, x)
    if mid < qr: range_div(2 * o + 1, mid + 1, r, ql, qr, x)
    t[o] = merge_node(t[2 * o], t[2 * o + 1])

fn query(o: Int, l: Int, r: Int, ql: Int, qr: Int) -> Node:
    if ql <= l and r <= qr: return t[o]
    push(o, l, r)
    let mid = (l + r) / 2
    if qr <= mid: return query(2 * o, l, mid, ql, qr)
    elif ql > mid: return query(2 * o + 1, mid + 1, r, ql, qr)
    else: return merge_node(query(2 * o, l, mid, ql, qr), query(2 * o + 1, mid + 1, r, ql, qr))

build(1, 0, n - 1)
for _ in 0..q:
    let (ty, l, r): (Int, Int, Int) = read()
    if ty == 1:
        let x: Int = read()
        range_add(1, 0, n - 1, l - 1, r - 1, x)
    elif ty == 2:
        let x: Int = read()
        range_div(1, 0, n - 1, l - 1, r - 1, x)
    elif ty == 3:
        print(query(1, 0, n - 1, l - 1, r - 1).mn)
    else:
        print(query(1, 0, n - 1, l - 1, r - 1).sm)
