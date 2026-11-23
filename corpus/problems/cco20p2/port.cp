// CCO 2020 Exercise Deadlines: n exercises in a row, each with a deadline.
// They are done left to right, one a day, and swapping two neighbours costs
// one move. Print the fewest moves that meet every deadline, or -1.
//
// Fill the days from the last one: any exercise due that day or later can
// take it, and the one furthest right is the cheapest to bring there. The
// moves are then the inversions of the order chosen, counted as it is built.

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
        else: return none
    total
print(moves() ?? -1)
