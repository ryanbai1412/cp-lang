// CF 1846E1 Rudolf and Snowflakes (simple version)
// in: t; per test n. out: YES if n = 1 + k + k^2 + ... + k^p for some k >= 2, p >= 2, else NO.
var snowflake: [0..=1e6]Bool = false
for k in 2..=1000:
    var layer = k * k
    var total = 1 + k + layer
    while total <= 1e6:
        snowflake[total] = true
        layer *= k
        total += layer

@!multitest
let n = read<1..=1e6>()
print(yes(snowflake[n]))
