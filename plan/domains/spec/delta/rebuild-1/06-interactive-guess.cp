// cpl 0.3.0
// Problem: guess a hidden h in 1..=n with at most 30 queries.
// Input: n. Query "? x"; the judge answers "Y" if h <= x, else "N".
// Finish with "! h".
// Guesses: H-BSEARCH-DEBUG-PROBES (checked builds call the predicate on both
// sides of the answer: extra queries in an interactive task),
// H-LITERAL-UNION-RETURN.

let wire = Wire()
let n = read<1..=1e9>()
fn ask(x: Int) -> "Y" | "N" = wire.ask("? {x}")
let h = (1..=n).first_true(x => ask(x) == "Y")!
print("! {h}")
