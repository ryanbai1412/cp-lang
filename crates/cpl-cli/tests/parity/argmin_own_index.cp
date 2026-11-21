domain V = 0..3
let a: [1..=4]Int = [5, 2, 2, 5]
print(a.argmin, a.argmax, a[a.argmin!], a[a.argmax!])
let l = [3, 9, 2, 9, 2]
print(l.argmax, l.argmin, argmax(l), argmin(a), l[l.argmin!])
let w: [V]Int = [4, 8, 1]
let hi = w.argmax!
print(w[hi], w[w.argmin!], hi)
let e: List<Int> = []
print(e.argmax, e.argmin)
let fl = [1.5, 0.5, 2.5]
print(fl.argmax ?? -1, "zebra".argmax, [7].argmin)
let k: [5..9]Int = [4, 1, 9, 9]
print(k.argmax, k.argmin, k[k.argmax!])
