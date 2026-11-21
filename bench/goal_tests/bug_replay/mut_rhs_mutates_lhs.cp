fn pop_idx(mut a: List<Int>) -> Int:
    let _ = a.pop()
    0
var a: List<Int> = [1, 2, 3]
a[0] = pop_idx(mut a)
print(a[0])
