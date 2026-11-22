// P0: bsearch_first / bsearch_last over 1..=1e9 call the predicate up to 33 / 31 times;
// ceil(log2(1e9)) = 30 is enough. The io-and-formatting.md interactive example
// (bsearch_first + ask(limit: 30)) exceeds its own query budget.
// Expected: 30 or fewer calls on every line. Was: 1 33 / 2 32 / 1000000000 31; fixed, and
// stdlib/seq.cp binary_search_asks_at_most_log2_of_the_outcomes is the regression test.
var calls = 0
let s: Int = read()
fn ge(y: Int) -> Bool:
    calls += 1
    y >= s
print(bsearch_first(1..=1_000_000_000, ge)!, calls)
