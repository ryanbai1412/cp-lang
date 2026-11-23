// Tuned: sorted_set.cp with its order-statistic tree written out as an AVL
// tree over one list of 24-byte nodes (key, size, children, height).
// `SortedSet` is a randomized treap (expected depth about 2 ln n, a 40-byte
// node with parent links and multiplicities for cursors and `SortedBag`);
// sorted_set.cpp's `__gnu_pbds::tree` is a red-black tree, whose depth AVL
// matches. Same operations, same output.
type Ix = -1..=200_000
struct Node:
    var key: Int
    var sz: 0..=200_000
    var l: Ix
    var r: Ix
    var h: 0..=64
var t: List<Node> = []
var root: Ix = -1
t.reserve(100000)

fn size(u: Ix) -> Int:
    if u < 0: return 0
    unchecked: return t[u].sz

fn height(u: Ix) -> Int:
    if u < 0: return 0
    unchecked: return t[u].h

fn pull(u: Ix):
    unchecked:
        let (a, b) = (t[u].l, t[u].r)
        t[u].sz = size(a) + size(b) + 1
        t[u].h = max(height(a), height(b)) + 1

fn rot_right(u: Ix) -> Ix:
    unchecked:
        let v = t[u].l
        t[u].l = t[v].r
        t[v].r = u
        pull(u)
        pull(v)
        return v

fn rot_left(u: Ix) -> Ix:
    unchecked:
        let v = t[u].r
        t[u].r = t[v].l
        t[v].l = u
        pull(u)
        pull(v)
        return v

fn balance(u: Ix) -> Ix:
    pull(u)
    unchecked:
        let d = height(t[u].l) - height(t[u].r)
        if d > 1:
            let l = t[u].l
            if height(t[l].l) < height(t[l].r): t[u].l = rot_left(l)
            return rot_right(u)
        if d < -1:
            let r = t[u].r
            if height(t[r].r) < height(t[r].l): t[u].r = rot_right(r)
            return rot_left(u)
    u

fn insert(u: Ix, x: Int) -> (Ix, Bool):
    if u < 0:
        t.push(Node(x, 1, -1, -1, 1))
        return (t.len - 1, true)
    unchecked:
        if x < t[u].key:
            let (n, added) = insert(t[u].l, x)
            if not added: return (u, false)
            t[u].l = n
            return (balance(u), true)
        if t[u].key < x:
            let (n, added) = insert(t[u].r, x)
            if not added: return (u, false)
            t[u].r = n
            return (balance(u), true)
    (u, false)

fn take_min(u: Ix) -> (Ix, Ix):
    unchecked:
        let l = t[u].l
        if l < 0: return (t[u].r, u)
        let (n, m) = take_min(l)
        t[u].l = n
        return (balance(u), m)

fn erase(u: Ix, x: Int) -> (Ix, Bool):
    if u < 0: return (u, false)
    unchecked:
        if x < t[u].key:
            let (n, gone) = erase(t[u].l, x)
            if not gone: return (u, false)
            t[u].l = n
            return (balance(u), true)
        if t[u].key < x:
            let (n, gone) = erase(t[u].r, x)
            if not gone: return (u, false)
            t[u].r = n
            return (balance(u), true)
        let (l, r) = (t[u].l, t[u].r)
        if r < 0: return (l, true)
        let (n, m) = take_min(r)
        t[m].l = l
        t[m].r = n
        return (balance(m), true)

fn rank(x: Int) -> Int:
    var ans = 0
    var u = root
    unchecked:
        while u >= 0:
            if t[u].key < x:
                ans += size(t[u].l) + 1
                u = t[u].r
            else:
                u = t[u].l
    ans

fn nth(i: Int) -> Int:
    var k = i
    var u = root
    unchecked:
        while true:
            let l = size(t[u].l)
            if k < l:
                u = t[u].l
            elif k == l:
                return t[u].key
            else:
                k -= l + 1
                u = t[u].r
    0

var rng: U64 = U64(246813579)
var keys: List<Int> = []
keys.reserve(100000)
for _ in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let x = Int((rng >> 33) % U64(10000000))
    root = insert(root, x).0
    keys.push(x)
var acc = 0
for i in 0..50000:
    let (r, gone) = erase(root, keys[i * 2])
    root = r
    if gone: acc += 1
let n = size(root)
for _ in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let q = Int((rng >> 33) % U64(10000000))
    acc += rank(q)
for _ in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let idx = Int((rng >> 33) % U64(n))
    acc += nth(idx)
print(acc + n)
