// Codeforces 1934B Yet Another Coin Problem
// in: t; per test n; out: per test, the minimum number of coins of values 1, 3, 6, 10, 15 summing to n
@!multitest
let n: 1..=1e9 = read()

fn coin_counts(n: Int) -> yields Int:
    for ones in 0..3:
        for threes in 0..2:
            for sixes in 0..5:
                for tens in 0..3:
                    let rest = n - ones - 3 * threes - 6 * sixes - 10 * tens
                    if rest >= 0 and rest % 15 == 0:
                        yield ones + threes + sixes + tens + rest / 15

print(coin_counts(n).min()!)
