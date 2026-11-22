// expect: ok, output "3 7"
// rule: 8.1
// Method calls with explicit generic type arguments `r.method<T>()` parse without `::`.
struct Boxed(var count: Int):
    fn bump<D>(mut self, v: D) -> Int:
        self.count += 1
        Int(v) + self.count
domain V = 1..=5
var b = Boxed(2)
let x = b.bump<V>(V(1))
print(b.count, x + 3)
