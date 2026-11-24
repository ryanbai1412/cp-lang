// expect: E0302
// rule: 12.4
// The result of a call over a bound is known, not left unchecked: `tally`
// gives a `Counter<Int, Int>`, which is not a `Str`.
let s: Str = [1, 2, 1].tally() // <- here
