// expect: ok, output "16 16"
// rule: 05.10.9 05.10.12
// The cache hands out plain copies, and the fn parameter runs before any lend into the field.
struct Sq:
    n: Int
    @cache memo: ?Int
    fn get(self, f: Int -> Int) -> Int:
        let v = self.memo ?? f(self.n)
        self.memo = Some(v)
        v
let s = Sq(n: 4, memo: None)
print(s.get(x => x * x), s.get(x => x * x))
