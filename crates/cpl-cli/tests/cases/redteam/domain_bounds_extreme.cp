// the length of a domain is computed without overflowing, and an inverted range is empty
let a: Int = read()
let b: Int = read()
domain V = a..=b
print(V.len)
let c: Int = read()
let d: Int = read()
domain W = c..=d
print(W.len)
for w in W: print(0)
