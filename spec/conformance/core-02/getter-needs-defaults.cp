// expect: E0317 at 4
// rule: 02.6.6
// A getter is read like a field, so every parameter but `self` needs a default; `k` has none.
struct P:
    x: Int
    @get fn add(self, k: Int) -> Int = self.x + k
let p = P(x: 1)
print(p.add(2))
