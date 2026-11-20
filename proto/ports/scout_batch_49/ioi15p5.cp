// IOI 2015 Day 2 P2, Sorting
// original: corpus/submissions/5723358.cpp
@grader
fn findSwapPairs(start: Perm<Slot>, ermek: List<(Slot, Slot)>) -> List<(Slot, Slot)>:
    domain Slot                                  // inferred from the shape of start
    var row = start.copy()
    var slot_of = row.inv()
    // FRICTION: Perm has no "swap two entries and keep the inverse in step"
    fn exchange(a: Slot, b: Slot):
        swap(mut row[a], mut row[b])
        swap(mut slot_of[row[a]], mut slot_of[row[b]])
    // rewinds to the start and plays Ermek's first t swaps alone
    fn ermek_alone(t: Int):
        row = start.copy()
        slot_of = row.inv()
        for (x, y) in ermek[..t]: exchange(x, y)

    // Our swaps can be chosen on values instead of slots, so they commute with
    // his: t rounds are enough exactly when his row is n - cycles <= t swaps from sorted.
    let rounds = bsearch_first(0..=ermek.len, t =>
        ermek_alone(t)
        Slot.K - row.cycles().len <= t)!

    // Sort the row Ermek alone leaves, recording each swap as a pair of values.
    ermek_alone(rounds)
    var pairs: List<(Slot, Slot)> = []
    for v in Slot if row[v] != v:
        pairs.push((v, row[v]))
        exchange(v, slot_of[v])

    // Replay: after his swap each round, swap the slots those two values sit in now.
    ermek_alone(0)
    var ours: List<(Slot, Slot)> = []
    for (t, (x, y)) in ermek[..rounds].items:
        exchange(x, y)
        let (a, b) = if t < pairs.len: (slot_of[pairs[t].0], slot_of[pairs[t].1]) else: (Slot.first, Slot.first)
        ours.push((a, b))
        exchange(a, b)
    ours
