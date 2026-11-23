// expect: ok, output "1 3"
// rule: 11.1.4
// `pub` fields and methods are visible outside their module.
mod m:
    pub struct P:
        pub x: Int
        y: Int
        pub fn sum(self) -> Int = self.x + self.y
        fn secret(self) -> Int = self.y
    pub fn make() -> P = P(x: 1, y: 2)
    pub fn peek(p: P) -> Int = p.y + p.secret()
let p = m.make()
print(p.x, p.sum())
