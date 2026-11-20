let n: 1..=20 = read()

// Small linear recurrence (see memo_dp.cp for cached @memo DP).
fn f(i: Int) -> Int:
    if i == 0:
        return 1
    f(i - 1) * 3 + i

print(f(n - 1))
