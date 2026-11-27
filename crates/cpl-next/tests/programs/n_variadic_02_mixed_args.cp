// expect: E0303
// rule: 06.1.5
// One call gives `..xs` positional or named arguments, not both.
fn pack<Ts>(..xs: Ts) -> Ts = xs
let p = pack(1, n: 2) // <- here
