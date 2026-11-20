// cp: 0.1 5df8551
let n: 1..=1e7 = read()
type Key = 0..=999999
var s = Set<Key>()
var x = 12345
for i in 0..n:
    x = (x * 1103515245 + 12345) % 2147483648
    s.insert(x % 1000000)
print(s.len)
