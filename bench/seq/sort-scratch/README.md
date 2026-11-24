# sort: tuned std body vs the builtin sort arm (scratch, not for main)

Handoff to Perf A. `std/seq.cp` on this branch has the tuned `sort` body (`sort` + `sort_run`):
top-down stable merge sort, in place through `self[ks.nth(i)]`, one `n/2` `List<T>` buffer, insertion
sort for runs <= 8, `by` keys recomputed (never stored). It is the same algorithm as the arm's C++
`cpl::merge_sort`. The arms still catch `sort`/`sorted`, so each `*_body.cp` carries a private copy
of the body named `mysort` / `my_sort_run` that the arms cannot see.

| bench | arm | std body | body/arm | hand C++ (`bench/seq/*.cpp`) |
|---|---|---|---|---|
| `sort_by_records` (1M records, `by: desc(cost)`, x4) | 0.16 s / 38 MB | 0.23 s / 50 MB | 1.4x (about 1.5x on the sort alone; generation is 0.03 s) | 0.18 s |
| `sort_by_cross` (`by:` key with a user `Ord`) | 0.49 s / 149 MB | 0.59 s / 166 MB | 1.2x | 0.41 s |
| `slice_sort` (40 sorts of 100k-Int `MutView`s) | 1.15 s / 4 MB | 0.37 s / 5 MB | 0.32x (body wins) | 0.22 s |

g++ 11.4 `-O2 -std=c++17`, 8 cores, 5 runs each, wall time and max RSS from `/usr/bin/time`; outputs equal.

## Commands

```sh
cargo build --release --bin cpl
cd bench/seq/sort-scratch
for f in slice_sort sort_by_records sort_by_cross; do
  for v in arm body; do
    ../../../target/release/cpl build ${f}_$v.cp -o ${f}_$v.cpp
    g++ -O2 -std=c++17 -o ${f}_$v ${f}_$v.cpp
  done
done
for r in 1 2 3 4 5; do for f in slice_sort sort_by_records sort_by_cross; do for v in arm body; do
  /usr/bin/time -f "${f}_$v %e s %M KB" ./${f}_$v > /dev/null
done; done; done
# IR of the hot fns (scratch-only hook in crates/cpl-ir/src/lower/mod.rs):
CPL_DUMP_IR=my_sort_run ../../../target/release/cpl build sort_by_records_body.cp -o /dev/null 2> my_sort_run.ir
```

## Where the +12 MB on sort_by_records comes from

Counted with a global `operator new` override (`newcount.h` below, `#include`d first):

- arm: 8 allocations >= 1 MB over 4 rounds: per round the 24 MB records list and one 12,000,000 B
  half buffer (`std::vector<V> buf(b, b + n/2)`, sized exactly).
- body: 20 allocations >= 1 MB: per round the 24 MB records list and the half buffer, built by the
  comprehension `[self[ks.nth(i)] for i in 0..n / 2]` with **no `reserve`**, so it grows
  1.5 -> 3.1 -> 6.3 -> 12.6 MB (12,582,912 B final capacity); the last regrowth holds 6.3 + 12.6 MB at once.
- The buffer is allocated once per `sort` call, not per recursion or merge. `by:` keys are never
  materialised (`Desc_Int_14` is built in registers per compare). So the extra RSS is the unreserved
  comprehension's geometric growth (peak about 19 MB vs 12 MB), not keys. Its cost is a few ms, so
  it doesn't explain the 1.4x.

```cpp
#include <cstdlib>
#include <cstdio>
#include <new>
static size_t g_n=0,g_b=0,g_big=0;
void* operator new(size_t s){ g_n++; g_b+=s; if(s>=1<<20){g_big++; fprintf(stderr,"alloc %zu\n",s);} void*p=malloc(s); if(!p) throw std::bad_alloc(); return p;}
void operator delete(void*p) noexcept { free(p);} void operator delete(void*p,size_t) noexcept { free(p);}
struct Rep{~Rep(){fprintf(stderr,"news=%zu bytes=%zu big=%zu\n",g_n,g_b,g_big);}} g_rep;
```

## Other variants tried (sort_by_records, body time)

| variant | time |
|---|---|
| main's std body (`List<(K,T)>` copies, bottom-up passes) | 0.32 s |
| copy into `List<T>`, bottom-up, insertion runs of 8 | 0.24 s |
| copy into `List<T>`, top-down recursion | 0.24 s |
| copy into `List<T>` + `std::stable_sort` on it (hand-edited C++) | 0.30 s |
| in place via `ks.nth`, top-down (this branch) | 0.23 s |

## Hot merge loop: arm vs body (generated C++, sort_by_records)

Arm (`cpl::merge_sort` in the runtime prelude, comparator from the arm):

```cpp
// call site
bool t2 = true;
auto t3 = [&](const std::tuple<ll, ll, ll>& j) -> ll { return std::get<0>(j); };
cpl::stable_sort_lt(t1.begin(), t1.end(), [&](const auto& t4, const auto& t5) { return t2 ? t3(t5) < t3(t4) : t3(t4) < t3(t5); });
// merge step
size_t m = lo + (hi - lo) / 2;
merge_sort(b, m, hi, buf, lt);
merge_sort(b, lo, m, buf, lt);
auto i = buf, im = std::move(b + lo, b + m, buf);
auto j = b + m, jm = b + hi, k = b + lo;
for (; i != im && j != jm; ++k) { if (lt(*j, *i)) { *k = std::move(*j); ++j; } else { *k = std::move(*i); ++i; } }
std::move(i, im, k);
```

Body (`my_sort_run`, from the std-style body; `S = List<Rec>`, `ks: Range<Int>`):

```cpp
void my_sort_run_Lst_Rec3_cost_Int_v_Int_p_Int_Int_Rec3_cost_Int_v_Int_p_Int_S14_149(std::vector<std::tuple<ll, ll, ll>>& a, const cpl::Range<ll>& ks, ll lo, ll hi, std::vector<std::tuple<ll, ll, ll>>& buf, [[maybe_unused]] bool by) {
  if (cpl::isub(hi, lo) <= 8) {
    for (ll i = cpl::iadd(lo, 1), i_end = hi; i < i_end; i++) {
      std::tuple<ll, ll, ll> x = a[cpl::iadd(ks.lo, i)];
      Desc_Int_14 kx = cpl_fn0(x);
      ll j = i;
      while (j > lo && Desc_Int_14_lt(kx, cpl_fn0(a[cpl::iadd(ks.lo, cpl::isub(j, 1))]))) {
        ll t0 = cpl::iadd(ks.lo, j);
        a[t0] = a[cpl::iadd(ks.lo, cpl::isub(j, 1))];
        j = cpl::isub(j, 1);
      }
      ll t1 = cpl::iadd(ks.lo, j);
      a[t1] = x;
    }
    return;
  }
  ll mid = cpl::iadd(lo, cpl::isub(hi, lo) >> 1);
  my_sort_run_Lst_Rec3_cost_Int_v_Int_p_Int_Int_Rec3_cost_Int_v_Int_p_Int_S14_149(a, ks, lo, mid, buf, false);
  my_sort_run_Lst_Rec3_cost_Int_v_Int_p_Int_Int_Rec3_cost_Int_v_Int_p_Int_S14_149(a, ks, mid, hi, buf, false);
  if (!Desc_Int_14_lt(cpl_fn0(a[cpl::iadd(ks.lo, mid)]), cpl_fn0(a[cpl::iadd(ks.lo, cpl::isub(mid, 1))]))) {
    return;
  }
  ll m = cpl::isub(mid, lo);
  for (ll t = 0; t < m; t++) {
    buf[t] = a[cpl::iadd(ks.lo, cpl::iadd(lo, t))];
  }
  ll i_2 = 0;
  ll j_2 = mid;
  ll k = lo;
  while (i_2 < m && j_2 < hi) {
    if (Desc_Int_14_lt(cpl_fn0(a[cpl::iadd(ks.lo, j_2)]), cpl_fn0(buf[i_2]))) {
      ll t2 = cpl::iadd(ks.lo, k);
      a[t2] = a[cpl::iadd(ks.lo, j_2)];
      j_2 = cpl::iadd(j_2, 1);
    } else {
      ll t3 = cpl::iadd(ks.lo, k);
      a[t3] = buf[i_2];
      i_2 += 1;
    }
    k = cpl::iadd(k, 1);
  }
  return [&](auto&& a, auto&& ks, auto&& buf, auto&& m, auto&& i_2, auto&& k) __attribute__((noinline)) -> void {
    while (i_2 < m) {
      ll t4 = cpl::iadd(ks.lo, k);
      a[t4] = buf[i_2];
```

Differences visible in the C++:
- The body indexes `std::vector&` through `a[cpl::iadd(ks.lo, j)]` (an `S: IndexedMut` element via
  `ks.nth`). The arm walks raw iterators, so it never reloads `a.data()` or adds `ks.lo`.
- The body copies (`a[t2] = a[...]`, `buf[t] = a[...]`) where the arm moves. That's the same for this POD
  tuple, but it matters for non-trivial `T` (`sort_by_cross`).
- The body's tail `while i < m` is emitted as a `__attribute__((noinline))` lambda.
- The body's key compare is `Desc_Int_14_lt(cpl_fn0(x), cpl_fn0(y))`, all `inline`. The arm keeps `desc`
  unwrapped and compares `ll` with a runtime `t2` flag.

`perf` is blocked on the VM (`perf_event_paranoid`), so I have no hardware profile.
