// 1738C Even Number Addicts
// in: t; n; a_1..a_n. out: per test, "Alice" if Alice can force an even sum, else "Bob".
with io(answers: ("Alice", "Bob"))

@memo(array) fn alice_wins(odd: 0..=100, even: 0..=100, odd_sum: Bool, alice_moves: Bool) -> Bool:
    if odd == 0 and even == 0: return not odd_sum
    if alice_moves:
        (odd > 0 and alice_wins(odd - 1, even, not odd_sum, false))
            or (even > 0 and alice_wins(odd, even - 1, odd_sum, false))
    else:
        (odd == 0 or alice_wins(odd - 1, even, odd_sum, true))
            and (even == 0 or alice_wins(odd, even - 1, odd_sum, true))

@!multitest
let n = read<1..=100>()
let a = read<[n]-1e9..=1e9>()
let odd = a.count(x => x % 2 == 1)
print(yes(alice_wins(odd, n - odd, false, true)))
