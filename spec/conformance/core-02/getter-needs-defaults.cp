// expect: E0317
// rule: 02.6.6
// A getter is read like a field, so every parameter but `self` needs a default; `k` has none.
struct P:
    x: Int
    @get fn add(self, k: Int) -> Int = self.x + k // <- here
let p = P(x: 1)
print(p.add(2))
