let n = 5
fn f(m: Int) -> Int:
    let k = m * 2
    k
print(f(n))
// A name inside an inline module is reached as `geo.k`, so it shadows nothing.
mod geo:
    pub fn k() -> Int = 4
let table: Map<Int, Int> = [3: 4, 5: 6]
print(geo.k() + table[3])
