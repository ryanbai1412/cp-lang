fn key(x: Int) -> Int:
    100 / x

var xs = [4, 2]
xs.sort(by: x => key(x))
print("sorted", xs)
var zs = [0]
zs.sort(by: x => key(x))
print("not reached", zs)
