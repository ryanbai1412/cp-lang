// Codeforces 2135 D1 — From the Unknown (Easy Version, interactive)
//
// Find hidden line width `W in 1..=WMAX` (`WMAX = 100_000`) in 2 queries of
// total length `<= 100_000`:
// - Query 1 sends `Q1 = 10_000` words of width `G = 125`.
//   - If `resp1 == 0`, then `W < G`. Query 2 sends `Q02 = 15_000 >= G * (G - 1)`
//     words of width `1`, making `resp2 = ceil(Q02 / W)` injective on `1..G`.
//   - Otherwise each line holds `floor(W / G)` words, so
//     `resp1 = ceil(Q1 / floor(W / G))` pins `W` to `[w_min, w_max]` with
//     `w_min = ceil(Q1 / resp1) * G` and
//     `w_max = min(WMAX, (Q1 - 1) / (resp1 - 1) * G + G - 1)` (width `<= 25_000`).
// - Query 2 sends pairs `(w_min, i - w_min)` for `i in w_min + 1..=w_max`
//   (`2 * (w_max - w_min) <= 50_000` words). Pair `i` fits on 1 line iff
//   `i <= W` and 2 lines iff `i > W`, so `resp2 = (w_max - w_min) + (w_max - W)`,
//   giving `W = (w_max - w_min) + w_max - resp2`.

@!interactive

const WMAX = 100_000
const G = 125
const Q1 = 10_000
const Q02 = 15_000

fn ask(vals: List<Int>) -> Int:
    print("?", vals.len, vals)
    read()

@!multitest
let resp1 = ask(List.filled(Q1, G))
if resp1 == 0:
    let resp2 = ask(List.filled(Q02, 1))
    print("!", (1..=Q02).find(ans => resp2 == ceil_div(Q02, ans))!)
    return

let w_min = ceil_div(Q1, resp1) * G
let w_max = min(WMAX, (Q1 - 1) / (resp1 - 1) * G + G - 1)
let q = [x for i in w_min + 1..=w_max for x in [w_min, i - w_min]]
let resp2 = ask(q)
print("!", (w_max - w_min) + w_max - resp2)
