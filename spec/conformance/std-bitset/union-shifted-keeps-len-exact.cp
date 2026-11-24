// expect: ok, output "YES"
// union_shifted is `|= o.shifted(k)` in place, and len, count and is_empty stay exact after
// word-parallel ops drop the kept count; checked against a [D]Bool model.
domain D = 0..150
var ok = true
for trial in 0..200:
    var a = Bitset<D>.from([D] _ => rand(0, 3) == 0)
    let o = Bitset<D>.from([D] _ => rand(0, 2) == 0)
    let k = rand(-160, 160)
    var want: [D]Bool = [D] d => a.contains(d)
    for d in o.each():
        if let e = D.at(d.ord + k): want[e] = true
    a.union_shifted(o, k)
    if a.each().collect() != [d for d in D if want[d]]: ok = false
    let c = [d for d in D if want[d]].len
    if a.len != c or a.count != c or a.is_empty != (c == 0): ok = false
    a.toggle(D(rand(0, 149)))
    if a.len != a.each().collect().len: ok = false
    if a.complement().len != D.count - a.len: ok = false
    var e = Bitset<D>()
    e.union_shifted(o, 200)
    if not e.is_empty or e.len != 0: ok = false
print(yes(ok))
