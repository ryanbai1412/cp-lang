// NOI '21 P5 (Code): maintain a dynamic string of 'W'/'E' characters under APPEND,
// range FLIP, and range REVERSE, outputting the continued-fraction numerator and
// denominator mod 998244353 after every operation.
// Implicit treap over a flat List<Node> arena with 2x2 Mod<998244353> matrices,
// v2.2 `?? return` early exits, and RHS-before-LHS arena assignments.
// original: corpus/submissions/3927923.cpp

const MOD = 998_244_353
type Z = Mod<MOD>
type Id = 0..=250_005

struct Mat:
    a00: Z; a01: Z; a10: Z; a11: Z

    fn mul(self, o: Mat) -> Mat:
        Mat(
            self.a00 * o.a00 + self.a01 * o.a10, self.a00 * o.a01 + self.a01 * o.a11,
            self.a10 * o.a00 + self.a11 * o.a10, self.a10 * o.a01 + self.a11 * o.a11,
        )

    fn flipped(self) -> Mat = Mat(self.a11, self.a10, self.a01, self.a00)
    fn reversed(self) -> Mat = Mat(self.a11, self.a01, self.a10, self.a00)

fn base_mat(is_e: Bool) -> Mat =
    if is_e: Mat(1, 1, 0, 1) else: Mat(1, 0, 1, 1)

struct Node:
    var is_e: Bool
    var sz: Int = 1
    pri: Int
    var flip: Bool = false
    var rev: Bool = false
    var data: Mat
    var lc: ?Id = none
    var rc: ?Id = none

var arena: List<Node> = []

fn make_node(is_e: Bool) -> Id:
    let id: Id = arena.len
    arena.push(Node(is_e: is_e, pri: rand(0, 2_000_000_000), data: base_mat(is_e)))
    id

fn size_of(t: ?Id) -> Int = if let u = t: arena[u].sz else: 0

fn pull(u: Id):
    var sz = 1
    var d = base_mat(arena[u].is_e)
    if let l = arena[u].lc:
        sz += arena[l].sz
        d = arena[l].data.mul(d)
    if let r = arena[u].rc:
        sz += arena[r].sz
        d = d.mul(arena[r].data)
    if arena[u].flip: d = d.flipped()
    if arena[u].rev: d = d.reversed()
    arena[u].sz = sz
    arena[u].data = d

fn push(u: Id):
    let (fl, rv) = (arena[u].flip, arena[u].rev)
    if fl: arena[u].is_e = not arena[u].is_e
    if rv: swap(mut arena[u].lc, mut arena[u].rc)
    for child in [arena[u].lc, arena[u].rc]:
        if let c = child:
            arena[c].flip = arena[c].flip != fl
            arena[c].rev = arena[c].rev != rv
            pull(c)
    arena[u].flip = false
    arena[u].rev = false

fn split(t: ?Id, k: Int) -> (?Id, ?Id):
    let u = t ?? return (none, none)
    push(u)
    let lsz = size_of(arena[u].lc)
    if k <= lsz:
        let (a, b) = split(arena[u].lc, k)
        arena[u].lc = b
        pull(u)
        (a, u)
    else:
        let (a, b) = split(arena[u].rc, k - lsz - 1)
        arena[u].rc = a
        pull(u)
        (u, b)

fn merge(a: ?Id, b: ?Id) -> ?Id:
    let u = a ?? return b
    let v = b ?? return a
    if arena[u].pri < arena[v].pri:
        push(u)
        arena[u].rc = merge(arena[u].rc, v)
        pull(u)
        u
    else:
        push(v)
        arena[v].lc = merge(u, arena[v].lc)
        pull(v)
        v

let (n, q): (Int, Int) = read()
let s: Str = read()
var root: ?Id = none
for ch in s: root = merge(root, make_node(ch == 'E'))

fn emit():
    let m = arena[root!].data
    print(m.a00 + m.a01, m.a10 + m.a11)

emit()
for _ in 0..q:
    let op: Str = read()
    if op == "APPEND":
        let ch: Char = read()
        root = merge(root, make_node(ch == 'E'))
    else:
        let (l, r): (Int, Int) = read()
        let (ab, c) = split(root, r)
        let (a, b) = split(ab, l - 1)
        let mid = b!
        if op == "FLIP": arena[mid].flip = not arena[mid].flip
        else: arena[mid].rev = not arena[mid].rev
        pull(mid)
        root = merge(merge(a, mid), c)
    emit()
