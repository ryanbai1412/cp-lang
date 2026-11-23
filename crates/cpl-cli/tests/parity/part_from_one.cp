var a: [1..=4]Int = [5, 2, 9, 5]
print(a[2..=4], sum(a[2..=4]), a[2..=4].argmax ?? -1, a[1..3].argmin ?? -1, a[3..=4].max()!)
a[2..=3] += 10
a[3..=4] = a[1..=2] + 1
a[1..=2] max= a[3..=4]
print(a, a[..], a[2..], a[..=2])
var one: [1..=2, 1..=3]Int = 7
one[2, 3] = 9
print(one[.., 3].argmax ?? -1, one[2, ..].argmax ?? -1, one[2, 2..=3].argmax ?? -1, sum(one[1..=2, 2]))
one[1, 2..=3] += one[2, 2..=3]
one[.., 1] = 0
print(one[1, ..], one[.., 1], one[2, ..])
fn last_index<D, T>(xs: [D]T) -> ?D:
    var at: ?D = None
    for i in D: at = Some(i)
    at
fn total<D>(xs: [D]Int) -> Int = sum(xs[i] for i in D)
print(last_index(a[..]) ?? -1, last_index(a[2..=4]) ?? -1, total(a[2..=4]), a[2..=4].argmax ?? -1)
print(last_index(one[.., 1]) ?? -1, last_index(one[1, ..]) ?? -1, total(one[2, 2..=3]))
var b: [3..7]Int = [1, 2, 3, 4]
b[4..6] *= 5
b[5..7] = b[3..5] + 1
print(b, b[4..7], b[3..3], b[6..], b[..5].argmax ?? -1, b[..].argmin ?? -1)
let (l, r): (Int, Int) = read()
print(a[l..=r], b[l + 2..r + 3], one[2, l..r])
a[l..=r] min= 3
one[l - 1..=r - 1, 2] = 1
print(a, one[.., 2])
