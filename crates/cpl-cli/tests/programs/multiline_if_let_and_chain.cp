// Indented multi-line `if`/`elif`/`else` expressions, `let x =\n    <expr>`,
// and indented `.method(...)` chains.
fn classify(n: Int) -> Int:
    let tag = if n < 0: -1
        elif n == 0: 0
        else: 1
    tag

let picked =
    if classify(5) == 1:
        let xs = [1, 2, 3, 4, 5]
            .filter(x => x % 2 == 1)
            .map(x => x * 10)
        xs.sum()
    else:
        0

print(classify(-3), classify(0), classify(7), picked)
