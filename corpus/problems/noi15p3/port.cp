// Source: NOI 2015 P3 (Sushi Dinner), https://dmoj.ca/problem/noi15p3
// limits: 1000 ms, 512 MB
// Problem: count pairs of subsets X, Y of {2..n} with every x in X coprime to every y in Y,
//   n <= 500, modulo a runtime p <= 1e9.
// Solution: every number up to 500 has at most one prime above 19. Give each of the 8 small
//   primes a side (X, Y or nobody; 3^8 ways); then each big prime's group (its multiples, or a
//   lone number without one) goes wholly to X, wholly to Y or nowhere, among the members the
//   small sides allow. A small prime used by nobody is counted three times, so
//   inclusion-exclusion with weight (-2)^|S| over primes forced to nobody fixes it. O(3^8 n).
// original: corpus/submissions/3782724.cpp

let (n, p): (2..=500, 1..=1e9) = read()
type Z = Mod<p>

domain Small = 0..8
let primes: [Small]Int = [2, 3, 5, 7, 11, 13, 17, 19]
type Mask = Subset<Small>
fn small(x: Int) -> Mask:
    var m = Mask.empty
    for i in Small:
        if x % primes[i] == 0: m = m.with(i)
    m

let sieve = Sieve(n)
fn big(x: Int) -> ?Int = sieve.factor(x).map(f => f.0).find(q => q > 19)
// per big prime its multiples, and every other number alone: the small primes of each member
var groups: [2..=n]List<Mask> = []
for x in 2..=n:
    if let q = big(x): groups[q].push(small(x))
    else: groups[x].push(small(x))
let pow2 = [Z.new(2) ** i for i in 0..=n]

// Give each small prime to X only (xs), Y only (ys) or nobody. A group then goes wholly to X
// (2^a choices among its a members that fit X), wholly to Y, or nowhere: 2^a + 2^b - 1.
// Ways are summed by the set of small primes left to nobody.
var by_none: [Mask]Z = 0
for xs in Mask:
    for ys in xs.complement().subsets:
        var way = Z.new(1)
        for g in groups if not g.is_empty:
            var (fit_x, fit_y) = (0, 0)
            for s in g:
                if (s - xs).is_empty: fit_x += 1
                if (s - ys).is_empty: fit_y += 1
            way *= pow2[fit_x] + pow2[fit_y] - Z.new(1)
        by_none[(xs | ys).complement()] += way

// A prime nobody uses is counted under all three of its choices; inclusion-exclusion with weight
// (-2)^|S| over primes forced to nobody keeps each pair once.
var total = Z.new(0)
for s in Mask:
    var forced = Z.new(0)
    for t in s.supersets: forced += by_none[t]
    total += forced * (-Z.new(2)) ** s.len
print(total)
