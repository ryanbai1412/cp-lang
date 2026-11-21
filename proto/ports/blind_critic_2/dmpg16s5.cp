// DMPG '16 S5 Pizza Bag: n slices in a circle, take at most k adjacent ones (possibly none)
// for the largest total.
// Sliding-window minimum of prefix sums over the circle unrolled twice.
let n: 1..=1e5 = read()
let k: 1..=n = read()
let a: [n]-1e9..=1e9 = read()

// pre[e] = sum of the first e slices of the circle unrolled twice
var pre: [0..=2 * n]Int = 0
for e in 1..=2 * n: pre[e] = pre[e - 1] + a[(e - 1) % n]

@solution
fn main():
    var best = 0
    var window: List<Int> = []      // candidate starts, pre increasing
    var head = 0
    for e in 1..=2 * n:
        while head < window.len and pre[window.last!] >= pre[e - 1]: window.pop()
        window.push(e - 1)
        while window[head] < e - k: head += 1
        best max= pre[e] - pre[window[head]]
    print(best)

@brute(main)
fn slow():
    var best = 0
    for e in 1..=2 * n:
        for s in max(0, e - k)..e:
            best max= pre[e] - pre[s]
    print(best)
