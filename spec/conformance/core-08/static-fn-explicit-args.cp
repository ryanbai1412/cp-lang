// expect: ok, output "YES"
// rule: 08.2.5
struct Box<T>:
    x: ?T
    pub fn empty() -> Box<T> = Box<T>(None)
let b = Box<Int>.empty()
let c: Box<Int> = b
print(yes(c.x == None))
