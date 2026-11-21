# C++ idiom → CP Lang spelling [proposed]

One line per idiom. Every spelling is from spec chapters 1–8; nothing here is new.

## Input and indexing
| C++ | spelling |
|---|---|
| `int n; cin >> n; vector<int> a(n);` | `domain A = read(1..=1e5)`, then `let a: [A]Int = read()` |
| `int k; cin >> k; for k items` | `let vs: [len: 1..=n]V = read()` |
| `--x` after reading a 1-based index | `with io(base: 1)` at the top of the file |
| mixed 0/1-based input | `read(base: 0)` at that one call |
| `vector<int>(n)` indexed by `int` | anonymous `[n]T`, indexed `0..n` |
| `pre[i + 1] - pre[l]` | `pre[r.after] - pre[r.before]` (cuts) |
| `a.begin() + l, a.begin() + r` | `a[l .. r]` or `a[prev]` with `prev: Range<Cut>` |
| `n - 1 - i` | `D.mirror(i)`; suffix work with `suffix_max`/`suffix_sums`; iterate backwards with `xs.rev()` |
| `max(0, i - k)` | `p.back(k)` (clamped) |
| `i + 1 < n ? i + 1 : -1` | `v.next` (`?D`) |
| `(i + 1) % n` | `domain K = ring(n)`, then `i + 1` |
| `id[x] = cnt++` over a subset | `let (domain K, xs) = S.filter(p).numbered()` |
| coordinate compression | `let (domain H, hs) = compress(ys)`; `H(y)`, `hs.value[h]` |
| `grid[r][c]` with `r * C + c` | `domain Cell = grid(r, c)`; `cell.row`, `cell.neighbours` |
| source/sink as `n`, `n + 1` | `domain Node = Cell + labels("S", "T")` |

## Loops and scans
| C++ | spelling |
|---|---|
| `int best = 0; for … best = max(best, x)` | `xs.max()` (`?T`), or `prefix_max` / `suffix_max` |
| `-1` / `INF` seed | `?T` with `max=` / `min=` (`none` = no value yet) |
| `if (i > 0) prev = a[i - 1]` | `for (prev, x) in xs.pairs()`, or `xs.with_prev()` |
| `pk = -1; if a[i] != a[pk]` | `xs.runs(by: key)` |
| `pre[0] = 0; pre[i + 1] = pre[i] + a[i]` | `a.prefix_sums()` (indexed by cuts) |
| `x = f(x)` accumulation, final value only | `xs.fold(init, step)` (no buffer) |
| every intermediate `x` | `xs.scan(init, step)` |
| `res *= …` | `xs.map(f).product()` |
| `lo = 0, hi = n; while (lo < hi)` | `(lo..=hi).partition_point(p)` |
| `if (seen[v]) continue; seen[v] = 1;` | `if not seen.mark(v): continue` |
| `nth_element` / top-k | `xs.min_k(k, by:)` |
| `while (u != -1) u = nxt[u]` | `successors(first: u, next: v => nxt[v])` |
| `cnt += (a[i] > 0)` | `xs.count(where: x => x > 0)` |

## Containers
| C++ | spelling |
|---|---|
| `s.upper_bound(x)` | `s.first(after: x)` |
| `s.lower_bound(x)` | `s.first(from: x)` |
| `prev(s.lower_bound(x))` | `s.last(before: x)` |
| `*s.begin()`, `s.erase(s.begin())` | `s.first`, `s.pop_first()` |
| `map<int,int> cnt; cnt[x]++` | `Counter<T>`: `counts.add(x)`, `counts.of(x)` |
| order-statistics tree | `RankedCounter<T>`: `kth(i, order:)`, `rank(x)`; standalone, not a segment-tree payload |
| `priority_queue<int>` | `Heap<T>(order: desc)` (order required) |
| DSU with size arrays | `Dsu<V, M>(init:)`; `groups.fold(v)` |
| `vector<pair<int,int>> adj[n]` | `Graph<V>.undirected(from:, to:, weight:)`; `g.adj(v)` |

## Algorithms and numbers
| C++ | spelling |
|---|---|
| lazy segment tree, 80 lines | `SegTree<D, M, A>(init:)`; `fold`, `apply`, `prefix_fold(while:)` |
| BIT | `Fenwick<D, M>` |
| `memset(dp, -1)` memo | `@memo` |
| `fact[]`, `inv_fact[]`, `C(n, k)` | `Z.factorials(n)`; `f.binom(n, k)` |
| `modint<MOD>` | `type Z = Mint(mod: 998244353)`, or `Mint(mod: p)` for a runtime p |
| `__builtin_popcount`, mask loops | `Bits<D>` |
| `assert(x)` | `assert(x)` |
| `fabs(a - b) < eps` | `approx(a, b, abs: eps)` |
| `cout << (ok ? "YES" : "NO")` | `print(yes(ok))` |
| `int t; cin >> t; while (t--) solve();` | `--- with multitest() ---`; the rest of the file is the test |
| `"Case #" << t << ": "` | `--- with multitest(prefix: i => "Case #{i}: ") ---` |

## Testing
| C++ habit | spelling |
|---|---|
| separate `brute.cpp` + script | `@alt(program, n: ..=8) fn slow()` above the wrap line, same file |
| custom checker | `@check` |
| random generator | `@gen` |
| interactive local judge | `@interactor`; message enum with `Read + Print` (+ `Limited`); `Wire<Query>(error_reply:, limit:)` |

## Round-3 additions [p3]

| C++ | CP Lang |
|---|---|
| `-1` as "absent" in a lookup table | `var nxt: [D]?T = none` |
| `INF` / `0x3f` seed in a min-DP | `?Int` with `min=`, then `?? -1` when printing |
| `vector<int> d; sort(d)`, then index it | `let (domain D, d) = xs.sorted().numbered()` |
| children lists from a parent array | `parent.fibers()` |
| `tin`/`tout`, subtree = `[tin, tout)` | `t.euler()`, `tour.span[v]` |
| find the cycle of a unicyclic graph | `g.cycle()`; for a functional graph, `successors` + `seen.mark` |
| root a forest | `Tree(edges).rooted(at: roots)` |
| multi-source BFS with `dist = -1` | `bfs(from: roots, next: g.adj).dist` |
| `dx`/`dy` arrays, `"URDL"[k]` | `cell.neighbours`, `cell.step(d)`, `a.dir_to(b)` with `Dir4` |
| `if (d + w < dist[v]) { dist[v] = d + w; pq.push(…) }` | `if dist[v] min= d + w: heap.push(…)` |
| `lo`/`hi`/`mid` binary search | `D.partition_point(p)`, then `cut.right!` |
| KMP `pi` and border chain | `s.prefix_function()`, `s.borders()` |
| `2*i`, `2*i^1` for the two sides of an edge | `type Side = (Edge, Bool)` |
| bit-packed small state `x & 2`, `x & 1` | `type State = (fed: Bool, blue: Bool)` |
| `(int)modnum` | `Int(z)` |
| `for (b…) if (w >> b & 1)` | `for i in Bits<B>.from_raw(w)` |
| `res ^= x` | `xs.fold(0, (a, x) => a ^ x)`, or the `Xor` monoid |
| `x == -1 ? y : f(x)` | `x.map(f) ?? y` |
| first index with `p`, asserted to exist | `D.first(where: p)!` |
| `(k + 1) % 4` direction turn | `domain Dir = ring(labels(N, E, S, W))`, `d + 1` |
| `dp[i+1]` from `dp[i]` rolling rows | `fill` + `swap`; `cur = nxt` copies |
| argmax, first index on ties | `xs.max_by(key)`; in a segment tree, `ArgMax(ties: first)` |
| DSU tracking per-component min/max | `Dsu<V, Hull<V>>` (semigroup) |
| `cout << idx` summed or XORed | `v.num` |
| `bool a; cin >> a` | `let a: [V]Bool = read()` |
| recursive lambda | nested `fn` |
| `dist[v]` read after a fill loop that set every entry | `let dist = dist!` (`[V]?Int` → `[V]Int`, one check) |
| `mx = max(...)`, then `count(v.begin(), v.end(), mx)` | `let (mx, cnt) = xs.max_count() ?? (…)` |
