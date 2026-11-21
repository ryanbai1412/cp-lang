// Codeforces 2057 H: turning off machine j sends floor(a_j / 2) each way, and the operations commute,
// so the best for i is a_i plus what flows in from each side separately, with i never turned off.
// Pushing a prefix toward i leaves it all 1s but for a stack of 0 positions: each student sent in moves
// the nearest 0 one step toward i, and a 0 that reaches i is consumed. The stack of 0s is the whole state.
@!multitest
let a = read<[read(1..=1e6)]0..=1e9>()

fn inflow(a: [_]Int) -> [_]Int:
    var zeros = Stack<Int>()
    var carried = 0
    a.positions.map(i =>
        if i == a.first_position: return 0
        zeros.push(i - 1)
        var sent = carried + a[i - 1]
        carried = 0
        while sent > 0:
            match zeros.top:
                none:
                    let (laps, rest) = sent.divmod(i + 1)
                    carried += i * laps
                    sent = rest
                    zeros.push(-1)
                z if sent >= i - z:
                    sent -= i - z
                    carried += i - z - 1
                    zeros.pop()
                _:
                    zeros.top! += sent
                    carried += sent
                    sent = 0
        carried)

print(zip(a, inflow(a), inflow(a.rev).rev).map(sum))
