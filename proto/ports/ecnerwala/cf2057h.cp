// Codeforces 2057 H: Coffee Break.
// Turning off machine `j` sends `floor(a[j] / 2)` students each way, and
// operations commute, so the maximum at `i` is `a[i]` plus the independent
// maximum inflow from the left prefix and right suffix (never turning `i` off).
// Pushing a prefix greedily toward `i` leaves every position at `1` except a
// stack of `0` positions: each student sent in shifts the nearest `0` one step
// toward `i` and carries one student forward until that `0` reaches `i` (where
// it is consumed, contributing `d - 1` carried for `d` students).
// When the stack is empty, all `i` positions `0..i` hold `1`; sending `i + 1`
// students completes one full lap (carrying `i` forward and leaving the stack
// empty again). Thus `laps = sent / (i + 1)` adds `i * laps` to `carried`, and
// any remainder `rest = sent % (i + 1) > 0` creates a new zero at `rest - 1`
// while carrying `rest` students forward.

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
