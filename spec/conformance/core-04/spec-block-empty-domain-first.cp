// expect: R0301
// rule: 04.4.21
// input: "0\n"
let m = read<0..=2e5>()
domain E = 0..m
let e = E.first!             // traps naming `E` when m == 0 // <- here
let first = E.first ?? return    // or handle it
