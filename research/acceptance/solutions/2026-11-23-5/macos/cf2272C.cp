// C. Yet Another Black Cell Problem
// https://codeforces.com/contest/2272/problem/C
// judge: generic

use std.range.{first_true}

@!multitest
let (n, x): (2..=2e5, 0..=n) = read()
let a: [0..n]0..=1e9 = read()

fn needs(k: Int) -> Int:
    var lonely: List<Int> = []
    for i in 0..n:
        let near_left = i > 0 and a[i] - a[i - 1] <= k
        let near_right = i + 1 < n and a[i + 1] - a[i] <= k
        if not near_left and not near_right: lonely.push(i)
    var need = 0
    var j = 0
    while j < lonely.len:
        need += 1
        if j + 1 < lonely.len and lonely[j + 1] == lonely[j] + 1 and a[lonely[j + 1]] - a[lonely[j]] <= 2 * k:
            j += 2
        else:
            j += 1
    return need

print((1..=1_000_000_000).first_true(k => needs(k) <= x)!)
