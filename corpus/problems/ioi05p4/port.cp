// IOI 2005 Birthday, as it builds today: the circular gaps are a loop over
// list offsets with the wrap-around written beside it (batch 6 S17).
// original: corpus/submissions/5044895.cpp
let n: 1..=1e6 = read()
domain Seat = 1..=n

// a random seating, for stress tests (batch 8 H6: `Perm<Seat>` is not indexed by a seat)
fn random_wish() -> [Seat]Seat:
    var a: [Seat]Seat = [Seat] v => v
    for v in Seat:
        let w = Seat(rand(1, n))
        if v != w: swap(mut a[v], mut a[w])
    a

let wish: [Seat]Seat = read(gen: random_wish)

// With the circle turned by s, child i walks the distance between s and its
// own shift. The best turn is the middle of the arc that holds every shift.
fn worst(order: [Seat]Seat) -> Int:
    var shifts = [(i.label - order[i].label) % n for i in Seat]
    shifts.sort()
    var gap = shifts[0] + n - shifts[n - 1]
    for k in 1..n: gap = max(gap, shifts[k] - shifts[k - 1])
    (n - gap + 1) / 2

let mirrored = [Seat] i => wish[Seat(n + 1 - i.label)]
print(min(worst(wish), worst(mirrored)))
