// cp: 0.1 bea787c+dirty
@memo
fn dp(u: 0..=50, used: Bool) -> Int:
    if u == 0: return if used: 1 else: 0
    return dp(u - 1, used) + dp(u - 1, not used)
print(dp(5, true))
