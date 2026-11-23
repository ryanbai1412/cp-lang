// expect: ok, output "YES NO"
// A variant of a generic enum compared with a value of that enum takes the value's type
// arguments, as it would from an annotation.
enum Node<I, N>: Leaf(i: I), Inner(l: N, r: N)
let x: Node<Int, Bool> = Node.Inner(true, false)
print(yes(x == Node.Inner(true, false)), yes(x == Node.Leaf(1)))
