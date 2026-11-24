// Source: IOI 2022 P2 Prisoner Challenge, https://dmoj.ca/problem/ioi22p2
// Problem: bags A and B hold distinct coin counts in 1..=n (n <= 5000). Prisoners enter one at
//   a time, read the whiteboard i in 0..=x, open one bag (chosen by i) and see its count j, then
//   overwrite the whiteboard or name the smaller bag. Print a strategy table s[i][j] that always
//   names it correctly; full score needs x <= 20 (here 21 states).
// Solution: a fixed tree over 1..=5588: a node [lo, hi] keeps lo and hi itself and splits the
//   interior evenly among b children, b = 3, 3, 3, 3, 3, 2, 2, 1 by depth. State (depth t, child
//   c) means both counts share a depth t - 1 node and the last bag opened went into child c;
//   bags alternate by depth. A count at an endpoint or in another child settles the order, the
//   same child descends a level. 1 + sum b = 21 states, O(n * states * depth) to print.
// original: corpus/submissions/4783608.cpp (grader function devise_strategy)
// checker: corpus/problems/ioi22p2/checker.py
// stdin: n. stdout: the number of states, then per state the bag it opens and the action for
//   each count 1..=n (-1 / -2 name bag A / B as smaller, else the state to write).
let n: 2..=5000 = read()
domain Count = 1..=n
let fanout = [3, 3, 3, 3, 3, 2, 2, 1]
let depth = fanout.len

// size[t]: the counts a node at depth t covers; a leaf (depth `depth`) covers 2
var size = [2]
for b in fanout.rev: size.push(b * size.last! + 2)
size.reverse()

// the state (depth t, child 0); depth 0 has the one start state
let before = fanout.scan(+)
fn first_state(t: Int) -> Int = if t == 0: 0 else: 1 + before.cut(t - 1)

enum Step: Low | High | Child(Int)
// v's walk down the tree: the child it takes at each depth, then the endpoint it is
fn path(v: Int) -> List<Step>:
    var out: List<Step> = []
    var lo = 1
    for t in 0..=depth:
        if v == lo: return out + [Step.Low]
        if v == lo + size[t] - 1: return out + [Step.High]
        let c = (v - lo - 1) / size[t + 1]
        out.push(Step.Child(c))
        lo += 1 + c * size[t + 1]
    out
let paths: [Count]List<Step> = [Count] v => path(Int(v))

enum Bag: A | B
enum Act: Smaller(Bag) | Write(Int)
fn other(b: Bag) -> Bag = if b == Bag.A: Bag.B else: Bag.A
fn code(a: Act) -> Int = match a:
    Act.Smaller(b) => if b == Bag.A: -1 else: -2
    Act.Write(s) => s

// the opened count left the node by `step` while the other count went into child `c`
fn settle(bag: Bag, step: Step, c: Int) -> Act = match step:
    Step.Low => Act.Smaller(bag)
    Step.High => Act.Smaller(other(bag))
    Step.Child(mine) => Act.Smaller(if mine < c: bag else: other(bag))

// at depth t the opened count goes on into a child, or is an endpoint and settles
fn descend(bag: Bag, step: Step, t: Int) -> Act = match step:
    Step.Child(k) => Act.Write(first_state(t + 1) + k)
    _ => settle(bag, step, 0)

print(first_state(depth + 1))
for t in 0..=depth:
    let bag = if t % 2 == 0: Bag.A else: Bag.B
    for c in 0..(if t == 0: 1 else: fanout[t - 1]):
        let acts = [Count] v =>
            let p = paths[v]
            // a count whose walk stops above depth t - 1 never sees this state
            if t == 0: descend(bag, p[0], 0)
            elif t - 1 >= p.len: Act.Write(0)
            elif p[t - 1] == Step.Child(c): descend(bag, p[t], t)
            else: settle(bag, p[t - 1], c)
        print([Int(bag == Bag.B)] + [code(acts[v]) for v in Count])
