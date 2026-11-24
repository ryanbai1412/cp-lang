// Source: COCI 2021/2022 Contest 6 #3 (Naboj), https://dmoj.ca/problem/coci21c6p3
// limits: 1000 ms, 512 MB
// Problem: a connected simple graph, n <= 2e5 balls and 1 <= m <= 5e5 wires, each wire
//   wanting its electrons at a given end. Charging a ball + pulls every wire at it towards
//   it, - pushes them away; the last charge on a wire's ends decides it. Print -1 or a sequence
//   of k <= 2e5 chargings.
// Solution: think backwards: the last ball charged must have all its remaining wires pointing
//   the same way (in- or out-degree 0), and its wires are then settled. Peel such balls with a
//   stack, updating degrees, like a topological sort; if some ball is never peeled the wires
//   hold a cycle and the answer is -1. Print the peel order reversed, O(n + m).
// original: corpus/submissions/4530006.cpp
// Bounds: n: 1..=2e5; m: 1..=5e5; wires.u, wires.v: 1..=n

let n: 1..=2e5 = read()
let m: 1..=5e5 = read()
domain Ball = 1..=n
domain Wire = 0..m
let wires: [Wire](u: Ball, v: Ball) = read()
let out = Graph.directed(wires)
let into = out.transpose()

// unsettled wires leaving and entering each ball
var out_left: [Ball]Int = 0
var in_left: [Ball]Int = 0
for w in wires:
    out_left[w.u] += 1
    in_left[w.v] += 1

var peeled: [Ball]Bool = false
var todo: List<Ball> = []
fn offer(b: Ball):
    if not peeled[b] and (in_left[b] == 0 or out_left[b] == 0):
        peeled[b] = true
        todo.push(b)
for b in Ball: offer(b)

var order: List<(ball: Ball, charge: 0..=1)> = []
while let b = todo.pop():
    order.push((ball: b, charge: if in_left[b] == 0 and out_left[b] > 0: 1 else: 0))
    for w in out.next(b):
        in_left[w] -= 1
        offer(w)
    for w in into.next(b):
        out_left[w] -= 1
        offer(w)

if Ball.all(b => peeled[b]):
    print(order.len)
    for step in order.rev: print(step.ball, step.charge)
else: print(-1)
