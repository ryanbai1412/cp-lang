// expect: ok, output "0.750"
// rule: 13.3.6
// tool: run --release
// input: "5"
// The cold tail past a recursive call binds a `sorted(by:)` key's element
// itself, so the judge C++ of its own frame builds.
let top: Int = read()
var memo: Map<Int, F64> = [:]
fn wins(n: Int) -> F64:
    if let known = memo.get(n): return known
    var chances: List<F64> = []
    for k in 0..n: chances.push(1.0 - wins(k))
    let best = chances.sorted(by: x => desc(x)).take(2)
    let chance = if best.is_empty: 0.0 else: best.sum() / F64(best.len)
    memo[n] = chance
    chance

print("{wins(top):.3}")
