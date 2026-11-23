// CF 1207E XOR Guessing: one query fixes the high 7 bits, the other the low 7 bits.
let wire = Wire()
let budget = 2
var used = 0
fn query(xs: List<Int>) -> Int:
    used += 1
    assert used <= budget, "query budget exceeded"
    print("?", xs)
    read()
let hi = query([i for i in 1..=100])
let lo = query([i << 7 for i in 1..=100])
print("!", (hi >> 7 << 7) | (lo & 127))
