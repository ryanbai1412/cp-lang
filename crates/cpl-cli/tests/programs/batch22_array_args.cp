// A list written out where a one-index array is declared is that array's
// cells, in a constructor and in a call as in a typed `let`; an array goes
// where a `List` is asked for; `Map` fns work on a `HashMap`.
type Z = Mod<998244353>

struct Node:
    kid: [0..2]?Int
    cnt: Int

struct Tri:
    at: [1..=3]Int
    name: Str

fn take(a: [0..3]Int) -> Int = a[0] + 10 * a[2]

fn first(a: [0..2]?Int, fallback: Int) -> Int = a[0] ?? a[1] ?? fallback

fn mul(a: List<Z>, b: List<Z>) -> List<Z>:
    var out: List<Z> = List.filled(a.len + b.len - 1, 0)
    for i in 0..a.len:
        for j in 0..b.len: out[i + j] += a[i] * b[j]
    out

let n: Int = read()
var t = [Node([None, None], 0)]
t.push(Node([Some(1), None], 2))
t.push(Node(kid: [None, Some(n)], cnt: 5))
t[0].kid[1] = Some(7)
for node in t: print(node.kid[0] ?? -1, node.kid[1] ?? -1, node.cnt)
let tri = Tri([n, n + 1, n + 2], "tri")
print(tri.at[1], tri.at[3], tri.name)
print(take([1, 2, 3]), take([n, 0, n]))
print(first([None, None], 4), first([None, Some(n)], 4), first([Some(2), None], 4))

var a: [0..n]Z = 1
var b: [0..n]Z = 2
for i in 0..n: a[i] += i
print(mul(a, b))

var h = HashMap<Int, Int>()
print(h.get_or_insert(1, 2), h.get_or_insert(1, 9))
print(h.get_or_insert(1, 5), h.get_or_insert(7, 5))
h[7] += 1
print(h.len, h[7])
var hs = HashMap<Str, Int>()
for name in ["x", "y", "x", "z", "y"]:
    let next = hs.len
    print(hs.get_or_insert(name, next))

// `s.remove(x)` for its effect alone, and for its answer.
var seen = Set<Int>()
seen.insert(3)
seen.insert(4)
seen.remove(3)
let gone = seen.remove(9)
if seen.remove(4): print("had 4")
print(seen.len, yes(gone))
