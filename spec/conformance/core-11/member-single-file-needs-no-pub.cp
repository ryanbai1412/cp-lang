// expect: ok, output "3"
// rule: 11.1.4
// A single-file program is one module, so its members need no `pub`.
struct P:
    x: Int
    fn twice(self) -> Int = self.x * 2 + 1
print(P(x: 1).twice())
