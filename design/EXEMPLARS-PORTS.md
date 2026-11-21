# Port exemplars: the gold standard

Every port is held to this bar. Five recent ports, each shown as the problem,
the accepted C++ (`main` only), the exemplar (how the solution should read in
CP Lang, using syntax and stdlib that may not exist yet), the port as it stood
when this was written, and what differs. The exemplar is also that port's
dream file (`proto/ports/ecnerwala/dream/NAME.cp`).

What the exemplars share, and what a reviewer checks for:

- The header comment states the editorial invariant in two to five lines; the
  code then reads as that argument, top to bottom.
- Standard algorithms and structures are named (`gomory_hu`, `runs`,
  `DualSegTree`, `MaxSub`), never re-derived inline.
- A structure with an invariant gets a name and its operations (`Staircase`,
  `Low` with `+`), instead of parallel arrays kept in sync by hand.
- Searches are `find`/`first`/`any`, not flag loops; undo mirrors recursion
  instead of a hand-kept stack.
- Index spaces are domains (`Item`, `Cut`, `V`, enum `Sign` indexing an
  array), and each bound, modulus and base is written once.
- No identity sentinels or fake monoids: an operation is defined on the type
  it combines.

For each problem: the problem, the original C++ (`main` only; the library template above it is omitted), the exemplar (how it should read, with syntax and stdlib that may not exist yet), the current port, and what differs.


---

## cf2069c: C. Beautiful Sequence

[Problem](https://codeforces.com/contest/2069/problem/C). Array of values 1..3, n ≤ 2·10^5. Count subsequences of length ≥ 3 where every element but the first has a smaller element to its left and every element but the last has a larger one to its right, mod 998244353. (Such a subsequence is exactly 1 2…2 3 with at least one 2.)

### Original C++

```cpp
int main() {
	using namespace std;
	ios_base::sync_with_stdio(false), cin.tie(nullptr);

	int T; cin >> T;
	while (T--) {
		int N; cin >> N;
		using num = modnum<998244353>;
		num dp = 0, ans = 0;
		num empty = 0;
		for (int i = 0; i < N; i++) {
			int a; cin >> a;
			if (a == 1) {
				dp++;
				empty++;
			} else if (a == 2) {
				dp += dp;
			} else if (a == 3) {
				ans += dp - empty;
			}
		}
		cout << ans << '\n';
	}

	return 0;
}
```

### Exemplar

```rust
// A beautiful subsequence is 1 2…2 3 with at least one 2. Scan once, counting
// subsequences that end in a 1, and those that end in "1 2+".
@!multitest
type Z = Mod<998_244_353>

let n: 3..=2e5 = read()
var ones, open, ans: Z = 0
for a in read<[n]1..=3>():
    match a:
        1 => ones += 1                 // start a new "1"
        2 => open = 2 * open + ones    // extend any "1 2*" by this 2, or start "1 2"
        3 => ans += open               // close every "1 2+"
print(ans)
```

### Current port

```rust
// Codeforces 2069 C: Beautiful Sequence.
type Z = Mod<998_244_353>

@!multitest
let n: Int = read()
var ones, one_twos, ans: Z = (0, 0, 0)
for _ in 0..n:
    match read::<1..=3>():
        1 => ones += 1
        2 => one_twos = one_twos * 2 + ones
        3 => ans += one_twos
print(ans)
```

### What differs

- Nearly the same as the current port. The bound `n: 3..=2e5` from the statement replaces `Int`.
- `for a in read<[n]1..=3>()` streams typed input; today it's `for _ in 0..n: match read::<1..=3>()`.
- The original's `dp − empty` trick turns into a variable whose name says what it counts.


---

## cf2084h: H. Turtle and Nediam 2

[Problem](https://codeforces.com/contest/2084/problem/H). Binary string s, n ≤ 2·10^6. One operation picks a window of 3, takes its median x and deletes the first x at or after the window start. Count the distinct strings reachable with at most n−2 operations, mod 1e9+7.

### Original C++

```cpp
int main() {
	using namespace std;
	ios_base::sync_with_stdio(false), cin.tie(nullptr);

	int T; cin >> T;
	while (T--) {
		int N; cin >> N;
		std::string S; S.reserve(N); cin >> S; assert(int(S.size()) == N);

		assert(N >= 1);

		std::vector<int> flips; flips.reserve(N+1);
		flips.push_back(0);
		flips.push_back(1);
		for (int i = 1; i < N; i++) {
			if (S[i] != S[i-1]) flips.push_back(i+1);
		}
		int tail_len = N+1 - flips.back();

		using num = modnum<int(1e9)+7>;
		int F = int(flips.size());
		std::vector<num> dp(F);
		std::array<int, 2> offset{0, 0};
		// len - offset, dp value
		std::array<std::vector<std::pair<int, num>>, 2> stk;
		std::array<num, 2> stk_val{0, 0};
		stk[0].reserve(F);
		stk[1].reserve(F);
		dp[0] = 1;
		dp[1] = 1;
		for (int i = 2; i < F; i++) {
			// first, add the option of i-1 .. i
			int z = i & 1;
			offset[z]++;
			int len = flips[i] - flips[i-1] - offset[z];
			if (i >= 3) {
				while (!stk[z].empty() && len >= stk[z].back().first) {
					auto [l, v] = stk[z].back();
					stk[z].pop_back();
					if (!stk[z].empty()) stk_val[z] -= (stk[z].back().first - l) * stk[z].back().second;
				}
				if (!stk[z].empty()) stk_val[z] += (stk[z].back().first - len) * stk[z].back().second;
				stk[z].push_back({len, dp[i-1]});
			}
			dp[i] = stk_val[z];
			if (!stk[z].empty()) {
				dp[i] += (stk[z].back().first + offset[z]) * stk[z].back().second;
			}
			dp[i] += num(flips[!z+1] - flips[!z] + offset[z] - 1);
		}

		num ans = dp.back();
		ans *= num(tail_len);
		if (int(flips.size()) == 2) ans--;
		cout << ans << '\n';
	}


	return 0;
}
```

### Exemplar

```rust
// The DP runs over maximal blocks of s. For each parity of block index, the
// transition sums, over every height h ≥ 1, the dp of the newest block at least h
// tall, while all blocks grow by one per round: a monotone "staircase".
@!multitest
type M = Mod<1e9 + 7>

let n: 3..=2e6 = read()
let blocks = read<Str>().runs().map(r => r.len)
if blocks.len == 1: return print(M(n - 1))

// At height h the newest step at least h tall is visible. Every step grows by one
// each round (`lift`), so tops are stored minus the lift and never rewritten.
struct Staircase:
    var steps: List<(top: Int, v: M)> = []   // tops strictly decreasing
    var lift = 0
    var hidden: M = 0                        // Σ v·(top − next top), all but the newest step

    fn push(mut self, height: Int, v: M):
        let top = height - lift
        while let s = steps.pop_if(s => s.top <= top):
            if let b = steps.last: hidden -= M(b.top - s.top) * b.v
        if let b = steps.last: hidden += M(b.top - top) * b.v
        steps.push((top, v))

    fn visible(self) -> M =                  // Σ over heights h ≥ 1 of the visible v
        hidden + (steps.last.map(s => M(s.top + lift) * s.v) ?? 0)

var stairs: [0..2]Staircase = Staircase()
var dp: M = 1
for (k, len) in blocks.drop_last().enumerate():
    let st = mut stairs[k % 2]
    st.lift += 1
    if k >= 1: st.push(len, dp)
    let floor = if k % 2 == 0: blocks.first else: 1
    dp = st.visible() + M(floor + st.lift - 1)
print(dp * M(blocks.last))
```

### Current port

```rust
@!multitest
type M = Mod<1_000_000_007>

let n: 3..=2_000_000 = read()
let s: Str = read()

var runs = List<Int>()
var run = 1
for i in 1..n:
    if s[i] == s[i - 1]:
        run += 1
    else:
        runs.push(run)
        run = 1
runs.push(run)

if runs.len == 1:
    print(M(n - 1))
else:
    var offset: [0..2]Int = 0
    var stk: [0..2]List<(len: Int, val: M)> = []
    var stk_sum: [0..2]M = 0
    var prev_dp: M = 1

    for k in 0..runs.len - 1:
        let z = k % 2
        offset[z] += 1
        let shifted = runs[k] - offset[z]

        if k >= 1:
            while let top = stk[z].last, shifted >= top.len:
                stk[z].pop()
                if let below = stk[z].last:
                    stk_sum[z] -= M(below.len - top.len) * below.val
            if let below = stk[z].last:
                stk_sum[z] += M(below.len - shifted) * below.val
            stk[z].push((len: shifted, val: prev_dp))

        var cur_dp = stk_sum[z]
        if let top = stk[z].last:
            cur_dp += M(top.len + offset[z]) * top.val
        let base = if z == 0: runs[0] else: 1
        cur_dp += M(base + offset[z] - 1)
        prev_dp = cur_dp

    print(prev_dp * M(runs.last!))
```

### What differs

- `read<Str>().runs()` replaces the hand-written run-length loop.
- The monotone stack becomes a named `Staircase` whose comment states its invariant. Today it's three parallel `[0..2]` arrays (`offset`, `stk`, `stk_sum`) indexed by parity.
- `steps.pop_if(pred)`, `drop_last()` and `let st = mut stairs[k % 2]` (a mutable alias to one element) don't exist today.


---

## cf2147f: F. Exchange Queries

[Problem](https://codeforces.com/contest/2147/problem/F). Two permutations p, s of n items; item i trades for j if p_i > p_j or s_i > s_j. After each of q swaps in p or s, print the number of pairs (i, j) such that j is reachable from i. n, q ≤ 10^5.

### Original C++

```cpp
int main() {
	using namespace std;
	ios_base::sync_with_stdio(false), cin.tie(nullptr);

	int T; cin >> T;
	while (T--) {
		int N, Q; cin >> N >> Q;
		std::vector<std::array<int, 2>> value(N, {-1, -1});
		for (int z = 0; z < 2; z++) {
			for (int i = 0; i < N; i++) {
				int v; cin >> v; v--;
				value[i][z] = v;
			}
		}

		struct seg_node {
			int lazy = 0;
			int first_zero;
			int last_zero;
			int64_t tot_cnt;
		};
		seg_tree::in_order_layout layout(N+1);
		std::vector<seg_node> seg(layout.N * 2);
		auto update_node = [&](seg_tree::point a) -> void {
			assert(seg[a].lazy >= 0);
			if (seg[a].lazy) {
				seg[a].first_zero = seg[a].last_zero = -1;
				seg[a].tot_cnt = 0;
			} else if (a >= layout.N) {
				seg[a].first_zero = seg[a].last_zero = layout.get_leaf_index(a);
				seg[a].tot_cnt = 0;
			} else {
				auto l = a.c(0);
				auto r = a.c(1);
				seg[a].tot_cnt = seg[l].tot_cnt + seg[r].tot_cnt;
				if (seg[l].last_zero != -1 && seg[r].first_zero != -1) {
					int d = seg[r].first_zero - seg[l].last_zero;
					seg[a].tot_cnt += int64_t(d) * int64_t(d - 1) / 2;
				}
				seg[a].first_zero = seg[l].first_zero != -1 ? seg[l].first_zero : seg[r].first_zero;
				seg[a].last_zero = seg[r].last_zero != -1 ? seg[r].last_zero : seg[l].last_zero;
			}
		};
		for (seg_tree::point a(2*layout.N - 1); a > 0; a--) update_node(a);

		auto update_range = [&](int i, int delta) -> void {
			auto [l, r] = value[i];
			if (l > r) std::swap(l, r);
			assert(l <= r);
			if (l == r) return;
			auto rng = layout.get_range(l+1, r+1);
			rng.for_each([&](auto a) -> void { seg[a].lazy += delta; update_node(a); });
			rng.for_parents_up(update_node);
		};

		for (int i = 0; i < N; i++) update_range(i, +1);

		for (int q = 0; q < Q; q++) {
			int z, i, j; cin >> z >> i >> j; z--, i--, j--;
			update_range(i, -1);
			update_range(j, -1);
			std::swap(value[i][z], value[j][z]);
			update_range(i, +1);
			update_range(j, +1);
			int64_t ans = int64_t(N) * int64_t(N+1) / 2;
			ans += seg[1].tot_cnt;
			assert(seg[1].first_zero == 0 && seg[1].last_zero == N);
			cout << ans << '\n';
		}
	}

	return 0;
}
```

### Exemplar

```rust
// Sort items by value. Cut c (values ≤ c vs > c) is free when no item has one value
// on each side; free cuts split the items into SCCs, each reaching all lower ones.
// So the answer is n(n+1)/2 plus Σ d(d−1)/2 over gaps d between consecutive free
// cuts. Item i covers cuts min(p,s) ..< max(p,s): keep, under range add, the cuts
// at the minimum cover (always 0, since cuts 0 and n are free) and their gap sum.
@!multitest
let n: 2..=1e5 = read()
let q: 1..=1e5 = read()
domain Item = 1..=n
domain Cut = 0..=n
var p, s: [Item]Cut = read()

struct Low(cover: Int, first: Cut, last: Cut, gaps: Int)

fn (a: Low) + (b: Low) -> Low = match cmp(a.cover, b.cover):
    less => a
    greater => b
    equal => Low(a.cover, a.first, b.last, a.gaps + b.gaps + choose2(b.first - a.last))

var low = LazySegTree<Cut>([Cut] c => Low(0, c, c, 0),
                           apply: (d: Int, l: Low) => l with cover: l.cover + d)

fn cover(i: Item, by d: Int):
    let (lo, hi) = minmax(p[i], s[i])
    low.apply(lo..hi, d)

for i in Item: cover(i, by: 1)
repeat q:
    let (t, i, j): (1..=2, Item, Item) = read()
    for x in [i, j]: cover(x, by: -1)
    if t == 1: p.swap(i, j) else: s.swap(i, j)
    for x in [i, j]: cover(x, by: 1)
    print(choose2(n + 1) + low.all.gaps)
```

### Current port

```rust
@!multitest
struct Node(mn: Int, first: Int, last: Int, pairs: Int)

fn merge(a: ?Node, b: ?Node) -> ?Node:
    let a = a ?? return b
    let b = b ?? return a
    if a.mn < b.mn: return a
    if b.mn < a.mn: return b
    let d = b.first - a.last
    Node(a.mn, a.first, b.last, a.pairs + b.pairs + d * (d - 1) / 2)

let n: 2..=100_000 = read()
let q: 1..=100_000 = read()
domain Item = 1..=n
domain Cut = 0..=n
var p: [Item]1..=n = read()
var s: [Item]1..=n = read()

let act = Action<?Node, Int>(
    id: 0,
    compose: (f, g) => f + g,
    apply: (f, x, _) => if let v = x: Node(v.mn + f, v.first, v.last, v.pairs) else: none,
    over: Monoid(id: none, op: merge),
)
var seg = LazySegTree<Cut, ?Node>(act, [Cut] c => Node(0, c.to_idx, c.to_idx, 0))

fn update_item(i: Item, delta: Int):
    let lo = min(p[i], s[i])
    let hi = max(p[i], s[i])
    if lo < hi: seg.apply(Cut(lo)..Cut(hi), delta)

for i in Item: update_item(i, 1)

let base = n * (n + 1) / 2
repeat q:
    let (tp, i, j): (1..=2, Item, Item) = read()
    update_item(i, -1)
    update_item(j, -1)
    if tp == 1: (p[i], p[j]) = (p[j], p[i])
    else: (s[i], s[j]) = (s[j], s[i])
    update_item(i, 1)
    update_item(j, 1)
    print(base + seg.all!.pairs)
```

### What differs

- The header comment states the editorial: free cuts, SCC gaps, n(n+1)/2 + Σ d(d−1)/2.
- The tree is built from a plain `+` on `Low` and an `apply`. Today it needs `Action(id:, compose:, apply:, over: Monoid(id: none, op:))`, plus `?Node` only to have an identity.
- `p.swap(i, j)`, `minmax`, `choose2`, `cmp` with `less/equal/greater`, and `with` record update.
- The permutation values are read straight into `Cut`, so there's no `Cut(lo)` conversion at the use site.


---

## cf2147h: H. Maxflow GCD Coloring

[Problem](https://codeforces.com/contest/2147/problem/H). Undirected graph, n ≤ 50, positive capacities. Color the vertices with the fewest colors so that, in each color's induced subgraph, some d ≥ 2 divides every pairwise max flow. Print the coloring.

### Original C++

```cpp
int main() {
	using namespace std;
	ios_base::sync_with_stdio(false), cin.tie(nullptr);

	int T; cin >> T;
	while (T--) {
		int N, M; cin >> N >> M;
		std::vector<kactl::Edge> edges(M);
		for (auto& e : edges) { cin >> e.u >> e.v >> e.w; e.u--, e.v--; }
		{
			auto gh = kactl::gomoryHu(N, edges);
			int64_t g = 0;
			for (auto e : gh) {
				g = std::gcd(g, e.w);
			}
			if (g != 1) {
				cout << 1 << '\n';
				cout << N << '\n';
				for (int i = 0; i < N; i++) cout << i+1 << " \n"[i+1==N];
			} else {
				tensor<bool, 2> sgn({N, N}, false);
				for (auto e : edges) { if (e.w & 1) { sgn[{e.u, e.v}] ^= 1, sgn[{e.v, e.u}] ^= 1; } }
				std::vector<bool> alive(N, true);
				std::vector<int> stk; stk.reserve(N);
				while (true) {
					int v;
					for (v = 0; v < N; v++) {
						if (!alive[v]) continue;
						int deg = 0;
						for (int u = 0; u < N; u++) {
							if (!alive[u]) continue;
							deg ^= sgn[{v, u}];
						}
						if (deg) break;
					}
					if (v == N) break;
					alive[v] = false;
					stk.push_back(v);
					for (int u = 0; u < N; u++) {
						if (!alive[u]) continue;
						if (!sgn[{v, u}]) continue;
						for (int w = 0; w < N; w++) {
							if (!alive[w]) continue;
							if (!sgn[{v, w}]) continue;
							if (u == w) continue;
							sgn[{u, w}] ^= 1;
						}
					}
				}
				std::vector<bool> side(N, false);
				while (!stk.empty()) {
					int v = stk.back(); stk.pop_back();
					assert(!alive[v]);
					for (int u = 0; u < N; u++) {
						if (!alive[u]) continue;
						if (!sgn[{v, u}]) continue;
						for (int w = 0; w < N; w++) {
							if (!alive[w]) continue;
							if (!sgn[{v, w}]) continue;
							if (u == w) continue;
							sgn[{u, w}] ^= 1;
						}
					}
					alive[v] = true;
					std::array<int, 2> cnt{0, 0};
					for (int u = 0; u < N; u++) {
						if (!alive[u]) continue;
						if (!sgn[{v, u}]) continue;
						cnt[side[u]] ^= 1;
					}
					side[v] = cnt[1] == 0 ? 1 : 0;
				}
				std::array<std::vector<int>, 2> res;
				res[0].reserve(N);
				res[1].reserve(N);
				for (int i = 0; i < N; i++) {
					res[side[i]].push_back(i);
				}
				cout << 2 << '\n';
				for (const auto& r : res) {
					cout << r.size() << '\n';
					for (int z = 0; z < int(r.size()); z++) cout << r[z]+1 << " \n"[z+1==int(r.size())];
				}
			}
		}
	}

	return 0;
}
```

### Exemplar

```rust
// The gcd of all pairwise max flows is the gcd of the Gomory–Hu tree's weights.
// If it is not 1, one color. Otherwise two colors always suffice (Gallai): split V
// so each side's odd-weight edges give every vertex even degree; then every cut
// inside a side has even weight, so 2 divides all its max flows.
@!multitest
domain V = read(1..=50)
let m: 0..=choose2(V.count) = read()
let edges = read<[m](u: V, v: V, w: 1..=1e6)>()

fn answer(colors: List<Bitset<V>>):
    print(colors.len)
    for c in colors:
        print(c.len)
        print(c)

if gcd(gomory_hu(edges).map(e => e.w)) != 1: return answer([V.all])

var odd = adjacency(edges.filter(e => e.w % 2 == 1), into: Bitset)

fn complement_within(s: Bitset<V>):          // toggle every edge inside s
    for u in s: odd[u] ^= s.without(u)

// Gallai's induction: an odd-degree v is removed after complementing its
// neighbourhood; on the way back v joins the side where its degree is even.
fn even_split(alive: Bitset<V>) -> Bitset<V>:
    let v = alive.find(v => (odd[v] & alive).len % 2 == 1) ?? return Bitset.empty
    let rest = alive.without(v)
    let nbr = odd[v] & rest
    complement_within(nbr)
    var side = even_split(rest)
    complement_within(nbr)
    if (nbr & side).len % 2 == 0: side.add(v)
    return side

let side = even_split(V.all)
answer([V.all - side, side])
```

### Current port

```rust
@!multitest
let n: 1..=50 = read()
let m: 0..=1225 = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V, w: 1..=1_000_000) = read()

var par: [V]V = V.first
var g = 0
for i in V:
    if i == V.first: continue
    var fg = FlowGraph<V>()
    for e in edges: fg.add_both(e.u, e.v, e.w)
    g = gcd(g, fg.max_flow(i, par[i]))
    let cut = fg.min_cut(i)
    for j in V:
        if j > i and par[j] == par[i] and cut[j]:
            par[j] = i

if g != 1:
    print(1)
    print(n)
    print([v for v in V])
else:
    var adj: [V]Bitset<V> = [V] _ => Bitset<V>.new()
    for e in edges:
        if e.w % 2 == 1:
            adj[e.u].toggle(e.v)
            adj[e.v].toggle(e.u)

    fn flip_clique(mut adj: [V]Bitset<V>, nbr: Bitset<V>):
        for u in nbr.each():
            adj[u] ^= nbr
            adj[u].remove(u)

    var alive = Bitset<V>.from([V] _ => true)
    var stk: List<V> = []
    loop:
        var pick: ?V = none
        for v in alive.each():
            if (adj[v] & alive).len % 2 == 1:
                pick = v
                break
        let v = pick ?? break
        alive.remove(v)
        stk.push(v)
        flip_clique(mut adj, adj[v] & alive)

    var side1 = Bitset<V>.new()
    while let v = stk.pop():
        let nbr = adj[v] & alive
        flip_clique(mut adj, nbr)
        alive.add(v)
        if (nbr & side1).len % 2 == 0: side1.add(v)

    let c0 = [v for v in V if v not in side1]
    let c1 = [v for v in V if v in side1]
    print(2)
    print(c0.len)
    print(c0)
    print(c1.len)
    print(c1)
```

### What differs

- `gomory_hu(edges)` is in the stdlib. Today the port hand-rolls Gusfield with a `par` array and n max-flows.
- The elimination becomes Gallai's induction, written as the recursive proof: find an odd vertex, complement its neighbourhood, recurse, undo, pick a side. Today it's a stack, a `?V` flag search loop, and a second loop that undoes the first.
- `alive.find(pred)`, `V.all`, `Bitset.empty`, `s.without(u)`, direct `for u in s`, and `adjacency(…, into: Bitset)`, which replaces the hand-maintained symmetric toggles.
- `answer(colors)` keeps the output format in one place for both branches.


---

## cf2255e2: E2. What Will Remain at the End? (Hard)

[Problem](https://codeforces.com/contest/2255/problem/E2). Array of n values; q encoded operations: range assign, range negate, range clamp to 0, or query p, the maximum subarray sum of position p's values over all earlier versions. Every operation creates a version. n, q ≤ 5·10^5.

### Original C++

```cpp
int main() {
	std::ios_base::sync_with_stdio(false), std::cin.tie(nullptr);

	int T; std::cin >> T;
	while (T--) {
		int N, Q; std::cin >> N >> Q;

		struct arr_val_t {
			// all required to be nonempty
			bool has = false;
			int64_t val = 0;
			int64_t tot = 0;
			int64_t pref = 0;
			int64_t suff = 0;
		};
		auto make_arr_val_t = [&](int64_t v) -> arr_val_t {
			return {true, v, v, v, v};
		};
		auto concat = [&](arr_val_t a, arr_val_t b) -> arr_val_t {
			if (!a.has) return b;
			if (!b.has) return a;
			return {
				true,
				std::max({a.val, b.val, a.suff + b.pref}),
				a.tot + b.tot,
				std::max(a.pref, a.tot + b.pref),
				std::max(b.suff, b.tot + a.suff)
			};
		};
		auto scale = [&](arr_val_t a, int64_t s) -> arr_val_t {
			a.val *= s;
			a.tot *= s;
			a.pref *= s;
			a.suff *= s;
			return a;
		};

		struct op_t {
			// let's start by maintaining the current value
			// first, it could be set
			bool has_set = false;
			int set_val = 0; // stores the current value
			// otherwise, for each input signum, we have an output signum
			std::array<int, 3> new_sign = {-1, 0, +1};

			std::array<arr_val_t, 3> pref_vals = {arr_val_t{}, arr_val_t{}, arr_val_t{}}; // this is just in terms of the input
			// tail_val only valid if has_set = true
			arr_val_t tail_val = arr_val_t{};
		};
		auto signum = [&](int v) -> int { return (v > 0) - (v < 0); };
		// first do a, then b
		auto apply_op = [&](op_t& a, const op_t& b) -> void {
			// first, apply b.pref_vals
			if (a.has_set) {
				int z2 = signum(a.set_val);
				a.tail_val = concat(a.tail_val, scale(b.pref_vals[z2+1], std::abs(a.set_val)));
				a.set_val = std::abs(a.set_val) * b.new_sign[z2+1];
			} else {
				for (int z = -1; z <= 1; z++) {
					int z2 = a.new_sign[z+1];
					a.new_sign[z+1] = b.new_sign[z2+1];
					a.pref_vals[z+1] = concat(a.pref_vals[z+1], b.pref_vals[z2+1]);
				}
			}
			if (b.has_set) {
				// now, apply b.tail_val
				if (!a.has_set) {
					a.tail_val = arr_val_t{};
					a.has_set = true;
				}
				a.set_val = b.set_val;
				a.tail_val = concat(a.tail_val, b.tail_val);
			}
		};

		seg_tree::in_order_layout layout(N);
		std::vector<op_t> seg(layout.N*2, op_t{});
		auto downdate_node = [&](seg_tree::point a) -> void {
			apply_op(seg[a.c(0)], seg[a]);
			apply_op(seg[a.c(1)], seg[a]);
			seg[a] = op_t{};
		};
		for (int i = 0; i < N; i++) {
			int v; std::cin >> v;
			seg_tree::point a = layout.get_point(i);
			seg[a].has_set = true;
			seg[a].set_val = v;
			seg[a].tail_val = make_arr_val_t(v);
		}
		uint64_t last_ans = 0;
		for (int q = 0; q < Q; q++) {
			int op; std::cin >> op;
			auto read_enc = [&]() -> int {
				uint64_t u; std::cin >> u;
				u ^= last_ans;
				return int(u % N);
			};
			auto read_range = [&]() -> auto {
				int l = read_enc();
				int r = read_enc();
				if (l > r) std::swap(l, r);
				return layout.get_range(l, r+1);
			};
			if (op == 1) {
				auto rng = read_range();
				int x; std::cin >> x;
				rng.for_parents_down(downdate_node);
				op_t cur;
				cur.has_set = true;
				cur.set_val = x;
				rng.for_each([&](seg_tree::point a) -> void { apply_op(seg[a], cur); });
			} else if (op == 2) {
				auto rng = read_range();
				rng.for_parents_down(downdate_node);
				op_t cur;
				cur.new_sign = {+1, 0, -1};
				rng.for_each([&](seg_tree::point a) -> void { apply_op(seg[a], cur); });
			} else if (op == 3) {
				auto rng = read_range();
				rng.for_parents_down(downdate_node);
				op_t cur;
				cur.new_sign = {0, 0, +1};
				rng.for_each([&](seg_tree::point a) -> void { apply_op(seg[a], cur); });
			} else if (op == 4) {
				auto a = layout.get_point(read_enc());
				a.for_parents_down(downdate_node);
				int64_t ans = seg[a].tail_val.val;
				std::cout << ans << '\n';
				last_ans = uint64_t(ans);
			} else assert(false);
			{
				op_t adv;
				adv.pref_vals[0] = make_arr_val_t(adv.new_sign[0]);
				adv.pref_vals[1] = make_arr_val_t(adv.new_sign[1]);
				adv.pref_vals[2] = make_arr_val_t(adv.new_sign[2]);
				apply_op(seg[1], adv);
			}
		}
	}
	return 0;
}
```

### Exemplar

```rust
// Every version appends each position's value to that position's history; a query
// is the best subarray of one history. A range tag is a function on (value,
// history). Negate and clamp only see the sign and scale with |value|, so before an
// assignment a tag is, per sign, a new sign plus the history it records per unit of
// |value|; after one, the value is constant and the recorded history fixed. Tags
// compose in O(1): one dual segment tree (range apply, point get) does everything.
enum Sign: neg = -1, zero = 0, pos = 1

struct Cell(val: Int, hist: MaxSub)          // MaxSub: stdlib Kadane monoid, scales by k ≥ 0
struct Path(to: Sign, seen: MaxSub)          // `seen` is per unit of |val|
struct Tag(path: [Sign]Path, set: ?Cell)     // set: value assigned, history since

fn run(f: Tag, c: Cell) -> Cell:
    let p = f.path[sign(c.val)]
    let hist = c.hist + p.seen * abs(c.val)
    if let s = f.set: Cell(s.val, hist + s.hist) else: Cell(abs(c.val) * Int(p.to), hist)

fn then(g: Tag, f: Tag) -> Tag:              // f first, then g
    if let s = f.set: return Tag(f.path, run(g, s))
    Tag([Sign] x => (let a = f.path[x], b = g.path[a.to]; Path(b.to, a.seen + b.seen)), g.set)

fn tag(to: Sign -> Sign = x => x, seen: Sign -> MaxSub = _ => .empty, set: ?Cell = none) -> Tag =
    Tag([Sign] x => Path(to(x), seen(x)), set)

let record = tag(seen: x => MaxSub.of(Int(x)))
let negate = tag(to: x => Sign(-Int(x)))
let clamp = tag(to: x => max(x, .zero))
fn assign(v: Int) -> Tag = tag(set: Cell(v, .empty))

@!multitest
let (n, q): (1..=5e5, 1..=5e5) = read()
domain I = 1..=n
let a: [I]-1e9..=1e9 = read()
var st = DualSegTree<I>([I] i => Cell(a[i], MaxSub.of(a[i])), id: tag(), compose: then, apply: run)
var last: U64 = 0
fn at(y: U64) -> I = I.at((y ^ last) % n)
repeat q:
    match read<1..=4>():
        4 =>
            let ans = st[at(read())].hist.best
            print(ans)
            last = U64.wrap(ans)
        kind =>
            let (l, r) = minmax(at(read()), at(read()))
            st.apply(l..=r, match kind: 1 => assign(read()); 2 => negate; _ => clamp)
    st.apply(.., record)
```

### Current port

```rust
// Every version appends each position's value to that position's history, so a range tag
// is a function on (value, history summary). Negate and clamp only look at the sign, and
// scale with |value|: before the first assignment a tag maps each sign to a new sign and
// a history per unit of |value|; after it, the value is a constant and the history is fixed.
type Seg = (best: Int, total: Int, pre: Int, suf: Int)

fn join(a: ?Seg, b: ?Seg) -> ?Seg:
    let x = a ?? return b
    let y = b ?? return a
    (
        max(max(x.best, y.best), x.suf + y.pre),
        x.total + y.total,
        max(x.pre, x.total + y.pre),
        max(y.suf, y.total + x.suf),
    )

fn scaled(a: ?Seg, k: 0..=1e9) -> ?Seg:
    let x = a ?? return none
    (x.best * k, x.total * k, x.pre * k, x.suf * k)

fn one(v: Int) -> ?Seg = (v, v, v, v)

type Sign = -1..=1

fn sign(v: Int) -> Sign = if v < 0: -1 elif v == 0: 0 else: 1

type Cell = (val: Int, hist: ?Seg)
type Path = (to: Sign, seen: ?Seg)
type Tag = (neg: Path, zero: Path, pos: Path, set: ?Cell)

fn path(f: Tag, s: Sign) -> Path = if s < 0: f.neg elif s == 0: f.zero else: f.pos

fn tag(to: Sign -> Sign, seen: Sign -> ?Seg = s => none, set: ?Cell = none) -> Tag =
    ((to(-1), seen(-1)), (to(0), seen(0)), (to(1), seen(1)), set)

fn run(f: Tag, c: Cell) -> Cell:
    let p = path(f, sign(c.val))
    let hist = join(c.hist, scaled(p.seen, abs(c.val)))
    if let s = f.set: (s.val, join(hist, s.hist)) else: (abs(c.val) * p.to, hist)

fn then(g: Tag, f: Tag) -> Tag:
    if let s = f.set: return (f.neg, f.zero, f.pos, run(g, s))
    fn via(p: Path) -> Path = (path(g, p.to).to, join(p.seen, path(g, p.to).seen))
    (via(f.neg), via(f.zero), via(f.pos), g.set)

let identity = tag(s => s)
let record = tag(s => s, s => one(s))
let negate = tag(s => -s)
let clamp = tag(s => max(s, 0))
fn assign(x: Int) -> Tag = tag(s => s, set: (x, none))

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
domain I = 1..=n
let a: [I]-1e9..=1e9 = read()
let blank: Cell = (0, none)
let cells = Action(
    id: identity,
    compose: (g, f) => then(g, f),
    apply: (f, c, span) => run(f, c),
    over: Monoid(id: blank, op: (c, d) => c),
)
var st = LazySegTree<I, Cell>(cells, [I] i => (a[i], one(a[i])))
var last: U64 = 0
fn at(y: U64) -> I = I(Int((y ^ last) % U64(n)) + 1)
repeat q:
    let kind: 1..=4 = read()
    if kind == 4:
        let p = at(read())
        let ans = st.get(p).hist!.best
        print(ans)
        last = U64(ans)
    else:
        let (l, r) = (at(read()), at(read()))
        let f = match kind:
            1 => assign(read())
            2 => negate
            _ => clamp
        st.apply(min(l, r)..=max(l, r), f)
    st.apply(.., record)
```

### What differs

- `enum Sign` indexes an array: `Tag(path: [Sign]Path, …)` and `f.path[sign(v)]`. Today an enum can't be a domain, so there are three fields and a hand-written selector.
- `MaxSub` is a stdlib monoid (Kadane summary, concatenation `+`, scaling `* k`). Today it's a 4-tuple, `?Seg`, `join` and `scaled`, written out.
- `DualSegTree` (range apply, point get) needs no fake monoid. Today `over: Monoid(id: blank, op: (c, d) => c)` is there only to satisfy `LazySegTree`.
- The same structure must run at C++ speed: the current port is 4.4x slower (records passed by value, `std::function`).
