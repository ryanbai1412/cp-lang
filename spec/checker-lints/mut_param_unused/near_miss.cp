fn grow(mut xs: List<Int>) -> Int:
    xs.push(0)
    return xs.len

var a = [1, 2]
print(grow(mut a))
