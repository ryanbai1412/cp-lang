// expect: ok, output "3 2"
// rule: decision 34
// A closure keeps the domain size of the call that made it (t14 neighbour), called after a deeper call made another.
fn make(n: Int) -> Int:
    domain V = 0..n
    let size = () => V.count
    let inner = if n > 2: make(n - 1) else: 0
    size() * 1 + inner * 0
print(make(3), make(2))
