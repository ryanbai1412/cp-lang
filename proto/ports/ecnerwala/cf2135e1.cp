// Codeforces 2135 E1: Beyond the Palindrome (easy version).
// Count valid ±1 walks of length n by their final height x = 2k - n (k in 0..=n
// is the number of +1 steps), then weight each height by C(n, k).
// By the reflection principle across two barriers separated by `delta`, each
// odd reflection w * delta subtracts/adds a point mass proportional to the span
// width (`span_len`), and each even reflection (w + 1) * delta adds/subtracts a
// flat interval of half-width `half_span` (`delta - 2` when delta ≡ n (mod 2),
// `delta - 3` when delta ≢ n (mod 2)).

type Z = Mod<998_244_353>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
domain Step = 0..=n

var diff = Diff<Step, Z>()

fn add_height_range(lo_x: Int, hi_x: Int, v: Z):
    if lo_x > n or hi_x < -n: return
    let lo = Step((max(lo_x, -n) + n) / 2)
    let hi = Step((min(hi_x, n) + n) / 2)
    diff.add(lo..=hi, v)

fn add_height(x: Int, v: Z): add_height_range(x, x, v)

add_height_range(-n, n, 1)

// Same-parity barriers (delta ≡ n mod 2): single center at w * delta.
for delta in (2 + n % 2..=n + 2).step(2):
    let half_span = delta - 2
    let span_len = half_span + 1
    let mult = if delta == n + 2: 2 else: 4
    for w in (1..=n / delta).step(2):
        add_height(w * delta, -(mult * span_len))
        add_height_range((w + 1) * delta - half_span, (w + 1) * delta + half_span, mult)

// Opposite-parity barriers (delta ≢ n mod 2): split centers at w * delta ± 1.
for delta in (3 + n % 2..=n + 1).step(2):
    let half_span = delta - 3
    let span_len = half_span + 1
    for w in (1..=(n + 1) / delta).step(2):
        add_height(w * delta - 1, 2 * span_len)
        add_height(w * delta + 1, 2 * span_len)
        add_height_range((w + 1) * delta - half_span, (w + 1) * delta + half_span, -4)

let counts = diff.finish()
print([comb.choose(n, k.ord) * counts[k] for k in Step].sum())
