// Source: IOI 2004 P5 (Farmer), https://dmoj.ca/problem/ioi04p5
// Problem: M <= 2000 fields are cycles of N_i cypresses and K <= 2000 strips are rows of R_i
//   cypresses (3 <= N_i, 2 <= R_i <= 150), with an olive between each two neighbouring ones.
//   Pick Q <= 150000 cypresses; print the most olives with both neighbours picked.
// Solution: a whole field gives one olive per cypress, any other run of c cypresses c - 1.
//   If Q covers every field, take them all, then the longest strips first, each costing one
//   olive. Otherwise the answer is Q if some set of whole fields has exactly Q cypresses (a
//   Bitset subset sum over 0..=Q, O(M Q / 64)), else Q - 1 (one field is cut).
// original: corpus/submissions/4829839.cpp
// Bounds: q: 1..=150000; n, m: 0..=2000; fields: 3..=150; strips: 2..=150

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
    reach.add(Taken.first!)
    for len in fields: reach.union(reach.shifted(len))
    if reach.contains(Taken.last!): q else: q - 1
print(most())
