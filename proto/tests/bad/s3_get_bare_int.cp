// error: expects V or ?V
// Synthesis #3 Leap 3: `a.get(?I)` rejects bare `Int` on a nominal-domain array.
let n: 1..=10 = read()
domain V = 1..=n
let a: [V]Int = read()
print(a.get(0) ?? 0)
