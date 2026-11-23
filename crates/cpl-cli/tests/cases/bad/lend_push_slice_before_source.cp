// error: `.push(...)` would keep the lent slice
let n: 2..=9 = read()
domain V = 1..=n
fn keep():
    var parts: List<[V]Int> = []
    var a: [V]Int = read()
    parts.push(a[V(1)..=V(2)])
    print(parts.len)
keep()
