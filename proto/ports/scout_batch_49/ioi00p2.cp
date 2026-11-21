// IOI 2000 Day 1 P2, Car Parking
// original: corpus/submissions/4682555.cpp
let (n, m, w): (1..=20_000, 1..=50, 2..=50) = read()
domain Slot = 1..=n
type Kind = 1..=m
var park: [Slot]Kind = read()
let want = park.sorted()

// cursor[t]: first slot of type t's block not yet known to hold a type-t car
var cursor: [Kind]?Slot = [Kind] t => want.first_ge(t)

var rounds: List<List<(Slot, Slot)>> = []
var todo = [s for s in Slot if park[s] != want[s]]     // ascending; a slot leaves once it is right
reverse(mut todo)
while not todo.is_empty:
    var moves: List<(Slot, Slot)> = []
    // each chain needs two free drivers: one to leave slot `home`, one to fill it
    while let home = todo.last and moves.len + 2 <= w:
        if park[home] == want[home]:
            todo.pop()
            continue
        var (from, car) = (home, park[home])
        while car != want[home] and moves.len + 2 <= w:
            // FRICTION: "advance an optional cursor while a predicate holds" is a hand-written loop
            while park[cursor[car]!] == car: cursor[car] = cursor[car]! + 1
            let to = cursor[car]!
            moves.push((from, to))
            (park[to], car) = (car, park[to])
            from = to
        moves.push((from, home))
        park[home] = car
    if not moves.is_empty: rounds.push(moves)

print(rounds.len)
for moves in rounds:
    print(moves.len, " ".join("{a} {b}" for (a, b) in moves))
