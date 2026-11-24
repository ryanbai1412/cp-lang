// expect: E0105
// has: "[Pair<V>]"
// A ragged read says how the input is written, not what the data is: the triangle of pairs is `[Pair<V>]T`, read in the domain's walk order (indices-and-keys.md, "Lost").
domain V = 0..3
let t = read<[i in V][..i]Int>() // <- here
