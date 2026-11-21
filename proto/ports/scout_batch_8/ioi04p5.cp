// IOI 2004 Farmer: q trees are to be taken from fields (cycles of a[i]
// trees) and strips (paths of b[j] trees). A tree counts when both its
// neighbours are taken too. Print the most that can count.
// original: corpus/submissions/4829839.cpp

let q: 1..=150000 = read()
let n: 0..=2000 = read()
let m: 0..=2000 = read()
domain Field = 0..n
domain Strip = 0..m
let fields: [Field]3..=150 = read()
let strips: [Strip]2..=150 = read()

fn most() -> Int:
    let whole = fields.sum()
    if q >= whole:
        // Every field, then the longest strips: each strip started loses one tree.
        var (left, got) = (q - whole, whole)
        for len in strips.sorted().rev:
            let take = min(left, len)
            got += max(0, take - 1)
            left -= take
        return got
    // Whole fields that add up to exactly q lose nothing; otherwise one cut loses one.
    domain Taken = 0..=q
    var reach = Bitset<Taken>.new()
    reach.add(Taken.first)
    for len in fields: reach.union(reach.shifted(len))
    if reach.contains(Taken.last): q else: q - 1
print(most())
