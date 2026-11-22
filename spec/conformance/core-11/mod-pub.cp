// expect: ok, output "1"
// rule: 11.1.2 11.1.3
mod m:
    pub fn f() -> Int = 1
    fn g() -> Int = 2
print(m.f())
