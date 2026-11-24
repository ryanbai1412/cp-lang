// expect: ok, output "ok"
// input: "1 300\n"
// model: LinkedList
// LinkedList<Int> against a List: seeded random pushes and pops at both ends, splices of a fresh
// list on either side, reversals and rare clears, with both walks and both ends compared after
// each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.deque.{LinkedList}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var a: List<Int> = []
var l = LinkedList.from_list(a)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 8)
    let x = rng.int(0, 99)
    if op == 0:
        l.push(x)
        a.push(x)
    elif op == 1:
        l.push_front(x)
        a.insert(0, x)
    elif op == 2:
        let want = if a.len == 0: None else: Some(a.remove(a.len - 1))
        if l.pop() != want: bad = "op {k}: pop"
    elif op == 3:
        let want = if a.len == 0: None else: Some(a.remove(0))
        if l.pop_front() != want: bad = "op {k}: pop_front"
    elif op == 4 or op == 5:
        let b = [rng.int(0, 99) for _ in 0..rng.int(0, 4)]
        let o = if b.len == 1 and x % 2 == 0: LinkedList.of(b[0]) else: LinkedList.from_list(b)
        if op == 4:
            l.splice_back(o)
            a = a + b
        else:
            l.splice_front(o)
            a = b + a
    elif op == 6:
        l.reverse()
        a = [a[a.len - 1 - i] for i in 0..a.len]
    elif op == 7 and x < 5:
        l.clear()
        a = []
    let rev = [a[a.len - 1 - i] for i in 0..a.len]
    if bad == "" and (l.len != a.len or l.is_empty != (a.len == 0) or l.each().collect() != a or l.each_rev().collect() != rev or l.first != a.first or l.last != a.last):
        bad = "op {k}: contents {l.each().collect()} want {a}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
