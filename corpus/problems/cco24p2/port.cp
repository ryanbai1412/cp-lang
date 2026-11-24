// Source: CCO 2024 Day 1 P2, Pizza Party, https://dmoj.ca/problem/cco24p2
// limits: 4000 ms, 1024 MB
// Problem: pizzas a_1..a_N leave a cart top first, each onto a new stack or the top of one;
//   then people b_1..b_N each take a pizza of their flavour from some stack top (N <= 1e6,
//   flavours <= N). Print the fewest stacks K, each pizza's stack and each person's stack, or
//   -1 if impossible.
// Solution: flavour counts must match. Flavours <= 2: one stack if b is a reversed, else stack =
//   flavour. Distinct a: a person must take a pizza laid later than the next person's, so cover
//   b by chains decreasing in placement order: greedy patience piles, binary-searching the first
//   pile whose top was laid after the new pizza. O(N log N). Otherwise exact only for N <= 10
//   (other input prints nothing; the statement notes subtasks 4-5 may have no efficient
//   solution, and the judge's data has none): match pizzas in order to waiting people of their
//   flavour; a stack is pizzas whose people come in reverse placement order, so K is the fewest
//   patience piles over the matched people, pruned at the best so far. At most 10! leaves.
// Bounds: n: 1..=1e6; a, b: 1..=n
// original: corpus/problems/cco24p2/orig.cpp (6435481 plus the N <= 10 search)
let n: 1..=1e6 = read()
domain Flavour = 1..=n
domain Pizza = 0..n
domain Person = 0..n
let a: [Pizza]Flavour = read()
let b: [Person]Flavour = read()

fn print_stacks(k: Int, pizza: Pizza -> Int, person: Person -> Int):
    print(k)
    print([pizza(p) for p in Pizza])
    print([person(q) for q in Person])

var surplus: [Flavour]-1e6..=1e6 = 0
for f in a: surplus[f] += 1
let distinct = surplus.all(c => c <= 1)
for f in b: surplus[f] -= 1
if surplus.any(c => c != 0):
    print(-1)
elif a.all(f => f.label <= 2):
    if b.collect() == a.rev.collect(): print_stacks(1, _ => 1, _ => 1)
    else: print_stacks(2, p => a[p].label, q => b[q].label)
elif distinct:
    let laid = a.inverse()
    var piles: List<Pizza> = []
    var stack: [Flavour]0..=1e6 = 0
    for f in b:
        let p = laid[f]
        let i = (0..piles.len).first_true(i => piles[i] > p) ?? piles.len
        if i == piles.len: piles.push(p)
        else: piles[i] = p
        stack[f] = i + 1
    print_stacks(piles.len, p => stack[a[p]], q => stack[b[q]])
elif n <= 10:
    // The search's state: who is matched so far, each pile's latest person, and the best piles yet.
    var waiting: [Flavour]List<Person> = []
    for q in Person.rev: waiting[b[q]].push(q)
    var taken: [Person]Bool = false
    var tops: List<Person> = []
    var pizza_pile: [Pizza]Int = 0
    var person_pile: [Person]Int = 0
    var best = n + 1
    var best_pizza = pizza_pile
    var best_person = person_pile

    // Pizzas p.. matched to waiting people, each onto the leftmost pile whose latest person comes later.
    fn search(p: Int):
        if tops.len >= best: return
        if p == n:
            best = tops.len
            best_pizza = pizza_pile
            best_person = person_pile
            return
        for q in waiting[a[Pizza(p)]] if not taken[q]:
            let i = (0..tops.len).first_true(i => tops[i] > q) ?? tops.len
            let old = if i < tops.len: Some(tops[i]) else: None
            if i == tops.len: tops.push(q)
            else: tops[i] = q
            taken[q] = true
            pizza_pile[Pizza(p)] = i + 1
            person_pile[q] = i + 1
            search(p + 1)
            taken[q] = false
            if let top = old: tops[i] = top
            else: tops.pop()

    search(0)
    print_stacks(best, p => best_pizza[p], q => best_person[q])
