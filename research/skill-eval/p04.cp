let (n, t) = read<(1..=2e5, 1..=1e9)>()
let k = read<[n]1..=1e9>()

fn enough(time: Int) -> Bool:
    var made = 0
    for x in k:
        made += time / x
        if made >= t:
            return true
    false

print((1..=1_000_000_000_000_000_000).first_true(time => enough(time))!)
