// cp: 0.1 a6dfa18+dirty
let n: 1..=1e18 = read()

@memo
fn steps(x: 1..=1e18) -> Int:
    if x == 1: return 0
    if x % 2 == 0: return 1 + steps(x / 2)
    1 + min(steps(x - 1), steps(x + 1))

print(steps(n))
