// Source: CCO 2024 Day 1 P2, Pizza Party, https://dmoj.ca/problem/cco24p2
// limits: 4000 ms, 1024 MB
// Problem: pizzas a_1..a_N leave a cart top first, each onto a new stack or the top of one;
//   then people b_1..b_N each take a pizza of their flavour from some stack top (N <= 1e6,
//   flavours <= N). Print the fewest stacks K, each pizza's stack and each person's stack, or
//   -1 if impossible.
// Solution: partial: subtasks 1-3 only (flavours <= 2, or distinct a: the judged ones, the
//   statement notes 4-5 may have no efficient solution); other input prints nothing. Flavour
//   counts must match. Flavours <= 2: one stack if b is a reversed, else stack = flavour.
//   Distinct a: a person must take a pizza laid later than
//   the next person's, so cover b by chains decreasing in placement order: greedy patience
//   piles, binary-searching the first pile whose top was laid after the new pizza. O(N log N).
// Bounds: n: 1..=1_000_000
// original: corpus/submissions/6435481.cpp
let n: 1..=1e6 = read()
domain Flavour = 1..=n
domain Pizza = 0..n
domain Person = 0..n
let a: [Pizza]Flavour = read()
let b: [Person]Flavour = read()

fn print_stacks(k: Int, stack: Flavour -> Int):
    print(k)
    print([stack(a[p]) for p in Pizza])
    print([stack(b[q]) for q in Person])

if not a.is_permutation(b):
    print(-1)
elif a.all(f => f.label <= 2):
    if b.collect() == a.rev.collect(): print_stacks(1, _ => 1)
    else: print_stacks(2, f => f.label)
elif a.is_permutation(Flavour.collect()):
    let laid = a.inverse()
    var tops: List<Pizza> = []
    var stack: [Flavour]Int = 0
    for f in b:
        let p = laid[f]
        let i = (0..tops.len).first_true(i => tops[i] > p) ?? tops.len
        if i == tops.len: tops.push(p)
        else: tops[i] = p
        stack[f] = i + 1
    print_stacks(tops.len, f => stack[f])
