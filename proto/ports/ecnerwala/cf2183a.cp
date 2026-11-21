// Codeforces 2183 A: Bob wins if and only if both ends of the binary array are 0.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(if a.first! == 0 and a.last! == 0: "Bob" else: "Alice")
