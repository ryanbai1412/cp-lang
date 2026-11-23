// expect: E0207 at 10
// rule: 11.1.4
// A method without `pub` is private to its module; the fix adds `pub`.
mod m:
    pub struct P:
        pub x: Int
        y: Int
        pub fn sum(self) -> Int = self.x + self.y
        fn secret(self) -> Int = self.y
    pub fn make() -> P = P(x: 1, y: 2)
    pub fn peek(p: P) -> Int = p.y + p.secret()
let p = m.make()
print(p.secret())
