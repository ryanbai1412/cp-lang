@!multitest
let n: Int = read()
let a: List<Int> = [read::<Int>() for _ in 0..n]
for i in 0..(n - 1):
    if a[i] > a[i + 1]:
        print("YES")
        print(2)
        print(a[i], a[i + 1])
        return
print("NO")
