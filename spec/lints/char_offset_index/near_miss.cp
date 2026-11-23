let s = "abca"
let a = [1, 2, 3]
for c in s:
    print(c - 'a', a[c.ord % 3])
