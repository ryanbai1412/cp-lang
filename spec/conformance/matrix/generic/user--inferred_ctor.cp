// expect: ok, output "YES"
struct Box<T, F = Int>:
    x: ?T
    pub fn empty() -> Box<T, F> = Box<T, F>(None)
    pub fn same(self) -> Box<T, F> = self
let u: Box<Int, Int> = Box(None)
print(yes(u.x == None))
