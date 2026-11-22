// expect: E0206 at 2
// rule: 11.2.2
mod a:
    use b.g
    pub fn f() -> Int = 1
mod b:
    use a.f
    pub fn g() -> Int = 2
