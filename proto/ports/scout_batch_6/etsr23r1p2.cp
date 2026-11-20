// ETSR 2023 Round 1 P2: a set of ranges starts empty. An order of kind 2 adds
// the range l..=r after throwing out every range that touches it, and prints
// how many it threw out. An order of kind 1 prints how many ranges there are.
// original: corpus/submissions/5494248.cpp
let (q, n): (1..=2e5, 1..=1e9) = read()                                             // WISH S8
@tags("1", "2")
enum Order:
    Count
    Add(l: 1..=n, r: l..=n)                                                         // WISH S8, S4

// No two ranges of the set overlap, so they are in the same order by start and
// by end.
var held: Map<Int, Int> = []            // each range, as start -> end
for _ in 0..q:
    match read::<Order>():                                                          // WISH S8
        Count => print(held.len)
        Add(l, r) =>
            var gone = 0
            while let (start, _) = held.first_ge(l) and start <= r:                 // WISH S13, S6
                held.remove(start)
                gone += 1
            if let (start, end) = held.last_lt(l) and end >= l:                     // WISH S13
                held.remove(start)
                gone += 1
            held[l] = r
            print(gone)
