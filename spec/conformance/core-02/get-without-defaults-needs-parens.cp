// expect: E0303 at 6
// rule: 02.6.6
// `@get` drops parentheses only when every other parameter has a default; `p.add` still needs `k`.
struct P:
    x: Int
    @get fn add(self, k: Int) -> Int = self.x + k
let p = P(x: 1)
print(p.add)
