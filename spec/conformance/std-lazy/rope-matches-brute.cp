// expect: ok, output "1 2 3\n1 12 13\n7 7 3\n3\n3\n1 9 3\n2 1 3\nYES\nYES"
use std.lazy.{Rope}
domain U = 0..3
let a0: [U]Int = [1, 2, 3]
var s1 = Rope(a0)
print(s1.to_array())
var s2 = Rope(a0, merge: (t, x) => t + x)
s2.apply(U(1)..=U(2), 10)
print(s2.to_array())
var s3 = Rope(a0)
s3.apply(U(0)..U(2), 7)
print(s3.to_array())
var s4 = Rope(a0)
s4.reverse(U(0)..=U(2))
print(s4.get(U(0)))
var s5 = Rope(a0)
s5.reverse(U(1)..=U(2))
print(s5.get(U(1)))
var s6 = Rope(a0)
s6.set(U(1), 9)
print(s6.to_array())
var s7 = Rope(a0)
s7.reverse(U(0)..U(2))
print(s7.to_array())
domain V = 0..40
var seed = 7
fn rnd(mut seed: Int, k: Int) -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    (seed / 65536) % k
var a: [V]Int = 0
for i in V: a[i] = rnd(mut seed, 100)
var ref = [a[i] for i in V]
var ro = Rope(a, merge: (t, x) => t + x)
var ok = true
for step in 0..2000:
    let op = rnd(mut seed, 4)
    let l = rnd(mut seed, 40)
    let h = l + rnd(mut seed, 40 - l)
    if op == 0:
        ro.reverse(V(l)..=V(h))
        var (i, j) = (l, h)
        while i < j:
            let t = ref[i]
            ref[i] = ref[j]
            ref[j] = t
            i += 1
            j -= 1
    elif op == 1:
        let d = rnd(mut seed, 21) - 10
        ro.apply(V(l)..=V(h), d)
        for k in l..=h: ref[k] += d
    elif op == 2:
        let x = rnd(mut seed, 100)
        ro.set(V(l), x)
        ref[l] = x
    else:
        if ro.get(V(l)) != ref[l]: ok = false
let out = ro.to_array()
for i in V:
    if out[i] != ref[i.ord]: ok = false
print(yes(ok))
var ov = Rope(a)
ov.apply(V(3)..=V(10), 5)
ov.reverse(V(0)..=V(20))
ov.apply(V(0)..=V(4), 1)
var r2 = [a[i] for i in V]
for k in 3..=10: r2[k] = 5
var (i2, j2) = (0, 20)
while i2 < j2:
    let t = r2[i2]
    r2[i2] = r2[j2]
    r2[j2] = t
    i2 += 1
    j2 -= 1
for k in 0..=4: r2[k] = 1
print(yes([ov.get(i) for i in V] == r2))
