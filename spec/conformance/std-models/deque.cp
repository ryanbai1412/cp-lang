// expect: ok, output "ok"
// input: "1 300\n"
// model: Deque
// Deque<Int> against a List: seeded random pushes, pops, sets and rotations at both ends,
// with every element compared after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var d = Deque<Int>()
var a: List<Int> = []
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let x = rng.int(0, 99)
    if op == 0:
        d.push(x)
        a.push(x)
    elif op == 1:
        d.push_front(x)
        a.insert(0, x)
    elif op == 2:
        let want = if a.len == 0: None else: Some(a.remove(a.len - 1))
        if d.pop() != want: bad = "op {k}: pop"
    elif op == 3:
        let want = if a.len == 0: None else: Some(a.remove(0))
        if d.pop_front() != want: bad = "op {k}: pop_front"
    elif op == 4 and a.len > 0:
        let i = x % a.len
        d.set(i, x)
        a[i] = x
    elif op == 5 and a.len > 0:
        let r = x % a.len
        d.rotate_left(r)
        for _ in 0..r:
            a.push(a.remove(0))
    elif op == 6 and a.len > 0:
        let r = x % a.len
        d.rotate_right(r)
        for _ in 0..r:
            a.insert(0, a.remove(a.len - 1))
    if bad == "" and (d.len != a.len or d.to_list() != a or d.first != a.first or d.last != a.last):
        bad = "op {k}: contents {d.to_list()} want {a}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
