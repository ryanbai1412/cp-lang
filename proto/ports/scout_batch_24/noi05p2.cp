// NOI '05 P2 - Maintaining a Sequence (DMOJ: noi05p2, 30pt, TL 0.6s, ML 64MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Dynamic sequence supporting up to 5e5 live elements (up to 4e6 total inserted)
// under strict 64 MB memory limit:
//   INSERT posi tot c_1..c_tot
//   DELETE posi tot
//   MAKE-SAME posi tot c
//   REVERSE posi tot
//   GET-SUM posi tot
//   MAX-SUM
//
// Implicit Treap over `Pool<Node>` + `Node.Id` with free-list recycling,
// non-commutative Kadane max-subarray-sum aggregate, and dual lazy tags (rev + same).
let (n, m): (1..=5e5, 1..=2e4) = read()
let init_a: [0..n]Int = read()

// WISH A12: Variable-length payload inside an enum variant (`INSERT posi tot c_1..c_tot`)
// cannot be described by a declarative `read()` type or shrunk by `cpl stress`;
// even worse, `posi` and `tot` must satisfy `1 <= posi <= cur_len` dynamically!
enum Cmd:
    @tag("INSERT") Insert(posi: Int, vals: List<Int>)
    @tag("DELETE") Delete(posi: Int, tot: Int)
    @tag("MAKE-SAME") MakeSame(posi: Int, tot: Int, c: -1000..=1000)
    @tag("REVERSE") Reverse(posi: Int, tot: Int)
    @tag("GET-SUM") GetSum(posi: Int, tot: Int)
    @tag("MAX-SUM") MaxSum

// WISH A13: `Pool<Node>` with free-list recycling is mandatory at 64 MB when
// 4e6 nodes are allocated/freed with <= 5e5 live. `Pool<Node>` has no built-in
// `.free(id)` / generational slot reuse, so every dynamic BST hand-rolls a free stack.
// Also, storing `Int` (64-bit) instead of `I32` inside `Node` doubles struct size
// and blows the 64 MB limit (12 fields * 8B = 96B -> 48MB vs 44B -> 22MB)!
struct Node:
    val: I32
    sz: I32
    sum: I32
    pref: I32
    suff: I32
    best: I32
    same: ?I32 = none
    rev: Bool = false
    prio: U32 = 0u32
    l: ?Node.Id = none
    r: ?Node.Id = none

fn solve(cmds: List<Cmd>) -> List<Int>:
    var pool = Pool<Node>.with_capacity(500_005)
    var free_ids: List<Node.Id> = []
    var rng: U32 = 2463534242u32

    mut fn next_prio() -> U32:
        rng ^= rng << 13
        rng ^= rng >> 17
        rng ^= rng << 5
        rng

    mut fn alloc_node(v: I32) -> Node.Id:
        let p = max(0i32, v)
        let nd = Node(val: v, sz: 1i32, sum: v, pref: p, suff: p, best: v, prio: next_prio())
        if not free_ids.is_empty:
            let id = free_ids.pop()!
            pool[id] = nd
            id
        else:
            pool.alloc(nd)

    mut fn recycle(id: ?Node.Id):
        if let u = id:
            recycle(pool[u].l)
            recycle(pool[u].r)
            free_ids.push(u)

    fn sz_of(id: ?Node.Id) -> I32 = if let u = id: pool[u].sz else: 0i32

    mut fn apply_rev(id: ?Node.Id):
        if let u = id:
            pool[u].rev = not pool[u].rev
            swap(mut pool[u].l, mut pool[u].r)
            swap(mut pool[u].pref, mut pool[u].suff)

    mut fn apply_same(id: ?Node.Id, c: I32):
        if let u = id:
            let s = c * pool[u].sz
            pool[u].val = c
            pool[u].sum = s
            pool[u].same = c
            pool[u].rev = false
            pool[u].pref = max(0i32, s)
            pool[u].suff = max(0i32, s)
            pool[u].best = if c >= 0i32: s else: c

    mut fn push(u: Node.Id):
        if let c = pool[u].same:
            apply_same(pool[u].l, c)
            apply_same(pool[u].r, c)
            pool[u].same = none
            pool[u].rev = false
        elif pool[u].rev:
            apply_rev(pool[u].l)
            apply_rev(pool[u].r)
            pool[u].rev = false

    mut fn pull(u: Node.Id):
        let v = pool[u].val
        let (lsz, lsum, lpre, lsuf, lbest) = if let lc = pool[u].l:
            (pool[lc].sz, pool[lc].sum, pool[lc].pref, pool[lc].suff, pool[lc].best)
        else:
            (0i32, 0i32, 0i32, 0i32, I32.MIN)
        let (rsz, rsum, rpre, rsuf, rbest) = if let rc = pool[u].r:
            (pool[rc].sz, pool[rc].sum, pool[rc].pref, pool[rc].suff, pool[rc].best)
        else:
            (0i32, 0i32, 0i32, 0i32, I32.MIN)

        pool[u].sz = lsz + 1i32 + rsz
        pool[u].sum = lsum + v + rsum
        pool[u].pref = max(lpre, lsum + v + rpre)
        pool[u].suff = max(rsuf, lsuf + v + rsum)
        pool[u].best = max(max(lbest, rbest), lsuf + v + rpre)

    mut fn split(id: ?Node.Id, k: I32) -> (?Node.Id, ?Node.Id):
        if id == none: return (none, none)
        let u = id!
        push(u)
        let lsz = sz_of(pool[u].l)
        if k <= lsz:
            let (a, b) = split(pool[u].l, k)
            pool[u].l = b
            pull(u)
            (a, u)
        else:
            let (a, b) = split(pool[u].r, k - lsz - 1i32)
            pool[u].r = a
            pull(u)
            (u, b)

    mut fn merge(a: ?Node.Id, b: ?Node.Id) -> ?Node.Id:
        if a == none: return b
        if b == none: return a
        let (au, bu) = (a!, b!)
        if pool[au].prio > pool[bu].prio:
            push(au)
            pool[au].r = merge(pool[au].r, bu)
            pull(au)
            au
        else:
            push(bu)
            pool[bu].l = merge(au, pool[bu].l)
            pull(bu)
            bu

    mut fn build_cartesian(vals: List<I32>) -> ?Node.Id:
        var stk: List<Node.Id> = []
        for v in vals:
            let cur = alloc_node(v)
            var last: ?Node.Id = none
            while not stk.is_empty and pool[stk.last!].prio < pool[cur].prio:
                let top = stk.pop()!
                pull(top)
                last = top
            pool[cur].l = last
            if not stk.is_empty:
                pool[stk.last!].r = cur
            stk.push(cur)
        while stk.len > 1:
            pull(stk.pop()!)
        if stk.is_empty: return none
        pull(stk[0])
        stk[0]

    var root = build_cartesian([I32(init_a[i]) for i in 0..n])
    var out: List<Int> = []

    for cmd in cmds:
        match cmd:
            Insert(posi, vals) =>
                let (a, b) = split(root, I32(posi))
                let mid = build_cartesian([I32(x) for x in vals])
                root = merge(merge(a, mid), b)
            Delete(posi, tot) =>
                let (a, bc) = split(root, I32(posi - 1))
                let (b, c) = split(bc, I32(tot))
                recycle(b)
                root = merge(a, c)
            MakeSame(posi, tot, c) =>
                let (a, bc) = split(root, I32(posi - 1))
                let (b, d) = split(bc, I32(tot))
                apply_same(b, I32(c))
                root = merge(merge(a, b), d)
            Reverse(posi, tot) =>
                let (a, bc) = split(root, I32(posi - 1))
                let (b, c) = split(bc, I32(tot))
                apply_rev(b)
                root = merge(merge(a, b), c)
            GetSum(posi, tot) =>
                let (a, bc) = split(root, I32(posi - 1))
                let (b, c) = split(bc, I32(tot))
                out.push(if let u = b: Int(pool[u].sum) else: 0)
                root = merge(merge(a, b), c)
            MaxSum =>
                out.push(Int(pool[root!].best))
    out

var cmds: List<Cmd> = []
for _ in 0..m:
    let op: Str = read()
    if op == "INSERT":
        let (posi, tot): (Int, Int) = read()
        let vals: List<Int> = [read() for _ in 0..tot]
        cmds.push(Cmd.Insert(posi, vals))
    elif op == "DELETE":
        let (posi, tot): (Int, Int) = read()
        cmds.push(Cmd.Delete(posi, tot))
    elif op == "MAKE-SAME":
        let (posi, tot, c): (Int, Int, Int) = read()
        cmds.push(Cmd.MakeSame(posi, tot, c))
    elif op == "REVERSE":
        let (posi, tot): (Int, Int) = read()
        cmds.push(Cmd.Reverse(posi, tot))
    elif op == "GET-SUM":
        let (posi, tot): (Int, Int) = read()
        cmds.push(Cmd.GetSum(posi, tot))
    else:
        cmds.push(Cmd.MaxSum)

for ans in solve(cmds): print(ans)
