// Repro: `@multitest fn solve():` emits an empty `void solve() {}` body in C++17.
// Discovered in: proto/ports/scout_batch_3/ccc03s4.cp (disagreement triage)
// Expected: prints `2 * x` for each test case.
// Actual (`cpl build`): emits `void solve() {}` and prints nothing (empty stdout).

@multitest
fn solve():
    let x: Int = read()
    print(x * 2)
