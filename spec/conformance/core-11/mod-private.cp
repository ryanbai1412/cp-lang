// expect: E0207 at 4
// rule: 11.1.2 11.1.3
mod m:
    pub fn f() -> Int = 1
    fn g() -> Int = 2
print(m.g())
