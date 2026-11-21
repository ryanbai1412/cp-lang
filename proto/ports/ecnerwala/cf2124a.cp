@!multitest
let n: Int = read()
let a: [0..n]Int = read()
for (x, y) in a.adjacent if x > y:
    print("YES")
    print(2)
    print(x, y)
    return
print("NO")
