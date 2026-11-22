// expect: ok, output "YES"
struct Box<T, F = Int>:
    x: ?T
    pub fn empty() -> Box<T, F> = Box<T, F>(None)
    pub fn same(self) -> Box<T, F> = self
let t = Box<Int, Int>(None)
let u: Box<Int, Int> = t
print(yes(u.x == None))
