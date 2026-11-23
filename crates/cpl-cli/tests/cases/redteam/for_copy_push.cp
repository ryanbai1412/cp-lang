// iterating a copy while growing the original is the spelled-out way to do it
var xs: List<Int> = [1, 2, 3]
for x in xs.clone():
    xs.push(x * 10)
print(xs)
for i in 0..3:
    xs.push(xs[i])
print(xs.len)
