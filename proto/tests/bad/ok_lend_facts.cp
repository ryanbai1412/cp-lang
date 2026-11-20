let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
fn go(i: V, j: V):
    if i == j: return
    swap(mut a[i], mut a[j])          // `if i == j: return` leaves i != j
fn go2(i: V, j: V):
    assert i < j
    swap(mut a[i], mut a[j])          // i < j proves i != j
go(l, r)
go2(l, r)
if l != r:
    swap(mut a[l], mut a[r])
else:
    a[l] += 1
let part = a[l..=r]
print(sum(part))
a[l] = 0                              // fine: `part` was last used on the line above
print(a)
