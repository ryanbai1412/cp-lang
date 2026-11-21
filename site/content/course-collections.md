Once you have basic arrays and domains down, most contest solutions live in a handful of standard containers and algorithms: dynamic lists, priority queues, sorted sets, hash maps, sorting with custom keys, coordinate compression, and binary search. CP Lang keeps their names short, their return types uniform, and their mutability explicit.

## `List`, `Deque`, and priority queues

A `List<T>` is a dynamic array (`std::vector<T>` in C++17). A `Deque<T>` supports `O(1)` operations at both ends (`push`, `pop`, `push_front`, `pop_front`, `.first`, `.last`). Every popping or peeking operation returns `?T` — `none` when empty — so you never trigger undefined behavior by reading `.back()` on an empty stack.

```cp CP Lang 0.2.2
let a = [3, 1, 2, 2, 4]
var st: List<Int> = []
for x in a:
    while st.last != none and st.last! >= x:
        st.pop()
    st.push(x)
```

For heaps, `std.heap` provides `MinHeap<T>` and `MaxHeap<T>`. If you need a mixed sort order — for example, smallest distance first and largest index on ties — wrap the descending component in `desc(x)`.

```example collections_heap
```

```exercise coll-heap-output
```

```exercise coll-heap-desc
```

## `Set`, `SortedBag`, `Map`, and `HashMap`

CP Lang separates unique ordered sets (`Set<T>`, like `std::set`) from ordered multisets (`SortedBag<T>`, like `std::multiset`) so you never accidentally erase all duplicates with `.erase(x)`:

- `SortedBag<T>` has `.insert(x)`, `.remove_one(x)` (removes a single copy), `.count(x)`, `.min`, `.max`, `.first_ge(x)`, `.last_le(x)`.
- `Map<K, V>` is an ordered tree map; `HashMap<K, V>` is a splitmix64-salted hash map that is immune to standard anti-hash blow-up tests on Codeforces.
- Reading `m[k]` returns `?V` (never silently inserts a default value the way C++ `map::operator[]` does!).
- Updating or initializing an entry uses compound operators directly:

```cp CP Lang 0.2.2
let a = [2, 1, 2]
var freq = HashMap<Int, Int>()
for x in a:
    freq[x] += 1                   // initializes missing key to 0, then adds 1

var buckets = HashMap<Int, List<Int>>()
for i in 0..a.len:
    let x = a[i]
    (buckets[x] ??= []).push(i)    // lazily creates [] if absent, then lends mutably
```

```exercise coll-map-lazy
```

## Sorting, coordinate compression, and sequence helpers

In-place sequence methods mutate their receiver (`a.sort()`, `a.reverse()`), while non-mutating `.sorted()` and `.reversed()` return a new `List`. Pass a key projection with `by:` instead of writing a verbose comparator lambda:

```cp CP Lang 0.2.2
use std.algebra.{desc}

type Interval = (l: Int, r: Int)
var intervals: List<Interval> = [(3, 5), (1, 5), (2, 4)]
let all_coords = [10, 5, 10]
intervals.sort(by: iv => (iv.r, desc(iv.l)))
let xs = compress(all_coords)      // sorted unique values; xs.at(v) returns its typed index
```

Three prelude sequence helpers replace repetitive contest loops:

- `prefix(a)` builds a `[0..=a.len]Int` prefix sum array (`pre[r] - pre[l]` is the sum on `l..r`).
- `runs(s)` yields `(value, count)` run-length groups in a single pass.
- `xs.runs(by: key)` groups neighbours with the same `key` into `(key, items)`, and `xs += ys` appends a whole list.
- `for b in Bool` visits `false` then `true`, so a two-way choice needs no `[false, true]` list.
- `a.distinct()` returns the sorted unique elements of `a`.

## Binary search: answer space vs sorted containers

In CP Lang, binary search comes in two clean shapes:

1. **Answer-space search** with `bsearch_first(lo..=hi, pred) -> ?Int` (smallest `x` where `pred(x)` is true) and `bsearch_last(lo..=hi, pred) -> ?Int` (largest `x` where `pred(x)` is true). Both return `none` if no value in the range satisfies `pred` — no off-by-one sentinel bugs.
2. **Sorted-sequence and set search** with `.first_ge(x)`, `.first_gt(x)`, `.last_le(x)`, `.last_lt(x)`:
   - On an **array or slice**, they return `?Index` (the position).
   - On a **`Set<T>` or `SortedBag<T>`**, they return `?T` (the element value itself).

```example sort_bsearch
```

```exercise coll-bsearch-output
```

```exercise coll-first-ge-types
```

## Write a program

```exercise coll-write-smallest
```
