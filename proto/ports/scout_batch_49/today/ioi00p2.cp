// IOI 2000 Day 1 P2, Car Parking (buildable today-form)
// original: corpus/submissions/4682555.cpp
// stdin: n m w, then the n car types. stdout: the number of rounds, then
// per round the number of drivers and their (from, to) slots.
let n: 1..=20000 = read()
let m: 1..=50 = read()
let w: 2..=50 = read()
var park: [0..n]Int = read()
var want: List<Int> = [park[i] for i in 0..n]
sort(mut want)

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
