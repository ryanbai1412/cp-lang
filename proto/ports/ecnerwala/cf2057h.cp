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
            if zeros.is_empty:
                carried += i * (sent / (i + 1))
                sent %= i + 1
                zeros.push(-1)
            let d = i - zeros.last!
            if sent >= d:
                sent -= d
                carried += d - 1
                zeros.pop()
            else:
                zeros[zeros.len - 1] += sent
                carried += sent
                sent = 0
        into.push(carried)
    into

let left = inflow(a)
let right = inflow(a.rev)
print([a[i] + left[i] + right[n - 1 - i] for i in 0..n])
