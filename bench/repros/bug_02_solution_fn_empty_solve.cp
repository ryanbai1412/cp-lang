// Repro: `@solution fn main():` emits an empty `void solve() {}` body in C++17.
// Discovered in: proto/ports/blind_critic_2/sac22cc2p4.cp (fixed port by moving to top-level)
// Expected: reads `n` and prints `n + 1`.
// Actual (`cpl build`): emits `void solve() {}` and prints nothing (empty stdout).

@solution
fn main():
    let n: Int = read()
    print(n + 1)
