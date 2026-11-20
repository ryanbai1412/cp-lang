// NOIP 2006 Energy Necklace: n beads in a ring; bead i has head mark[i] and
// its tail is the head of the next bead. Merging two neighbouring groups
// releases head(left) * head(right) * tail(right). Maximise the total
// released while merging the whole ring into one bead.
let n: 4..=100 = read()
domain Bead = 0..n
let mark: [Bead]1..=1000 = read()

// best(i, len): the most energy from merging the `len` beads starting at
// bead i (going round the ring) into one; the last merge splits them at k
@memo
fn best(i: Bead, len: 1..=n) -> Int:
    max(
        best(i, k) + best(
            Bead((i.idx + k) % n),
            len - k,
        ) + mark[i] * mark[Bead((i.idx + k) % n)] * mark[Bead((i.idx + len) % n)] for k in 1..len
    ) ?? 0

print(max(best(i, n) for i in Bead))
