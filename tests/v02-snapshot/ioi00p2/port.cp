// Source: IOI '00 P2 (Car Parking), https://dmoj.ca/problem/ioi00p2
// Problem: a row of n <= 20000 parked cars of types 1..m (m <= 50) and w workers,
//   2 <= w <= m. In a round each worker drives one car out and parks it in a slot freed in
//   the same round. Sort the row by type in at most ceil(n / (w - 1)) rounds; print the rounds.
// Solution: `want` is the sorted row and cursor[t] the next slot of type t's block that may
//   still be wrong. A round walks left to right: at the first misplaced slot i, follow the
//   chain (car goes to its block's next misplaced slot, the car there moves on, ...) until a
//   car belonging at i turns up or only two drivers are left, then the carried car fills i.
//   Every driver but one per chain fixes a slot, so each round fixes about w - 1 cars: O(n).
// original: corpus/submissions/4682555.cpp
// stdin: n m w, then the n car types. stdout: the number of rounds, then
// per round the number of drivers and their (from, to) slots.
let n: 1..=20000 = read()
let m: 1..=50 = read()
let w: 2..=50 = read()
var park: [0..n]Int = read()
var want: List<Int> = [park[i] for i in 0..n]
want.sort()

// cursor[t]: first slot of type t's block not yet known to hold a type-t car
var cursor: [0..=m]Int = n
for i in (0..n).rev: cursor[want[i]] = i

var rounds: List<List<Int>> = []
var i = 0
while i < n:
    var moves: List<Int> = []
    // every driver chain needs two free drivers: one to leave slot i, one to fill it
    while i < n and moves.len + 4 <= 2 * w:
        while i < n and park[i] == want[i]: i += 1
        if i == n: break
        var from = i
        var car = park[i]
        while car != want[i] and moves.len + 4 <= 2 * w:
            while park[cursor[car]] == car: cursor[car] += 1
            let to = cursor[car]
            moves.push(from + 1)
            moves.push(to + 1)
            swap(mut park[to], mut car)
            from = to
        moves.push(from + 1)
        moves.push(i + 1)
        park[i] = car
    if moves.len > 0: rounds.push(moves)
print(rounds.len)
for moves in rounds:
    var line: List<Int> = []
    line.push(moves.len / 2)
    for x in moves: line.push(x)
    print(line)
