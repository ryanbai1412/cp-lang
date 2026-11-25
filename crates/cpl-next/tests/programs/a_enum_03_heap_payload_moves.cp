// expect: ok, output "3\n0"
// rule: R02.7.1, R05.5
enum Bag:
    Full(xs: List<Int>)
    Empty
fn size(b: Bag) -> Int = match b:
    Full(xs) => xs.len
    Empty => 0
let xs = [1, 2, 3]
print(size(Bag.Full(xs)))
print(size(Bag.Empty))
