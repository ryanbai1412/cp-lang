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
// checker: corpus/problems/ioi18p4/checker.py
// stdin here: M n, then A[0..n).
// stdout: three lines, C[0..=M], X and Y (each X/Y line starts with the count).
// Bounds: m: 1..=100_000; n: 1..=200_000; a: 1..=m
let m: 1..=100_000 = read()
let n: 1..=200_000 = read()
domain Seq = 0..n
let a: [Seq]1..=m = read()

enum Exit: Origin | Trigger(Int) | Switch(Int)
fn code(e: Exit) -> Int:
    match e:
        Origin => 0
        Trigger(t) => t
        Switch(s) => -(s + 1)

// one complete tree of switches, 2^L >= n + 1 leaves; the t-th pass reaches leaf bitrev(t)
let depth = bit_len(n)
domain Leaf = 0..1 << depth
let unused = Leaf.count - (n + 1)
fn bitrev(t: Leaf) -> Leaf:
    var r = 0
    for b in 0..depth:
        r = 2 * r + ((Int(t) >> b) & 1)
    Leaf(r)

// the leftmost unused leaves send the ball back to the root; the rest, in pass order, get
// A and then the origin
let root = Switch(0)
var exit_at: [Leaf]Exit = root
for (k, p) in Leaf.map(bitrev).filter(p => Int(p) >= unused).collect().items:
    exit_at[p] = if k < n: Trigger(Int(a[Seq(k)])) else: Origin

// a subtree of only unused leaves collapses into an edge to the root
var switches: List<(x: Exit, y: Exit)> = []
fn build(span: Range<Leaf>) -> Exit:
    if Int(span.r) <= unused:
        return root
    if span.len == 1:
        return exit_at[span.first!]
    let s = switches.len
    switches.push((x: root, y: root))
    let x = build(span.l..span.mid)
    let y = build(span.mid..span.r)
    switches[s] = (x:, y:)
    Switch(s)

let top = build(Leaf.first!..=Leaf.last!)
print([code(top) for _ in 0..=m])
print(switches.len, switches.map(s => code(s.x)).collect())
print(switches.len, switches.map(s => code(s.y)).collect())
