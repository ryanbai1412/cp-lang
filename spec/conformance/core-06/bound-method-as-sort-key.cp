// expect: ok, output "1 2 0\n1 2 0\n2 0 1"
// rule: 06.1.8
// A bound method stands in for a lambda as a key, called once per element.
struct T:
    a: List<Int>
    fn key(self, x: Int) -> Int:
        return self.a[x]
let t = T(a: [5, 3, 4])
print([0, 1, 2].sorted(by: t.key))
var xs = [0, 1, 2]
xs.sort(by: t.key)
print(xs)
print([0, 1, 2].sorted(by: T(a: [4, 5, 3]).key))
