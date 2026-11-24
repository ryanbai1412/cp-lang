// expect: E0303
// `Bitset<D>(5)` is not C++'s `bitset<N>(5)` (the bits of 5); it is an error, and members go through `Bitset.of(xs)`.
let b = Bitset<0..8>(5) // <- here
