//! A dense set of positions of a domain, 64 per word.

/// One bit per position of `D`, in `ceil(K / 64)` words.
/// `Bitset<D>` is an explicit type you ask for; `[D]Bool` stays one byte
/// per element and never turns into this behind your back.
/// Single-bit operations are O(1); whole-set operations are O(K / 64).
pub struct Bitset<D>:
    w: List<U64>
    n: Int

    /// The empty set, every word zero; the same set as `empty()`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>().len == 0`
    pub fn new() -> Bitset<D>:
        Bitset(List.filled((D.K + 63) / 64, U64(0)), 0)

    /// The empty set, matching `Subset<D>.empty` (a call here: it is O(K / 64)); the same as `new()`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.empty().is_empty`
    pub fn empty() -> Bitset<D> = Bitset<D>.new()

    /// The full set containing every position of `D`, matching `Subset<D>.full`. No bit past the last
    /// position is ever set, so `len` is `D.K`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.full().len == 10`
    pub fn full() -> Bitset<D>:
        var b = Bitset(List.filled((D.K + 63) / 64, ~U64(0)), D.K)
        b.trim()
        b

    /// The singleton set `{i}`, matching `Subset<D>.single(i)`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.single(D(4)).each().collect() == [D(4)]`
    pub fn single(i: D) -> Bitset<D>:
        var b = Bitset<D>.new()
        b.add(i)
        b

    /// The set of positions where `a` is true.
    ///
    /// cost: O(K) time, O(K / 64) space
    /// empty: the empty set when no element is true
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.from([D] v => Int(v) % 4 == 0).each().collect() == [D(0), D(4), D(8)]`
    pub fn from(a: [D]Bool) -> Bitset<D>:
        var b = Bitset<D>.new()
        for i in D:
            if a[i]: b.add(i)
        b

    /// Build a bitset from a list or generator of positions `D`. Repeated positions are set once.
    ///
    /// cost: O(K / 64 + xs.len) time, O(K / 64) space
    /// empty: the empty set
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(3), D(1), D(3)]).each().collect() == [D(1), D(3)]`
    pub fn of(xs: List<D>) -> Bitset<D>:
        var b = Bitset<D>.new()
        for i in xs: b.add(i)
        b

    /// Whether position `i` is set; `i in b` calls it.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; let b = Bitset<D>.single(D(4)); b.contains(D(4)) and not b.contains(D(5))`
    pub fn contains(self, i: D) -> Bool = ((w[i.to_idx / 64] >> (i.to_idx % 64)) & U64(1)) == U64(1)

    /// Set the bit at position `i`; returns `true` if `i` was not already set (matches `Set.add`).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>(); let x = b.add(D(2)); let y = b.add(D(2)); x and not y and b.len == 1`
    pub fn add(mut self, i: D) -> Bool:
        if self.contains(i): return false
        w[i.to_idx / 64] |= bit(i.to_idx % 64)
        n += 1
        true

    /// Clear the bit at position `i`; returns `true` if `i` was set (matches `Set.remove`).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.single(D(2)); let x = b.remove(D(2)); let y = b.remove(D(2)); x and not y and b.is_empty`
    pub fn remove(mut self, i: D) -> Bool:
        if not self.contains(i): return false
        w[i.to_idx / 64] &= ~bit(i.to_idx % 64)
        n -= 1
        true

    /// Flip the bit at position `i`: set it if clear, clear it if set.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.single(D(2)); b.toggle(D(2)); b.toggle(D(5)); b.each().collect() == [D(5)]`
    pub fn toggle(mut self, i: D):
        n += if self.contains(i): -1 else: 1
        w[i.to_idx / 64] ^= bit(i.to_idx % 64)

    /// A copy of `self` with `i` added; `self` is unchanged.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.single(D(1)).with(D(3)).each().collect() == [D(1), D(3)]`
    pub fn with(self, i: D) -> Bitset<D>:
        var out = Bitset(w, n)
        out.add(i)
        out

    /// A copy of `self` with `i` removed; `self` is unchanged. Removing a non-member copies.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.full().without(D(0)).len == 9`
    pub fn without(self, i: D) -> Bitset<D>:
        var out = Bitset(w, n)
        out.remove(i)
        out

    /// A copy of `self` with `i` added if `on` or removed otherwise, matching `Subset<D>.set`.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.single(D(1)).set(D(1), false).is_empty`
    /// example: `domain D = 0..10; Bitset<D>().set(D(7), true).each().collect() == [D(7)]`
    pub fn set(self, i: D, on: Bool) -> Bitset<D>:
        if on: self.with(i) else: self.without(i)

    /// The complement of `self` within `D`, matching `Subset<D>.complement`; bits past the last position
    /// stay clear.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).complement().len == 7`
    pub fn complement(self) -> Bitset<D>:
        var out = Bitset([~x for x in w], D.K - n)
        out.trim()
        out

    /// Remove every position from the set.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.full(); b.clear(); b.is_empty`
    pub fn clear(mut self):
        for k in 0..w.len: w[k] = U64(0)
        n = 0

    /// True when some position in this set satisfies `pred`. Visits members in increasing order and stops
    /// at the first that satisfies it.
    ///
    /// cost: O(K / 64 + visited) words scanned and calls of `pred`
    /// empty: `false`
    /// traps: never (beyond what `pred` traps on)
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).any(d => Int(d) > 5)`
    /// example: `domain D = 0..10; not Bitset<D>().any(d => true)`
    pub fn any(self, pred: D -> Bool) -> Bool = self.find(pred) != None
    /// The number of positions in the set; the count is kept current by every update.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).len == 3`
    @get pub fn len(self) -> Int = n

    /// The same number as `len`, under the name C++ gives it.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).count == 3`
    @get pub fn count(self) -> Int = n

    /// Whether the set has no members. O(1): the count is kept current by every update.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>().is_empty and not Bitset<D>.full().is_empty`
    @get pub fn is_empty(self) -> Bool = n == 0

    /// The smallest position in the set, or `None`.
    ///
    /// cost: O(K / 64)
    /// empty: `None`
    /// ties: None possible; positions are distinct
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).first() == Some(D(1))`
    pub fn first(self) -> ?D = self.find(_ => true)

    /// The smallest position in the set satisfying `pred`, or `None`, as `seq.find`. Members are
    /// visited in increasing order, so `pred` is called only up to the answer.
    ///
    /// cost: O(K / 64 + visited)
    /// empty: `None`
    /// ties: None possible; the smallest qualifying position wins
    /// traps: never (beyond what `pred` traps on)
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).find(d => Int(d) > 2) == Some(D(3))`
    pub fn find(self, pred: D -> Bool) -> ?D:
        for k in 0..w.len:
            var x = w[k]
            while x != U64(0):
                let b = ctz(x)
                let v = D.nth(64 * k + b)
                if pred(v): return v
                x &= ~bit(b)
        None

    /// The largest position in the set satisfying `pred`, or `None`. Members are
    /// visited in decreasing order.
    ///
    /// cost: O(K / 64 + visited)
    /// empty: `None`
    /// ties: None possible; the largest qualifying position wins
    /// traps: never (beyond what `pred` traps on)
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).rfind(d => Int(d) < 8) == Some(D(3))`
    pub fn rfind(self, pred: D -> Bool) -> ?D:
        for v in self.each_rev():
            if pred(v): return v
        None

    /// Replace `self` with `self | o` in place, with no allocation.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.single(D(1)); b.union(Bitset<D>.single(D(2))); b.len == 2`
    pub fn union(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] |= o.w[k]
        self.recount()

    /// Replace `self` with `self & o` in place, with no allocation.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(2)]); b.intersect(Bitset<D>.single(D(2))); b.each().collect() == [D(2)]`
    pub fn intersect(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] &= o.w[k]
        self.recount()

    /// Clear every bit set in `o` from `self` in place: `self - o`.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(2)]); b.subtract(Bitset<D>.single(D(2))); b.each().collect() == [D(1)]`
    pub fn subtract(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] &= ~o.w[k]
        self.recount()

    /// Replace `self` with `self ^ o` in place.
    ///
    /// cost: O(K / 64)
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(2)]); b.symmetric(Bitset<D>.of([D(2), D(3)])); b.each().collect() == [D(1), D(3)]`
    pub fn symmetric(mut self, o: Bitset<D>):
        for k in 0..w.len: w[k] ^= o.w[k]
        self.recount()

    /// `a | b`: a new set of the positions in either set.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; (Bitset<D>.single(D(1)) | Bitset<D>.single(D(2))).len == 2`
    pub fn bitor(self, o: Bitset<D>) -> Bitset<D>:
        var out = Bitset<D>.new()
        for k in 0..w.len: out.w[k] = w[k] | o.w[k]
        out.recount()
        out

    /// `a & b`: a new set of the positions in both sets.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; (Bitset<D>.of([D(1), D(2)]) & Bitset<D>.of([D(2), D(3)])).each().collect() == [D(2)]`
    pub fn bitand(self, o: Bitset<D>) -> Bitset<D>:
        var out = Bitset<D>.new()
        for k in 0..w.len: out.w[k] = w[k] & o.w[k]
        out.recount()
        out

    /// `a ^ b`: a new set of the positions in exactly one of the sets.
    ///
    /// cost: O(K / 64) time and space
    /// traps: never
    /// example: `domain D = 0..10; (Bitset<D>.of([D(1), D(2)]) ^ Bitset<D>.of([D(2), D(3)])).each().collect() == [D(1), D(3)]`
    pub fn bitxor(self, o: Bitset<D>) -> Bitset<D>:
        var out = Bitset<D>.new()
        for k in 0..w.len: out.w[k] = w[k] ^ o.w[k]
        out.recount()
        out

    /// Do the two sets share a position? Stops at the first shared word; `(a & b).len > 0` without
    /// building `a & b`.
    ///
    /// cost: O(K / 64), less on an early hit
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(2)]).meets(Bitset<D>.of([D(2), D(3)]))`
    /// example: `domain D = 0..10; not Bitset<D>.single(D(1)).meets(Bitset<D>.single(D(2)))`
    pub fn meets(self, o: Bitset<D>) -> Bool:
        for k in 0..w.len:
            if (w[k] & o.w[k]) != 0: return true
        false

    /// A new set with every position `k` places up (down for a negative `k`); positions that leave `D`
    /// are dropped. Position `v` moves to the position whose ordinal is `v`'s plus `k`.
    ///
    /// cost: O(K / 64) time and space, for any `k`
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).shifted(2).each().collect() == [D(3), D(5)]`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).shifted(-2).each().collect() == [D(1), D(6)]`
    pub fn shifted(self, k: Int) -> Bitset<D>:
        var out = Bitset<D>.new()
        if k < 0:
            let d = -k
            let (words, bits) = (d / 64, d % 64)
            if words >= w.len: return out
            for i in 0..w.len - words:
                var x = w[i + words] >> bits
                if bits > 0 and i + words + 1 < w.len:
                    x |= w[i + words + 1] << (64 - bits)
                out.w[i] = x
            out.recount()
            return out
        let (words, bits) = (k / 64, k % 64)
        for i in (words..w.len).rev:
            var x = w[i - words] << bits
            if bits > 0 and i > words: x |= w[i - words - 1] >> (64 - bits)
            out.w[i] = x
        out.trim()
        out.recount()
        out

    /// Shift every position `k >= 0` places up in place; bits past `D` drop.
    ///
    /// cost: O(K / 64)
    /// traps: when `k < 0`
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.shift_up(1); b.each().collect() == [D(2), D(4), D(9)]`
    pub fn shift_up(mut self, k: Int):
        assert k >= 0
        if k == 0: return
        let (words, bits) = (k / 64, k % 64)
        if words >= w.len:
            self.clear()
            return
        for i in (words..w.len).rev:
            var x = w[i - words] << bits
            if bits > 0 and i > words: x |= w[i - words - 1] >> (64 - bits)
            w[i] = x
        for i in 0..words: w[i] = U64(0)
        self.trim()
        self.recount()

    /// In-place `self |= self << k` for `k >= 0`, with zero allocation: the knapsack step "every
    /// reachable sum, and every reachable sum plus `k`". Bits past `D` drop.
    ///
    /// cost: O(K / 64)
    /// traps: when `k < 0`
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.shift_or(2); b.each().collect() == [D(1), D(3), D(5), D(8)]`
    pub fn shift_or(mut self, k: Int):
        assert k >= 0
        if k == 0: return
        let (words, bits) = (k / 64, k % 64)
        if words >= w.len: return
        for i in (words..w.len).rev:
            var x = w[i - words] << bits
            if bits > 0 and i > words: x |= w[i - words - 1] >> (64 - bits)
            w[i] |= x
        self.trim()
        self.recount()

    /// The positions in the set, in increasing order; `for v in b` walks the same sequence.
    ///
    /// cost: O(K / 64 + len) to walk, O(1) per member amortized
    /// empty: yields nothing
    /// order: increasing
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(8), D(1)]).each().collect() == [D(1), D(8)]`
    pub fn each(self) -> yields D:
        for k in 0..w.len:
            var x = w[k]
            while x != 0:
                let b = ctz(x)
                yield D.nth(64 * k + b)
                x &= ~bit(b)

    /// The first position in the set at or after `i`, or `None`.
    ///
    /// cost: O(K / 64)
    /// empty: `None`
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).next(D(3)) == Some(D(3))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).next(D(4)) == Some(D(8))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).next(D(9)) == None`
    pub fn next(self, i: D) -> ?D:
        var k = i.to_idx / 64
        var x = w[k] & ~low(i.to_idx % 64)
        while true:
            if x != 0: return D.nth(64 * k + ctz(x))
            k += 1
            if k == w.len: return None
            x = w[k]

    /// The last position in the set at or before `i`, or `None` (the mirror of `next`).
    ///
    /// cost: O(K / 64)
    /// empty: `None`
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).prev(D(3)) == Some(D(3))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).prev(D(2)) == Some(D(1))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).prev(D(0)) == None`
    pub fn prev(self, i: D) -> ?D:
        let idx = i.to_idx
        if idx == 0: return None
        var k = idx / 64
        var x = w[k] & low(idx % 64)
        while true:
            if x != 0: return D.nth(64 * k + msb(x))
            if k == 0: return None
            k -= 1
            x = w[k]

    /// The number of positions in the set strictly before `i`.
    ///
    /// cost: O(K / 64)
    /// empty: zero
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).count_below(D(8)) == 2`
    pub fn count_below(self, i: D) -> Int:
        let idx = i.to_idx
        let (words, bits) = (idx / 64, idx % 64)
        var ans = 0
        for k in 0..words: ans += popcount(w[k])
        if bits > 0 and words < w.len: ans += popcount(w[words] & low(bits))
        ans

    /// All subset sums in `0..D.K` reachable from non-negative `weights`: the position with ordinal `s`
    /// is set exactly when some sub-multiset of `weights` sums to `s`. The empty sub-multiset makes `0`
    /// reachable; sums of `D.K` or more are dropped.
    /// Groups duplicate weights with binary carries so the work is O(K sqrt(S) / 64).
    ///
    /// cost: O(K sqrt(S) / 64) time, where S is the sum of `weights`; O(K / 64) space
    /// empty: `{D.first}` (just the sum 0), or the empty set when `D` is empty
    /// traps: when a weight is negative
    /// example: `domain S = 0..=10; Bitset<S>.subset_sums([3, 5, 5]).each().map(s => Int(s)).collect() == [0, 3, 5, 8, 10]`
    pub fn subset_sums(weights: List<Int>) -> Bitset<D>:
        var reach = Bitset<D>.new()
        if D.K == 0: return reach
        reach.add(D.nth(0))
        var cnt = List.filled(D.K, 0)
        for w in weights:
            if w > 0 and w < D.K: cnt[w] += 1
        for w in 1..D.K:
            let c = cnt[w]
            if c > 2:
                let carry = (c - 1) / 2
                cnt[w] -= 2 * carry
                if 2 * w < D.K: cnt[2 * w] += carry
            for _ in 0..cnt[w]: reach.shift_or(w)
        reach

    /// Shift all bits down by `k >= 0` positions in place (`i -> i - k`); bits below `D` drop.
    ///
    /// cost: O(K / 64)
    /// traps: when `k < 0`
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.shift_down(1); b.each().collect() == [D(0), D(2), D(7)]`
    pub fn shift_down(mut self, k: Int):
        assert k >= 0
        let s = self.shifted(-k)
        w = s.w
        n = s.n

    /// Count set bits in `r` (accepts cut ranges via decision 124).
    ///
    /// cost: O(r.len / 64 + 1)
    /// empty: zero
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).count_range(1..8) == 2`
    pub fn count_range(self, r: Range<D>) -> Int:
        var c = 0
        let (lo, hi) = (r.start.to_idx, r.end.to_idx)
        if lo >= hi: return 0
        for k in lo / 64..=(hi - 1) / 64: c += popcount(w[k] & span(k, lo, hi))
        c

    /// Set (or, with `on: false`, clear) every bit in `r` in place.
    ///
    /// cost: O(r.len / 64 + 1)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.set_range(2..5); b.len == 5`
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.set_range(0..4, on: false); b.each().collect() == [D(8)]`
    pub fn set_range(mut self, r: Range<D>, on: Bool = true):
        let (lo, hi) = (r.start.to_idx, r.end.to_idx)
        if lo >= hi: return
        for k in lo / 64..=(hi - 1) / 64:
            let m = span(k, lo, hi)
            n -= popcount(w[k] & m)
            if on: w[k] |= m else: w[k] &= ~m
            n += popcount(w[k] & m)

    /// Flip every bit in `r` in place.
    ///
    /// cost: O(r.len / 64 + 1)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..10; var b = Bitset<D>.of([D(1), D(3), D(8)]); b.flip_range(0..4); b.each().collect() == [D(0), D(2), D(8)]`
    pub fn flip_range(mut self, r: Range<D>):
        let (lo, hi) = (r.start.to_idx, r.end.to_idx)
        if lo >= hi: return
        for k in lo / 64..=(hi - 1) / 64:
            let m = span(k, lo, hi)
            n -= popcount(w[k] & m)
            w[k] ^= m
            n += popcount(w[k] & m)

    /// The `k`-th set position (0-based, in increasing order), or `None` if `k >= len`.
    ///
    /// cost: O(K / 64)
    /// empty: `None`
    /// traps: when `k < 0`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).nth(2) == Some(D(8))`
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).nth(3) == None`
    pub fn nth(self, k: Int) -> ?D:
        assert k >= 0
        var left = k
        for i in 0..w.len:
            let c = popcount(w[i])
            if left < c:
                var x = w[i]
                for _ in 0..left: x &= x - U64(1)
                return D.nth(64 * i + ctz(x))
            left -= c
        None

    /// The set positions in descending order.
    ///
    /// cost: O(K / 64 + len) to walk
    /// empty: yields nothing
    /// order: decreasing
    /// traps: never
    /// example: `domain D = 0..10; Bitset<D>.of([D(1), D(3), D(8)]).each_rev().collect() == [D(8), D(3), D(1)]`
    pub fn each_rev(self) -> yields D:
        for k in (0..w.len).rev:
            var x = w[k]
            while x != U64(0):
                let b = msb(x)
                yield D.nth(64 * k + b)
                x &= ~bit(b)

    fn recount(mut self):
        n = 0
        for x in w: n += popcount(x)

    fn trim(mut self):
        let full_words = (D.K + 63) / 64
        for k in full_words..w.len: w[k] = U64(0)
        if D.K % 64 != 0 and full_words > 0:
            w[full_words - 1] &= low(D.K % 64)

/// Runtime-sized 64-bit word bitset over `0..cap` when the width is computed dynamically without a domain.
/// Single-bit ops are O(1); word-parallel ops are O(cap / 64).
pub struct DynBitset:
    _w: List<U64>
    _cap: Int
    _n: Int

    fn recount(mut self):
        _n = 0
        for x in _w: _n += popcount(x)

    fn trim(mut self):
        let full_words = _cap / 64
        if _cap % 64 != 0: _w[full_words] &= low(_cap % 64)

    /// Create an all-zero `DynBitset` of width `cap`: bits `0..cap`.
    ///
    /// cost: O(cap / 64) time and space
    /// traps: when `cap < 0`
    /// example: `DynBitset(10).len == 0`
    pub fn new(cap: Int) -> DynBitset:
        assert cap >= 0
        DynBitset(List.filled((cap + 63) / 64, U64(0)), cap, 0)

    /// Create an all-one `DynBitset` of width `cap`; bits at or above `cap` stay clear.
    ///
    /// cost: O(cap / 64) time and space
    /// traps: when `cap < 0`
    /// example: `DynBitset.full(10).len == 10`
    pub fn full(cap: Int) -> DynBitset:
        assert cap >= 0
        var d = DynBitset(List.filled((cap + 63) / 64, ~U64(0)), cap, cap)
        d.trim()
        d

    /// The bit capacity: valid indices are `0..cap`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `DynBitset(10).cap == 10`
    @get pub fn cap(self) -> Int = _cap

    /// The number of set bits, kept current by every update.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.len == 2`
    @get pub fn len(self) -> Int = _n

    /// Whether no bits are set, kept current by every update.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `DynBitset(10).is_empty and not DynBitset.full(1).is_empty`
    @get pub fn is_empty(self) -> Bool = _n == 0

    /// Whether bit `i` is set.
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.add(3); d.contains(3) and not d.contains(4)`
    pub fn contains(self, i: Int) -> Bool:
        assert 0 <= i and i < _cap
        (_w[i / 64] >> (i % 64)) & U64(1) == U64(1)

    /// Set bit `i` to `on` in place (W8 `IndexSet` backing `b[i] = on`).
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.set(4); d.set(5, true); d.set(4, false); d.each().collect() == [5]`
    pub fn set(mut self, i: Int, on: Bool = true):
        if on: self.add(i) else: self.remove(i)

    /// Set bit `i` to 1; returns `true` if bit `i` was previously 0 (matches `Bitset.add`).
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); let x = d.add(2); let y = d.add(2); x and not y`
    pub fn add(mut self, i: Int) -> Bool:
        if self.contains(i): return false
        _w[i / 64] |= bit(i % 64)
        _n += 1
        true

    /// Clear bit `i` to 0; returns `true` if bit `i` was previously 1 (matches `Bitset.remove`).
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.add(2); let x = d.remove(2); let y = d.remove(2); x and not y and d.is_empty`
    pub fn remove(mut self, i: Int) -> Bool:
        if not self.contains(i): return false
        _w[i / 64] &= ~bit(i % 64)
        _n -= 1
        true

    /// Flip bit `i`.
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.toggle(6); d.toggle(6); d.toggle(7); d.each().collect() == [7]`
    pub fn toggle(mut self, i: Int):
        _n += if self.contains(i): -1 else: 1
        _w[i / 64] ^= bit(i % 64)

    /// Clear all bits; the capacity is unchanged.
    ///
    /// cost: O(cap / 64)
    /// traps: never
    /// example: `var d = DynBitset.full(10); d.clear(); d.is_empty and d.cap == 10`
    pub fn clear(mut self):
        for k in 0.._w.len: _w[k] = U64(0)
        _n = 0

    /// Resize to `new_cap` bits, zero-filling any new high bits. Shrinking drops the bits at or above
    /// `new_cap` (and `len` drops with them).
    ///
    /// cost: O(new_cap / 64) time and space
    /// traps: when `new_cap < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(8); d.resize(4); d.len == 1 and d.cap == 4`
    /// example: `var d = DynBitset.full(2); d.resize(5); d.each().collect() == [0, 1]`
    pub fn resize(mut self, new_cap: Int):
        assert new_cap >= 0
        let words = (new_cap + 63) / 64
        while _w.len > words: _w.pop()
        while _w.len < words: _w.push(U64(0))
        _cap = new_cap
        self.trim()
        self.recount()

    /// The smallest set bit index, or `None` if empty.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// ties: None possible; the smallest index wins
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(8); d.add(3); d.first() == Some(3)`
    /// example: `DynBitset(10).first() == None`
    pub fn first(self) -> ?Int = self.next(0)

    /// The largest set bit index, or `None` if empty.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// ties: None possible; the largest index wins
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(8); d.add(3); d.last() == Some(8)`
    pub fn last(self) -> ?Int:
        if _cap == 0: return None
        self.prev(_cap - 1)

    /// The smallest set bit `>= i`, or `None`.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// traps: when `i` is not in `0..=cap`
    /// example: `var d = DynBitset(10); d.add(3); d.add(8); d.next(3) == Some(3) and d.next(4) == Some(8) and d.next(9) == None`
    pub fn next(self, i: Int) -> ?Int:
        assert 0 <= i and i <= _cap
        if i == _cap: return None
        var k = i / 64
        var x = _w[k] & ~low(i % 64)
        while true:
            if x != U64(0): return 64 * k + ctz(x)
            k += 1
            if k == _w.len: return None
            x = _w[k]
        None

    /// The largest set bit `<= i`, or `None`; `i` itself counts, as in `Bitset.prev`.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// traps: when `i` is not in `0..cap`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.prev(3) == Some(3) and d.prev(2) == Some(1) and d.prev(0) == None`
    pub fn prev(self, i: Int) -> ?Int:
        assert 0 <= i and i < _cap
        var k = i / 64
        var x = _w[k] & low(i % 64 + 1)
        while true:
            if x != U64(0): return 64 * k + msb(x)
            if k == 0: return None
            k -= 1
            x = _w[k]
        None

    /// The number of set bits strictly below `i`.
    ///
    /// cost: O(cap / 64)
    /// empty: zero
    /// traps: when `i` is not in `0..=cap`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.count_below(8) == 2`
    pub fn count_below(self, i: Int) -> Int:
        assert 0 <= i and i <= _cap
        self.count_range(0..i)

    /// The number of set bits in `r`.
    ///
    /// cost: O(r.len / 64 + 1)
    /// empty: zero
    /// traps: when `r` is non-empty and reaches outside `0..cap`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.count_range(1..8) == 2`
    pub fn count_range(self, r: Range<Int>) -> Int:
        let (lo, hi) = (r.start, r.end)
        if lo >= hi: return 0
        assert 0 <= lo and hi <= _cap
        var c = 0
        for k in lo / 64..=(hi - 1) / 64: c += popcount(_w[k] & span(k, lo, hi))
        c

    /// The `k`-th set bit index (0-based, in increasing order), or `None` if `k >= len`.
    ///
    /// cost: O(cap / 64)
    /// empty: `None`
    /// traps: when `k < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.nth(1) == Some(3) and d.nth(3) == None`
    pub fn nth(self, k: Int) -> ?Int:
        assert k >= 0
        var left = k
        for i in 0.._w.len:
            let c = popcount(_w[i])
            if left < c:
                var x = _w[i]
                for _ in 0..left: x &= x - U64(1)
                return 64 * i + ctz(x)
            left -= c
        None

    /// In-place bitwise OR (`self |= other`).
    ///
    /// cost: O(cap / 64)
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset(4); a.add(0); var b = DynBitset(4); b.add(2); a.union(b); a.each().collect() == [0, 2]`
    pub fn union(mut self, other: DynBitset):
        assert other.cap == _cap
        for k in 0.._w.len: _w[k] |= other._w[k]
        self.recount()

    /// In-place bitwise AND (`self &= other`).
    ///
    /// cost: O(cap / 64)
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset.full(4); var b = DynBitset(4); b.add(2); a.intersect(b); a.each().collect() == [2]`
    pub fn intersect(mut self, other: DynBitset):
        assert other.cap == _cap
        for k in 0.._w.len: _w[k] &= other._w[k]
        self.recount()

    /// In-place set difference (`self &= ~other`).
    ///
    /// cost: O(cap / 64)
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset.full(4); var b = DynBitset(4); b.add(2); a.subtract(b); a.each().collect() == [0, 1, 3]`
    pub fn subtract(mut self, other: DynBitset):
        assert other.cap == _cap
        for k in 0.._w.len: _w[k] &= ~other._w[k]
        self.recount()

    /// In-place bitwise XOR (`self ^= other`).
    ///
    /// cost: O(cap / 64)
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset(4); a.add(1); a.add(2); var b = DynBitset(4); b.add(2); b.add(3); a.symmetric(b); a.each().collect() == [1, 3]`
    pub fn symmetric(mut self, other: DynBitset):
        assert other.cap == _cap
        for k in 0.._w.len: _w[k] ^= other._w[k]
        self.recount()

    /// Whether `self` and `other` share a set bit; stops at the first shared word.
    ///
    /// cost: O(cap / 64), less on an early hit
    /// traps: when `other.cap != self.cap`
    /// example: `var a = DynBitset(4); a.add(1); var b = DynBitset.full(4); a.meets(b) and not a.meets(DynBitset(4))`
    pub fn meets(self, other: DynBitset) -> Bool:
        assert other.cap == _cap
        for k in 0.._w.len:
            if _w[k] & other._w[k] != U64(0): return true
        false

    /// Shift all bits up by `k >= 0` in place (`i -> i + k`); bits reaching `cap` or beyond drop.
    ///
    /// cost: O(cap / 64)
    /// traps: when `k < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.shift_up(2); d.each().collect() == [3, 5]`
    pub fn shift_up(mut self, k: Int):
        assert k >= 0
        let n = _w.len
        let (words, bits) = (k / 64, k % 64)
        for i in (0..n).rev:
            var x = U64(0)
            if i >= words:
                x = _w[i - words] << bits
                if bits > 0 and i > words: x |= _w[i - words - 1] >> (64 - bits)
            _w[i] = x
        self.trim()
        self.recount()

    /// Shift all bits down by `k >= 0` in place (`i -> i - k`); bits falling below 0 drop.
    ///
    /// cost: O(cap / 64)
    /// traps: when `k < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.shift_down(2); d.each().collect() == [1, 6]`
    pub fn shift_down(mut self, k: Int):
        assert k >= 0
        let n = _w.len
        let (words, bits) = (k / 64, k % 64)
        for i in 0..n:
            var x = U64(0)
            if i + words < n:
                x = _w[i + words] >> bits
                if bits > 0 and i + words + 1 < n: x |= _w[i + words + 1] << (64 - bits)
            _w[i] = x
        self.recount()

    /// In-place `self |= (self << k)` for `k >= 0`, with zero allocation; bits reaching `cap` drop.
    ///
    /// cost: O(cap / 64)
    /// traps: when `k < 0`
    /// example: `var d = DynBitset(10); d.add(1); d.add(3); d.add(8); d.shift_or(2); d.each().collect() == [1, 3, 5, 8]`
    pub fn shift_or(mut self, k: Int):
        assert k >= 0
        if k == 0: return
        let n = _w.len
        let (words, bits) = (k / 64, k % 64)
        if words >= n: return
        for i in (words..n).rev:
            var x = _w[i - words] << bits
            if bits > 0 and i > words: x |= _w[i - words - 1] >> (64 - bits)
            _w[i] |= x
        self.trim()
        self.recount()

    /// The set bit indices in ascending order; `for i in d` walks the same sequence.
    ///
    /// cost: O(cap / 64 + len) to walk
    /// empty: yields nothing
    /// order: increasing
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(8); d.add(1); d.each().collect() == [1, 8]`
    pub fn each(self) -> yields Int:
        for k in 0.._w.len:
            var x = _w[k]
            while x != U64(0):
                let b = ctz(x)
                yield 64 * k + b
                x &= ~bit(b)

    /// The set bit indices in descending order.
    ///
    /// cost: O(cap / 64 + len) to walk
    /// empty: yields nothing
    /// order: decreasing
    /// traps: never
    /// example: `var d = DynBitset(10); d.add(8); d.add(1); d.each_rev().collect() == [8, 1]`
    pub fn each_rev(self) -> yields Int:
        for k in (0.._w.len).rev:
            var x = _w[k]
            while x != U64(0):
                let b = msb(x)
                yield 64 * k + b
                x &= ~bit(b)

    /// All subset sums in `0..cap` reachable from `weights`: bit `s` is set exactly when some
    /// sub-multiset of `weights` sums to `s`. The empty sub-multiset makes `0` reachable (when
    /// `cap > 0`); sums of `cap` or more drop. Duplicate weights are grouped with binary carries, as in
    /// `Bitset.subset_sums`.
    ///
    /// cost: O(cap sqrt(S) / 64) time, where S is the sum of `weights`; O(cap / 64) space
    /// empty: just bit 0 (None when `cap == 0`)
    /// traps: when `cap < 0` or a weight is negative
    /// example: `DynBitset.subset_sums(11, [3, 5, 5]).each().collect() == [0, 3, 5, 8, 10]`
    pub fn subset_sums(cap: Int, weights: List<Int>) -> DynBitset:
        var reach = DynBitset.new(cap)
        if cap == 0: return reach
        reach.add(0)
        var cnt = List.filled(cap, 0)
        for w in weights:
            if w > 0 and w < cap: cnt[w] += 1
        for w in 1..cap:
            let c = cnt[w]
            if c > 2:
                let carry = (c - 1) / 2
                cnt[w] -= 2 * carry
                if 2 * w < cap: cnt[2 * w] += carry
            for _ in 0..cnt[w]: reach.shift_or(w)
        reach

/// O(bits) per op. Dynamic binary bitwise XOR trie multiset over non-negative `bits`-bit integers (default 60 bits).
pub struct XorTrie:
    _bits: Int
    _mask: Int
    _c0: List<Int>
    _c1: List<Int>
    _cnt: List<Int>
    _dist: List<Int>

    fn child(self, node: Int, b: Int) -> Int = if b == 0: _c0[node] else: _c1[node]

    fn leaf(self, x: Int) -> Int:
        let z = x ^ _mask
        var node = 0
        for i in (0.._bits).rev:
            node = self.child(node, (z >> i) & 1)
            if node < 0: return -1
        node

    fn walk(mut self, x: Int, d: Int, dd: Int):
        let z = x ^ _mask
        var node = 0
        _cnt[0] += d
        _dist[0] += dd
        for i in (0.._bits).rev:
            let b = (z >> i) & 1
            var c = self.child(node, b)
            if c < 0:
                c = _cnt.len
                _c0.push(-1)
                _c1.push(-1)
                _cnt.push(0)
                _dist.push(0)
                if b == 0: _c0[node] = c else: _c1[node] = c
            node = c
            _cnt[node] += d
            _dist[node] += dd

    /// Create an empty `XorTrie` over `bits`-bit non-negative integers (default 60): values in
    /// `0..2^bits`.
    ///
    /// cost: O(1)
    /// traps: when `bits` is not in `1..=62`
    /// example: `XorTrie(3).is_empty`
    pub fn new(bits: Int = 60) -> XorTrie:
        assert bits >= 1 and bits <= 62
        XorTrie(bits, 0, [-1], [-1], [0], [0])

    /// Reserve pool capacity for at least `nodes` trie nodes, so later adds do not reallocate; the
    /// contents are unchanged.
    ///
    /// cost: O(nodes) time and space
    /// traps: when `nodes < 0`
    /// example: `var t = XorTrie(3); t.reserve(100); t.len == 0`
    pub fn reserve(mut self, nodes: Int):
        assert nodes >= 0

    /// The total number of stored integers, with multiplicity.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.add(5); t.len == 2`
    @get pub fn len(self) -> Int = _cnt[0]

    /// True when the trie holds no integers.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.remove(5); t.is_empty`
    @get pub fn is_empty(self) -> Bool = _cnt[0] == 0

    /// Insert one copy of `x`. `x` is stored as given, after any earlier `xor_all`: the mask applies
    /// only to elements already present.
    ///
    /// cost: O(bits) time, up to `bits` new nodes
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(5); t.add(5); t.count(5) == 2`
    pub fn add(mut self, x: Int):
        assert 0 <= x and x < 1 << _bits
        self.walk(x, 1, if self.count(x) == 0: 1 else: 0)

    /// Remove one copy of `x`; returns `false` if absent, leaving the trie unchanged.
    ///
    /// cost: O(bits)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); let x = t.remove(5); let y = t.remove(5); x and not y`
    pub fn remove(mut self, x: Int) -> Bool:
        let c = self.count(x)
        if c == 0: return false
        self.walk(x, -1, if c == 1: -1 else: 0)
        true

    /// Whether `x` is present at least once.
    ///
    /// cost: O(bits)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.contains(5) and not t.contains(4)`
    pub fn contains(self, x: Int) -> Bool = self.count(x) > 0

    /// The multiplicity of `x` currently stored; zero when absent.
    ///
    /// cost: O(bits)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.add(5); t.add(1); t.count(5) == 2 and t.count(2) == 0`
    pub fn count(self, x: Int) -> Int:
        if x < 0 or x >= 1 << _bits: return 0
        let node = self.leaf(x)
        if node < 0: 0 else: _cnt[node]

    /// Apply a lazy global XOR mask: every stored `y` becomes `y ^ mask`. Masks compose (`xor_all(a)`
    /// then `xor_all(b)` is `xor_all(a ^ b)`).
    ///
    /// cost: O(1)
    /// traps: when `mask` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.xor_all(1); t.contains(0) and t.contains(4) and not t.contains(1)`
    pub fn xor_all(mut self, mask: Int):
        assert 0 <= mask and mask < 1 << _bits
        _mask ^= mask

    /// The maximum value of `x ^ y` over stored `y`, or `None` if empty.
    ///
    /// cost: O(bits)
    /// empty: `None`
    /// ties: None possible; the value is unique even when several `y` reach it
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.max_xor(2) == Some(7)`
    /// example: `XorTrie(3).max_xor(2) == None`
    pub fn max_xor(self, x: Int) -> ?Int = self.extreme_xor(x, 1)

    /// The minimum value of `x ^ y` over stored `y`, or `None` if empty.
    ///
    /// cost: O(bits)
    /// empty: `None`
    /// ties: None possible; the value is unique even when several `y` reach it
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.min_xor(2) == Some(3)`
    pub fn min_xor(self, x: Int) -> ?Int = self.extreme_xor(x, 0)

    /// The number of stored `y` (with multiplicity) with `(x ^ y) < limit`. A `limit <= 0` gives 0 and
    /// a `limit >= 2^bits` gives `len`.
    ///
    /// cost: O(bits)
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.count_xor_less(2, 5) == 2`
    pub fn count_xor_less(self, x: Int, limit: Int) -> Int:
        assert 0 <= x and x < 1 << _bits
        if limit <= 0: return 0
        if limit >= 1 << _bits: return _cnt[0]
        let z = x ^ _mask
        var node = 0
        var out = 0
        for i in (0.._bits).rev:
            let zb = (z >> i) & 1
            if (limit >> i) & 1 == 1:
                let same = self.child(node, zb)
                if same >= 0: out += _cnt[same]
                node = self.child(node, 1 - zb)
            else:
                node = self.child(node, zb)
            if node < 0: return out
        out

    /// The `k`-th smallest value of `x ^ y` over stored `y` (0-based, with multiplicity), or `None` if
    /// `k >= len`.
    ///
    /// cost: O(bits)
    /// empty: `None`
    /// ties: equal values `x ^ y` (copies of one `y`) each take their own rank
    /// traps: when `k < 0` or `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.kth_xor(2, 1) == Some(4)`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.kth_xor(2, 3) == None`
    pub fn kth_xor(self, x: Int, k: Int) -> ?Int:
        assert k >= 0 and 0 <= x and x < 1 << _bits
        if k >= _cnt[0]: return None
        let z = x ^ _mask
        var node = 0
        var left = k
        var res = 0
        for i in (0.._bits).rev:
            let zb = (z >> i) & 1
            let same = self.child(node, zb)
            let c = if same >= 0: _cnt[same] else: 0
            if left < c:
                node = same
            else:
                left -= c
                res |= 1 << i
                node = self.child(node, 1 - zb)
        Some(res)

    /// The smallest non-negative integer not present under XOR mask `x` (mex of `{y ^ x}`); `2^bits`
    /// when every value is present.
    ///
    /// cost: O(bits)
    /// empty: zero
    /// traps: when `x` is not in `0..2^bits`
    /// example: `var t = XorTrie(3); t.add(1); t.add(5); t.add(6); t.mex_xor() == 0 and t.mex_xor(1) == 1`
    pub fn mex_xor(self, x: Int = 0) -> Int:
        assert 0 <= x and x < 1 << _bits
        let z = x ^ _mask
        var node = 0
        var res = 0
        for i in (0.._bits).rev:
            let zb = (z >> i) & 1
            let same = self.child(node, zb)
            if same >= 0 and _dist[same] == 1 << i:
                res |= 1 << i
                node = self.child(node, 1 - zb)
            else:
                node = same
            if node < 0: return res
        res

    fn extreme_xor(self, x: Int, want: Int) -> ?Int:
        assert 0 <= x and x < 1 << _bits
        if _cnt[0] == 0: return None
        let z = x ^ _mask
        var node = 0
        var res = 0
        for i in (0.._bits).rev:
            let zb = (z >> i) & 1
            let pref = self.child(node, zb ^ want)
            if pref >= 0 and _cnt[pref] > 0:
                node = pref
                res |= want << i
            else:
                node = self.child(node, zb ^ want ^ 1)
                res |= (want ^ 1) << i
        Some(res)

    /// Remove all elements and the global mask, keeping `bits` and the reserved pool.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = XorTrie(3); t.add(5); t.clear(); t.is_empty and not t.contains(5)`
    pub fn clear(mut self):
        _mask = 0
        _c0 = [-1]
        _c1 = [-1]
        _cnt = [0]
        _dist = [0]

/// `a - b`: the positions in `a` that are not in `b`.
impl Sub for Bitset<D>:
    fn sub(a, b):
        var out = Bitset<D>.new()
        for k in 0..a.w.len: out.w[k] = a.w[k] & ~b.w[k]
        out

/// Index of the highest set bit in non-zero `x` (0..64).
fn msb(x: U64) -> Int = bit_width(x) - 1

impl Show for Bitset<D>:
    fn show(a):
        var out = ""
        for v in a.each():
            if not out.is_empty: out += " "
            out += "{v}"
        out
/// The word with only bit `k` set.
fn bit(k: Int) -> U64 = U64(1) << k

fn span(k: Int, lo: Int, hi: Int) -> U64 =
    low(min(hi - 64 * k, 64)) & ~low(max(lo - 64 * k, 0))

/// The word with its low `n` bits set, for `n` in 0..=64.
fn low(n: Int) -> U64 = if n == 64: ~U64(0) else: (U64(1) << n) - U64(1)

@test
fn single_bits_across_word_borders():
    domain P = 0..200
    var b = Bitset<P>()
    for i in [0, 1, 62, 63, 64, 127, 128, 199]: b.add(P(i))
    assert b.len == 8 and b.count == 8
    assert b.contains(P(63)) and b.contains(P(64)) and not b.contains(P(65))
    b.remove(P(63))
    b.toggle(P(65))
    b.toggle(P(0))
    assert [p.to_idx for p in b.each()] == [1, 62, 64, 65, 127, 128, 199]
    assert b.next(P(2)) == P(62) and b.next(P(129)) == P(199) and b.next(P(64)) == P(64)
    b.remove(P(199))
    assert b.next(P(129)) == None

@test
fn set_algebra_matches_bool_arrays():
    domain P = 0..150
    let x = [P] p => rand(0, 2) == 0
    let y = [P] p => rand(0, 2) == 0
    let (bx, by) = (Bitset<P>.from(x), Bitset<P>.from(y))
    var u = copy(bx)
    u.union(by)
    var n = copy(bx)
    n.intersect(by)
    var d = copy(bx)
    d.subtract(by)
    var s = copy(bx)
    s.symmetric(by)
    for p in P:
        assert u.contains(p) == (x[p] or y[p])
        assert n.contains(p) == (x[p] and y[p])
        assert d.contains(p) == (x[p] and not y[p])
        assert s.contains(p) == (x[p] != y[p])
    assert bx.meets(by) != n.is_empty
    assert bx.len == sum(1 for p in P if x[p])

@test
fn shifted_matches_moving_each_position():
    domain P = 0..190
    let x = [P] p => rand(0, 3) == 0
    let b = Bitset<P>.from(x)
    for k in [0, 1, 7, 63, 64, 65, 128, 189, 190, 400]:
        let s = b.shifted(k)
        for p in P:
            let from = p.to_idx - k
            assert s.contains(p) == (from >= 0 and x[P.nth(max(from, 0))])

@test
fn subset_sum_by_shifting():
    domain S = 0..=100
    var reach = Bitset<S>()
    reach.add(S(0))
    for wt in [7, 13, 29, 40]: reach.union(reach.shifted(wt))
    assert reach.contains(S(20)) and reach.contains(S(89)) and reach.contains(S(49))
    assert not reach.contains(S(1)) and not reach.contains(S(30))
    assert reach.len == 16

@test
fn membership_and_a_signed_shift():
    domain D = 0..130
    var s = Bitset<D>()
    for x in [0, 3, 64, 129]: s.add(D(x))
    assert D(64) in s and D(65) not in s and s.len == 4
    s.toggle(D(3))
    s.toggle(D(5))
    assert [i.to_idx for i in s.shifted(-3).each()] == [2, 61, 126]
    assert [i.to_idx for i in s.shifted(2).each()] == [2, 7, 66]

@test
fn full_contains_every_position_and_nothing_past_end():
    for sz in [1, 63, 64, 65, 130]:
        domain D = 0..sz
        let f = Bitset<D>.full()
        assert f.len == sz
        for x in D: assert x in f
        assert (f & Bitset<D>.new()).is_empty

@test
fn nav_helpers_and_in_place_shifts() = todo

@test
fn subset_sums_matches_naive_with_duplicates():
    domain S = 0..=150
    let ws = [1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 7, 7, 15, 40, 40, 200, 0]
    let fast = Bitset<S>.subset_sums(ws)
    var naive = Bitset<S>.new()
    naive.add(S(0))
    for w in ws:
        if w > 0: naive.union(naive.shifted(w))
    for s in S: assert fast.contains(s) == naive.contains(s)

@test
fn subset_parity_constructors_and_complement() = todo
