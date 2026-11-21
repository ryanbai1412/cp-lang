enum Node: leaf, incr(a: Int, b: Int)
let x = Node.leaf
match x:
    incr(a, b) => print(a + b)
    _ => print(0)
