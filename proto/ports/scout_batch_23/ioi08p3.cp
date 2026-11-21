// IOI '08 P3 - Fish (DMOJ: ioi08p3, 35pt, TL 1.4s, ML 64MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// F fish, each with length L_i and gem type T_i in 1..K. A chosen fish of length X
// can eat any subset of fish with length <= X / 2. Count distinct non-empty multisets
// of gems obtainable, modulo a runtime (possibly composite) modulus M <= 30000.
//
// Sort gem types by maximum fish length L_max[1] <= ... <= L_max[K]. Attribute each
// valid multiset S to the LARGEST gem type t that can act as eater:
//   - Case A (1 <= S[t] <= count[t]): no type u > t may appear, so
//       count[t] * prod_{u < t} (count[u] + 1)
//   - Case B (S[t] == count[t] + 1): types u in (t, r(t)] may appear freely, where
//       r(t) is the last type with L_max[u] < 2 * len_of_(count[t]+1)-th_fish_of_t,
//       and types u > r(t) must be absent, so
//       prod_{u < t} (count[u] + 1) * prod_{t < u <= r(t)} (count[u] + 1).
// Because M can be composite, we cannot divide out factors: use a product SegTree.
use std.mod.{ModR}
use std.algebra.{Monoid}
use std.segtree.{SegTree}

let f_cnt: 1..=5e5 = read()
let k: 1..=5e5 = read()
let m_val: 1..=30000 = read()
// WISH MG2: ModR is an operation struct, not a value type with `+ - *` operators;
// WISH MG3: Monoid<Int> capturing runtime `mod` collides with static monoid specialization!
let mod = ModR(m_val)
let mul_m = Monoid<Int>(1 % m_val, (a, b) => mod.mul(a, b))

domain Fish = 0..f_cnt
domain RawGem = 1..=k
// WISH MG11: surjection constraint — every gem type 1..=k appears on at least one fish
let raw: [Fish](len: 1..=1e9, gem: RawGem) = read()

var max_len: [RawGem]Int = 0
for i in Fish: max_len[raw[i].gem] max= raw[i].len

let order = sorted([g for g in RawGem], by: g => max_len[g])
domain Gem = 0..k
var rank_of: [RawGem]Gem = Gem(0)
for idx in Gem: rank_of[order[idx.to_idx]] = idx

var by_gem: [Gem]List<Int> = []
var all_fish: List<(len: Int, gem: Gem)> = []
for i in Fish:
    let g = rank_of[raw[i].gem]
    by_gem[g].push(raw[i].len)
    all_fish.push((len: raw[i].len, gem: g))
for g in Gem: by_gem[g] = by_gem[g].sorted()
all_fish = all_fish.sorted(by: x => x.len)

let gem_max = [Gem] g => by_gem[g][by_gem[g].len - 1]
var eaten: [Gem]Int = 0
var st = SegTree<Gem, Int>.from(mul_m, [Gem] _ => 1 % m_val)

var ptr = 0
var ans = 0
for t in Gem:
    let lim = gem_max[t]
    while ptr < f_cnt and all_fish[ptr].len * 2 <= lim:
        let g = all_fish[ptr].gem
        eaten[g] += 1
        st.set(g, (eaten[g] + 1) % m_val)
        ptr += 1

    let pref = st.fold(Gem.first..t)
    let case_a = mod.mul(eaten[t] % m_val, pref)

    // Smallest fish of type t that t itself cannot eat (strictly longer than lim / 2)
    let next_len = by_gem[t][eaten[t]]
    // Largest u >= t with gem_max[u] < 2 * next_len
    let cut = (t.to_idx..k).first_where(u => gem_max[Gem(u)] >= 2 * next_len) ?? k
    let mid = if t.to_idx + 1 < cut: st.fold(Gem(t.to_idx + 1)..Gem(cut)) else: 1 % m_val
    let case_b = mod.mul(pref, mid)

    ans = mod.add(ans, mod.add(case_a, case_b))

print(ans)
