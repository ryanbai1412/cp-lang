// expect: ok, output "71\n7\n7"
// A nested fn that writes a cell of its caller's list, whether the cell is a struct, a list
// or is pushed to, writes the caller's list: the cell's words never land in a register the
// lending still uses.
struct P:
    var a: Int
    var b: Int
fn structs(n: Int) -> Int:
    var adj = [P(0, 0) for _ in 0..n]
    fn go():
        adj[1] = P(7, 1)
    go()
    adj[1].a * 10 + adj[1].b
fn lists(n: Int) -> Int:
    var adj = [List<Int>() for _ in 0..n]
    fn go():
        adj[1] = [7]
    go()
    adj[1][0]
fn pushes(n: Int) -> Int:
    var adj = [List<Int>() for _ in 0..n]
    fn go():
        adj[1].push(7)
    go()
    adj[1][0]
print(structs(3))
print(lists(3))
print(pushes(3))
