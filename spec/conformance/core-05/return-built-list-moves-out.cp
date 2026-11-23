// expect: ok, output "0 1 2"
// rule: 05.5.1
fn make(n: Int) -> List<Int>:
    var xs = List<Int>()
    for i in 0..n:
        xs.push(i)
    return xs
print(make(3))
