// expect: ok, output "3 2"
// rule: 02.9.2
// A domain a fn body declares is a type in that body: its arrays and positions name it.
fn f(n: Int) -> (Int, Int):
    domain Order = 0..n
    var t: [Order]Int = 0
    let o: Order = Order.last!
    (t.len, o.ord)
let (a, b) = f(3)
print(a, b)
