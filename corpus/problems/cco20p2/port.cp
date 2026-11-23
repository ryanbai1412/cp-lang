// Source: CCO 2020 Day 1 P2 (Exercise Deadlines), https://dmoj.ca/problem/cco20p2
// Problem: n <= 2e5 exercises in the order 1..=n, exercise i due by day d_i <= n; they are
//   done one a day in order, and swapping two neighbours costs one move. Print the fewest
//   moves after which every exercise meets its deadline, or -1.
// Solution: fill the days from the last one: any unplaced exercise due that day or later
//   may take it, and the one furthest right (MaxHeap) is the cheapest to bring there; none
//   means -1. The moves are the inversions of the order built, counted with a Fenwick of
//   exercises already given a later day that started further left. O(n log n).

let n: Int = read()
domain Ex = 1..=n  // an exercise, by where it starts
domain Day = 1..=n
let due: [Ex]Day = read()
var due_on: [Day]List<Ex> = []
for e in Ex: due_on[due[e]].push(e)

fn moves() -> ?Int:
    var free = MaxHeap<Ex>()         // not placed yet, and allowed on this day
    var placed = Fenwick<Ex, Int>()  // 1 for every exercise given a later day
    var total = 0
    for day in Day.rev:
        for e in due_on[day]: free.push(e)
        if let e = free.pop():
            total += placed.prefix(e.l)  // later days, but started further left
            placed.add(e, 1)
        else: return None
    total
print(moves() ?? -1)
