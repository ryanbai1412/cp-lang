// expect: R0101 (checked)
// rule: 09.3.10
fn get(a: List<Int>, i: Int) -> Int:
    assert i in a.keys // <- here
    a[i]
print(get([1, 2], 5))
