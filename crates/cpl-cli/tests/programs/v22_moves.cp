// v2.2 ownership and order: `.take()`, `swap`, `push_id`,
// `prev_permutation`, and an assignment's indices and right side run
// before its place is reached for.
use std.seq.{next_permutation, prev_permutation}

var a: List<Int> = [1, 2, 3]
var b: List<Int> = [9]
if b.len < a.len: swap(mut a, mut b)
print(a, "|", b)
let moved = b.take()
print(moved, b.len, yes(b.is_empty))
var rows: List<List<Int>> = [[1], [2, 3]]
swap(mut rows[0], mut rows[1])
print(rows[0], "|", rows[1])
var word = "abc"
let old = word.take()
print(old, word.len)

var p: List<Int> = [3, 2, 1]
var seen = 1
while prev_permutation(mut p): seen += 1
print(seen, p)
print(yes(next_permutation(mut p)), p)

struct Node:
    var ends: Bool = false
    var next: ?Int = none

var trie: List<Node> = [Node()]
fn push_id(v: Node) -> Int:
    let id = trie.len
    trie.push(v)
    id
fn grow() -> Int:
    push_id(Node())

// The list grows while the index is worked out; the place is found after.
trie[grow()].ends = true
trie[0].next ??= push_id(Node())
trie[0].next ??= push_id(Node())
print(trie.len, yes(trie[1].ends), trie[0].next ?? -1)

var v: List<Int> = [1]
fn bump() -> Int:
    v.push(5)
    7
v[0] = bump()
v[0] += bump()
v[v.len - 1] max= bump()
print(v)

var m = Map<Int, Str>()
m[1] = "one"
print(m.remove(1) ?? "-", m.remove(1) ?? "-", m.len)
