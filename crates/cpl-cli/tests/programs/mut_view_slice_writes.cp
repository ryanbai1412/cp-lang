// A `mut` slice in a local writes through to its list: cells, compound
// writes, reads, loops and prints through it, then a slice of the slice.
fn edit(n: Int) -> List<Int>:
    var a = [n].filled(0)
    for i in 0..n: a[i] = i * 10
    let s = mut a[1..n - 1]
    s[0] = 7
    s[1] += 5
    for k in s.keys: s[k] += 1
    var sum = 0
    for v in s: sum += v
    print(s, s.len, sum, s[2])
    let t = mut s[1..3]
    t[1] = -1
    t.reverse()
    print(t, t.len)
    a
print(edit(6))
