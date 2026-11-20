// A word read in a loop of a `@multitest` fn: the input ends where the loop ends, so the `print` after it runs once.
@multitest
fn solve():
    let n: 1..=5 = read()
    var total = 0
    for _ in 0..n:
        let word: [domain W]Char = read()
        total += W.len
    print(total)
