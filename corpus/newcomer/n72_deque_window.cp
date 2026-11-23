let (n, k): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
var dq: List<Int> = []
var head = 0  // expect: sliding_min_by_hand
var out: List<Int> = []
for i in 0..n:
    while dq.len > head and a[dq[dq.len - 1]] >= a[i]:
        dq.pop()
    dq.push(i)
    if dq[head] <= i - k:
        head += 1
    if i >= k - 1:
        out.push(a[dq[head]])  // expect: sliding_min_by_hand
print(out)
