// expect: ok, output "120 6"
// rule: 05.10.10
// An in-place method call on a `@cache` field (`push`) is a cache write too.
struct Fact:
    @cache f: List<Int>
    fn get(self, n: Int) -> Int:
        while f.len <= n: f.push(f[f.len - 1] * f.len)
        f[n]
let c = Fact(f: [1])
print(c.get(5), c.f.len)
