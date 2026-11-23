// Source: IOI 2018 P4 (Mechanical Doll), https://dmoj.ca/problem/ioi18p4
// Problem: m <= 1e5 triggers and a sequence A of n <= 2e5 of them. Build a circuit: an exit
//   for the origin and each trigger, and switches (exits X, Y, alternating from X) so a ball
//   from the origin enters the triggers exactly in order A, then returns to the origin with
//   every switch at X. Use at most 2n switches; full score needs about n + log2 n.
// Solution: every exit goes to the root of one complete binary tree of switches with 2^L >=
//   n + 1 leaves. The ball's t-th pass reaches leaf bitrev(t); the 2^L - n - 1 leftmost leaves
//   send it back to the root, and the rest, in pass order, get A and then the origin. Subtrees
//   of only such leaves collapse into an edge to the root: at most n + L switches. O(n).
// original: corpus/submissions/4724581.cpp (grader function create_circuit)
// stdin here: M n, then A[0..n).
// stdout: three lines, C[0..=M], X and Y (each X/Y line starts with the count).
let m: 1..=100000 = read()
let n: 1..=200000 = read()
domain Seq = 0..n
let a: [Seq]1..=m = read()

// 2^L leaves >= n + 1 targets (A, then the origin)
let l = bit_len(n)
let leaves = 1 << l
let unused = leaves - (n + 1)

fn bitrev(p: Int) -> Int:
    var r = 0
    var v = p
    for b in 0..l:
        r = r * 2 + (v & 1)
        v = v >> 1
    r

// visit number t lands on tree position bitrev(t); hand out the targets to
// the used positions in visit order
domain Pos = 0..leaves
var target: [Pos]Int = 0
var next = 0
for t in 0..leaves:
    let p = bitrev(t)
    if p >= unused:
        if next < n:
            target[Pos(p)] = a[Seq(next)]
        next += 1

var x: List<Int> = []
var y: List<Int> = []
fn build(lo: Int, hi: Int) -> Int:
    if hi <= unused:
        return -1
    if hi - lo == 1:
        return target[Pos(lo)]
    let s = x.len
    x.push(0)
    y.push(0)
    let mid = (lo + hi) / 2
    let left = build(lo, mid)
    let right = build(mid, hi)
    x[s] = left
    y[s] = right
    -(s + 1)

let root = build(0, leaves)
var c: List<Int> = []
for i in 0..(m + 1):
    c.push(root)
print(c)
print(x.len, x)
print(y.len, y)
