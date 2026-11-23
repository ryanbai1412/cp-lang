let (n, l, r): (1..=100, 0..=99, 1..=100) = read()
var a: [0..n]Int = read()
var tmp: List<Int> = []  // expect: push_loop
for i in l..r:  // expect: copy_sort_writeback
    tmp.push(a[i])
tmp.sort()
for i in l..r:
    a[i] = tmp[i - l]
print(a)
