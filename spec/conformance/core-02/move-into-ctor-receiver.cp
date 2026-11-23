// expect: ok, output "4"
// rule: 02.1.2
// A list moved into a constructor that is a method's receiver (`B(n, xs).m()`) moves once.
struct B(n: Int, d: List<Int>):
    fn m(self) -> Int = self.n + self.d.len
var xs = List<Int>()
xs.push(7)
print(B(3, xs).m())
