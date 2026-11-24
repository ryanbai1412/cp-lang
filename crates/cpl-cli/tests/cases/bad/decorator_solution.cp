// error: unknown decorator `@solution`
// Once bench/repros/bug_02: `cpl build` emitted an empty `void solve() {}` for this.
@solution
fn main():
    let n: Int = read()
    print(n + 1)
