// expect: ok, output "2 -1"
// rule: 01.6.2
domain V = 0..3
fn count_of(xs: ?List<V>) -> Int:
    let ys = xs else: return -1
    ys.len
print(count_of([V(0), V(2)]), count_of(None))
