mod a:
    pub fn f() -> Int = 1
    pub fn h() -> Int = 10
mod b:
    use a.{f, h}
    pub fn g() -> Int = f() + h()
mod c:
    use a.*
    pub fn k() -> Int = h() * 2
print(b.g(), c.k())
