// AtCoder DP K Stones (proto/ports/dpk.cp), with a planted off-by-one: a move needs strictly more
// stones than it takes, so taking the whole pile is never tried.
// class: off-by-one
let n: 1..=100 = read()
let k: 1..=1e5 = read()
let moves: [0..n]1..=1e5 = read()

fn first_wins(k: Int, moves: List<Int>) -> Bool:
    var wins = List.filled(k + 1, false)
    for p in 0..=k:
        for a in moves:
            if a < p and not wins[p - a]: wins[p] = true  // planted
    wins[k]

@brute(first_wins)
fn first_wins_slow(k: Int, moves: List<Int>) -> Bool =
    moves.any(a => a <= k and not first_wins_slow(k - a, moves))

print(if first_wins(k, moves): "First" else: "Second")
