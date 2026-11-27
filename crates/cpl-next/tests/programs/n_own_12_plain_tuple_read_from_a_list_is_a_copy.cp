// expect: ok, output "2 3"
// rule: R05.2.1
// A tuple of plain parts read out of a list is a copy that borrows nothing, so storing it in a
// longer-lived list is fine, inside a generic too and for range-typed elements.
fn walk<V>(from: List<V>) -> Int:
    var order = List<V>()
    var q = List<V>()
    for s in from: q.push(s)
    var head = 0
    while head < q.len:
        let u = q[head]
        head += 1
        order.push(u)
    order.len
var order = List<(Int, Int)>()
let q = [(0, 0), (1, 2)]
for i in 0..q.len:
    let u = q[i]
    order.push(u)
let r: List<0..=3> = [1, 2, 3]
print(order.len, walk(r))
