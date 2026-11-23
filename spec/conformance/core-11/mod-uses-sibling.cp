// expect: ok, output "2"
// rule: 11.2.2
mod a:
    pub fn f() -> Int = 1
mod b:
    use a.f
    pub fn g() -> Int = f() + 1
print(b.g())
