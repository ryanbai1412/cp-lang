// error: the `for` loop is going over `xs`
var xs: List<Int> = List<Int>()
xs.push(1)
xs.push(2)
for x in xs:
    if xs.len < 1000: xs.push(x)
print(xs.len)
