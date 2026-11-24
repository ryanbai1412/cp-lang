// expect: E0606
// rule: 10.3.7
// `s[k]` reads through a struct's `get`, so a `get -> ?T` gives `?T`, which does not print.
struct Sparse<T>:
    _xs: List<T>
    pub fn get(self, i: Int) -> ?T = _xs.get(i)
let s = Sparse([3])
print(s[0])  // <- here
