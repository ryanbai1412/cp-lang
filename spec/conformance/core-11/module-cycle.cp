// expect: E0206
// rule: 11.2.2
mod a:
    use b.g // <- here
    pub fn f() -> Int = 1
mod b:
    use a.f
    pub fn g() -> Int = 2
