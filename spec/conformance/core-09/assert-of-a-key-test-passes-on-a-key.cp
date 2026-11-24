// expect: ok, output "2"
// rule: 09.3.10
fn get(a: List<Int>, i: Int) -> Int:
    assert i in a.keys
    a[i]
print(get([1, 2], 1))
