// NOIP 2006 Energy Necklace: n beads in a ring; bead i has head mark[i] and
// its tail is the head of the next bead. Merging two neighbouring groups
// releases head(left) * head(right) * tail(right). Maximise the total
// released while merging the whole ring into one bead.
let n: 4..=100 = read()
domain Bead = 0..n
let mark: [Bead]1..=1000 = read()

// best(i, len): the most energy from merging the len beads from bead i on,
// going round the ring; the last merge joins the first k to the rest
@memo
fn best(i: Bead, len: 1..=n) -> Int =
    [best(i, k) + best(
            Bead.wrap(i.ord + k),
            len - k,
        ) + mark[i] * mark[Bead.wrap(i.ord + k)] * mark[Bead.wrap(i.ord + len)] for k in 1..len].max() ?? 0
print([best(i, n) for i in Bead].max()!)
