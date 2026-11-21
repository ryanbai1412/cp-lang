let n: 1..=9 = read()
domain I = 1..=n
type B = I.cuts
let a: [I]Int = read()
for i in I:
    if let j = i - 2 and let k = i + 2: print(i, j, k) else: print(i, "edge")
    if let j = i + 1 and a[j] > 3: print("big next", j)
    print(a[i + 4 ?? I.last], a[(i - 1) ?? i], sum(a[(..=i).suffix(3)]), (i..).prefix(2).len)
for b in B:
    if let x = b.left and let y = b.right: print(b, x, y)
    elif let x = b.left: print(b, "last", x)
    else: print(b, "first", b.right!)
print((I.first + 1)!)
print(a[I.first.offset(2)!], a[I.last.offset(-1)!])     // `.offset(d)` is the named form of `+ d`
