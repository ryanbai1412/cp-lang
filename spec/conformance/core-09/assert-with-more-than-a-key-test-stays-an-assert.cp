// expect: R0401 (checked)
// rule: 09.3.10
fn get(a: List<Int>, i: Int) -> Int:
    assert i in a.keys and i > 0 // <- here
    a[i]
print(get([1, 2], 5))
