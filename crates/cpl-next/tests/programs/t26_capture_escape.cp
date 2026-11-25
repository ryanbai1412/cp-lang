// expect: E0603
// input: "3\n"
// A closure that shares a `var` cannot be returned: the variable is gone by then.
fn total<D>(a: [D]Int) -> Int:
    var s = 0
    for v in D: s += a[v]
    s
fn go(n: Int) -> List<Int -> Int>:
    var out: List<Int -> Int> = []
    if n == 0: return out
    domain V = 0..n
    var a: [V]Int = 1
    out = go(n - 1)
    // V here is still the outer one, of size n.
    var b: [V]Int = 2
    print("{n} {V.count} {total(b)}")
    out.push(x => x + V.count * 100 + total(a))  // <- here
    out
let n: Int = read()
let fs = go(n)
for i in 0..fs.len: print(fs[i](0))
