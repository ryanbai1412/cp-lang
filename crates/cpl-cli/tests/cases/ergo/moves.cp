let n: 1..=9 = read()
domain I = 1..=n
type B = I.cuts
let a: [I]Int = read()
for i in I:
    if let j = i.prev(2) and let k = i.next(2): print(i, j, k) else: print(i, "edge")
    if let j = i.next and a[j] > 3: print("big next", j)
    print(a[i.next(4) ?? I.last!], a[i.prev ?? i], a[(i.prev(2) ?? I.first!)..=i].sum(), min((i..).len, 2))
for b in I.cuts:
    if let x = b.l and let y = b.r: print(b, x, y)
    elif let x = b.l: print(b, "last", x)
    else: print(b, "first", b.r!)
print(I.first!.next!)
print(a[I.first!.next(2)!], a[I.last!.prev!])     // `.next(k)` and `.prev(k)` step k positions
