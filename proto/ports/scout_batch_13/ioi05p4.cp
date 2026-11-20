// IOI 2005 Birthday: n children sit round a table, and each wants a given
// pair of neighbours, so the wanted circle is fixed up to turning it and
// reading it the other way. Children swap seats; make the longest walk as
// short as possible.
// original: corpus/submissions/5044895.cpp
let n: 1..=1e6 = read()
domain Seat = 1..=n
let wish: Perm<Seat> = read()                                       // WISH H6: indexed by a seat

// With the circle turned by s, child i walks the distance round the table
// between s and its own shift. The best turn is the middle of the arc that
// holds every shift, which is the table minus the widest gap between shifts.
fn worst(order: Perm<Seat>) -> Int:
    let shifts = sorted((i - order[i]) % n for i in Seat)
    let gap = max(b - a for (a, b) in (shifts ++ [shifts[0] + n]).adjacent)     // WISH S17
    (n - gap + 1) / 2

print(min(worst(wish), worst(wish.rev)))                            // WISH S17
