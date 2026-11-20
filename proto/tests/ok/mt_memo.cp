// cp: 0.1 8ef64b9+dirty
let scale: 1..=3 = read()             // shared across tests

@multitest
fn solve():
    let n: 1..=6 = read()
    domain V = 1..=n
    let a: [V]0..=9 = read()

    @memo
    fn suffix(i: V) -> Int:
        if let j = i.next:
            return a[i] * scale + suffix(j)
        a[i] * scale

    print(suffix(V.first))
