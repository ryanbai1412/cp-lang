// Codeforces 2137 C — Maximum Even Sum
//
// Transferring a divisor `k | b` replaces `(a, b)` with `(a * k, b / k)` of
// product `P = a * b <= 10^18` (guaranteed by the statement, so `a * b` fits
// in 64-bit `Int`). Since `x + P / x` is convex in `x`, maximizing an even sum
// picks the largest `k | b` making `a * k` and `b / k` have the same parity:
// - `a` odd, `b` odd: every `k` is odd, so `a * k` and `b / k` are both odd
//   (sum always even); `k = b` maximizes the sum at `a * b + 1`.
// - `b` even and `4 | a * b`: leaving a single factor of 2 in `b / k = 2`
//   (`k = b / 2`) makes both `a * b / 2` and `2` even, maximizing the sum at
//   `a * b / 2 + 2`.
// - Otherwise (`a` even and `b` odd, or `a` odd and `b == 2 (mod 4)`): one of
//   `a * k`, `b / k` is always odd and the other even for every `k | b`, so no
//   even sum exists (`-1`).

@!multitest
let (a, b): (1..=10 ** 18, 1..=10 ** 18) = read()
let ans = if a % 2 == 1 and b % 2 == 1: a * b + 1
elif b % 2 == 0 and (a * b) % 4 == 0: a * b / 2 + 2
else: -1
print(ans)
