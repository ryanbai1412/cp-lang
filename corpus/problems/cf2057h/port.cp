// Source: Codeforces 2057H (Coffee Break), https://codeforces.com/contest/2057/problem/H
// Problem: rooms 1..n hold a[i] <= 1e9 students. Turning off room i's light sends floor(x/2) of
//   its x students to each neighbour (rooms 0 and n+1 absorb them). For every i print the most
//   students that can gather in room i. n, sum n <= 1e6; multitest.
// Solution: operations commute and room i is never switched off, so its best is a[i] plus the
//   most that can flow in from the prefix and, mirrored, from the suffix. Pushing a prefix
//   greedily right leaves 1s except a stack of 0 positions: each arriving student moves the
//   nearest 0 one step on, carrying one student, until it reaches i; with no 0s, i + 1 students
//   make a full lap. `inflow` keeps that stack and counts laps by division: O(n) amortised.

@!multitest
let n: 1..=1_000_000 = read()
let a: [n]0..=1e9 = read()

fn inflow(a: List<Int>) -> List<Int>:
    var zeros: List<Int> = []
    var carried = 0
    var into = [0]
    for i in 1..a.len:
        zeros.push(i - 1)
        var sent = carried + a[i - 1]
        carried = 0
        while sent > 0:
            if let z = zeros.pop():
                let d = i - z
                if sent >= d:
                    sent -= d
                    carried += d - 1
                else:
                    zeros.push(z + sent)
                    carried += sent
                    sent = 0
            else:
                let laps = sent / (i + 1)
                let rest = sent % (i + 1)
                carried += i * laps + rest
                if rest > 0: zeros.push(rest - 1)
                sent = 0
        into.push(carried)
    into

let left = inflow(a)
let right = inflow(a.rev)
print([a[i] + left[i] + right[n - 1 - i] for i in 0..n])
