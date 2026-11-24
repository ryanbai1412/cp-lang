let n: 1..=10 = read()
var m: Map<1..=10 ** 6, Int> = Map()
fn show_all(m: Map<1..=10 ** 6, Int>):
    print(m)
    print(m.len)
for i in 0..n: m[n - i] += i * 1_000_000_000
show_all(m)
print(m)
