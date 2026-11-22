// error: say what a missing key means: `m[4] = (m.get(4) ?? start) / x`
var m: HashMap<Int, Int> = []
m[4] /= 2
print(m[4])
