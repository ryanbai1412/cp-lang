//! Ordered maps and sets, as ordinary CP: `Map` and `Set` keep their keys in `<` order in an AVL
//! tree, \(O(\log n)\) a step. Core re-exports both, so no module imports them.
//!
//! start: `Map`, `Set`
//! example: `var m = Map<Str, Int>(); m["a"] += 2; m["a"] == 2`
use std.seq.{IndexGet, Seq, Sized}

/// A missing key traps (R07.12.9).
use std.raw.{check_key}

/// The balanced search tree under `Set` and `Map`: an AVL tree whose nodes are slots of its
/// lists, so a removed node's slot is reused. Keys are in `<` order, each once.
struct OrdTree<K, V>:
    var keys: List<K> = []
    var vals: List<V> = []
    var left: List<Int> = []
    var right: List<Int> = []
    var height: List<Int> = []
    var free: List<Int> = []
    var root: Int = -1
    var n: Int = 0

impl<K, V> OrdTree<K, V>:
    fn key_at(self, t: Int) -> ?K:
        if t < 0: None else: Some(keys[t])

    /// The node of the least key, or with `last` the greatest; -1 when empty.
    fn edge(self, last: Bool) -> Int:
        var t = root
        while t >= 0:
            let c = if last: right[t] else: left[t]
            if c < 0: return t
            t = c
        t

    /// Walks the nodes in key order.
    fn nodes(self) -> yields Int:
        var stack: List<Int> = []
        var t = root
        while t >= 0 or not stack.is_empty:
            while t >= 0:
                stack.push(t)
                t = left[t]
            let top = stack.pop() ?? -1
            yield top
            t = right[top]

    fn clear(mut self):
        keys.clear()
        vals.clear()
        left.clear()
        right.clear()
        height.clear()
        free.clear()
        root = -1
        n = 0

impl<K: Ord, V> OrdTree<K, V>:
    fn h(self, t: Int) -> Int:
        if t < 0: 0 else: height[t]

    fn fix(mut self, t: Int):
        let a = self.h(left[t])
        let b = self.h(right[t])
        height[t] = if a < b: b + 1 else: a + 1

    fn rot_right(mut self, t: Int) -> Int:
        let l = left[t]
        left[t] = right[l]
        right[l] = t
        self.fix(t)
        self.fix(l)
        l

    fn rot_left(mut self, t: Int) -> Int:
        let r = right[t]
        right[t] = left[r]
        left[r] = t
        self.fix(t)
        self.fix(r)
        r

    fn balance(mut self, t: Int) -> Int:
        self.fix(t)
        let b = self.h(left[t]) - self.h(right[t])
        if b > 1:
            let l = left[t]
            if self.h(left[l]) < self.h(right[l]):
                let c = self.rot_left(l)
                left[t] = c
            return self.rot_right(t)
        if b < -1:
            let r = right[t]
            if self.h(right[r]) < self.h(left[r]):
                let c = self.rot_right(r)
                right[t] = c
            return self.rot_left(t)
        t

    fn node(mut self, k: K, v: V) -> Int:
        n += 1
        if let i = free.pop():
            keys[i] = k
            vals[i] = v
            left[i] = -1
            right[i] = -1
            height[i] = 1
            return i
        keys.push(k)
        vals.push(v)
        left.push(-1)
        right.push(-1)
        height.push(1)
        keys.len - 1

    fn ins(mut self, t: Int, k: K, v: V) -> Int:
        if t < 0: return self.node(k, v)
        if k < keys[t]:
            let c = self.ins(left[t], k, v)
            left[t] = c
        elif keys[t] < k:
            let c = self.ins(right[t], k, v)
            right[t] = c
        else:
            vals[t] = v
            return t
        self.balance(t)

    /// Sets `k`'s value to `v`, adding `k` when it is absent.
    fn put(mut self, k: K, v: V):
        let r = self.ins(root, k, v)
        root = r

    /// The node holding `k`, or -1.
    fn find(self, k: K) -> Int:
        var t = root
        while t >= 0:
            if k < keys[t]: t = left[t]
            elif keys[t] < k: t = right[t]
            else: return t
        -1

    /// The node of the greatest key below `k` (`eq`: or equal to it), or -1.
    fn below(self, k: K, eq: Bool) -> Int:
        var t = root
        var best = -1
        while t >= 0:
            if keys[t] < k or (eq and not (k < keys[t])):
                best = t
                t = right[t]
            else:
                t = left[t]
        best

    /// The node of the least key above `k` (`eq`: or equal to it), or -1.
    fn above(self, k: K, eq: Bool) -> Int:
        var t = root
        var best = -1
        while t >= 0:
            if k < keys[t] or (eq and not (keys[t] < k)):
                best = t
                t = left[t]
            else:
                t = right[t]
        best

    fn cut_min(mut self, t: Int) -> Int:
        if left[t] < 0: return right[t]
        let c = self.cut_min(left[t])
        left[t] = c
        self.balance(t)

    fn cut_max(mut self, t: Int) -> Int:
        if right[t] < 0: return left[t]
        let c = self.cut_max(right[t])
        right[t] = c
        self.balance(t)

    /// Removes the node of the least key, or with `last` the greatest; that node, or -1. Its
    /// key and value stay readable until the next add.
    fn pop_edge(mut self, last: Bool) -> Int:
        let i = self.edge(last)
        if i < 0: return -1
        let r = if last: self.cut_max(root) else: self.cut_min(root)
        root = r
        n -= 1
        free.push(i)
        i

    fn del(mut self, t: Int, k: K) -> Int:
        if t < 0: return t
        if k < keys[t]:
            let c = self.del(left[t], k)
            left[t] = c
        elif keys[t] < k:
            let c = self.del(right[t], k)
            right[t] = c
        else:
            n -= 1
            free.push(t)
            if left[t] < 0: return right[t]
            if right[t] < 0: return left[t]
            var s = right[t]
            while left[s] >= 0: s = left[s]
            let c = self.cut_min(right[t])
            right[s] = c
            left[s] = left[t]
            return self.balance(s)
        self.balance(t)

    /// Removes `k`; whether it was there.
    fn drop(mut self, k: K) -> Bool:
        if self.find(k) < 0: return false
        let r = self.del(root, k)
        root = r
        true

/// An ordered set: its elements in `<` order, each once; `O(log n)` to add, remove and find one
/// (R07.2.4).
pub struct Set<T>:
    var t: OrdTree<T, ()> = OrdTree()

impl<T> Set<T>:
    @get pub fn len(self) -> Int:
        t.n

    @get pub fn is_empty(self) -> Bool:
        t.n == 0

    /// The least element.
    @get pub fn first(self) -> ?T:
        t.key_at(t.edge(false))

    /// The greatest element.
    @get pub fn last(self) -> ?T:
        t.key_at(t.edge(true))

    pub fn min(self) -> ?T:
        t.key_at(t.edge(false))

    pub fn max(self) -> ?T:
        t.key_at(t.edge(true))

    pub fn clear(mut self):
        t.clear()

    /// The elements in order.
    pub fn sorted(self) -> List<T>:
        var out: List<T> = []
        for i in t.nodes(): out.push(t.keys[i])
        out

impl<T: Ord> Set<T>:
    /// Adds `x`; whether it was new.
    pub fn insert(mut self, x: T) -> Bool:
        if t.find(x) >= 0: return false
        t.put(x, ())
        true

    /// Adds `x`; whether it was new.
    pub fn add(mut self, x: T) -> Bool:
        self.insert(x)

    /// Removes `x`; whether it was there.
    pub fn remove(mut self, x: T) -> Bool:
        t.drop(x)

    /// Adds `x` when `on`, else removes it.
    pub fn set(mut self, x: T, on: Bool):
        if on: t.put(x, ()) else: t.drop(x)

    pub fn first_ge(self, x: T) -> ?T:
        t.key_at(t.above(x, true))

    pub fn first_gt(self, x: T) -> ?T:
        t.key_at(t.above(x, false))

    pub fn last_le(self, x: T) -> ?T:
        t.key_at(t.below(x, true))

    pub fn last_lt(self, x: T) -> ?T:
        t.key_at(t.below(x, false))

    /// Removes and returns the least element.
    pub fn pop_first(mut self) -> ?T:
        let i = t.pop_edge(false)
        t.key_at(i)

    /// Removes and returns the greatest element.
    pub fn pop_last(mut self) -> ?T:
        let i = t.pop_edge(true)
        t.key_at(i)

/// `x in s`.
impl<T: Ord> Contains<T> for Set<T>:
    fn contains(self, x: T) -> Bool = t.find(x) >= 0

impl<T> Seq<T> for Set<T>:
    fn iter(self) -> yields T:
        for i in t.nodes(): yield t.keys[i]

impl<T> Sized for Set<T>:
    @get fn len(self) -> Int = t.n

/// An ordered map: each key once, in `<` order, with its value; \(O(\log n)\) to read, write
/// and remove one. `m[k]` traps when `k` is absent (`m.get(k)` is the `?V`), `m[k] = v` inserts
/// or replaces, and `m[k] op= v` starts an absent key at `V.default()` (R07.12.9). A map
/// literal `[k: v]` builds one (R01.4.5).
@role(map)
pub struct Map<K, V>:
    var t: OrdTree<K, V> = OrdTree()

impl<K, V> Map<K, V>:
    @get pub fn len(self) -> Int = t.n

    @get pub fn is_empty(self) -> Bool = t.n == 0

    fn entry(self, i: Int) -> ?(K, V):
        if i < 0: None else: Some((t.keys[i], t.vals[i]))

    /// The entry of the least key.
    @get pub fn first(self) -> ?(K, V) = self.entry(t.edge(false))

    /// The entry of the greatest key.
    @get pub fn last(self) -> ?(K, V) = self.entry(t.edge(true))

    pub fn clear(mut self):
        t.clear()

    /// The entries in key order.
    @get pub fn items(self) -> yields (K, V):
        for i in t.nodes(): yield (t.keys[i], t.vals[i])

    /// The keys in order.
    @get pub fn keys(self) -> yields K:
        for i in t.nodes(): yield t.keys[i]

    /// The values in key order.
    @get pub fn values(self) -> yields V:
        for i in t.nodes(): yield t.vals[i]

impl<K: Ord, V> Map<K, V>:
    /// `k`'s value, or `None` when `k` is absent.
    pub fn get(self, k: K) -> ?V:
        let i = t.find(k)
        if i < 0: None else: Some(t.vals[i])

    /// Removes `k`; its value, or `None` when it was absent.
    pub fn remove(mut self, k: K) -> ?V:
        let v = self.get(k)
        let _ = t.drop(k)
        v

    /// The entry of the least key `>= k`.
    pub fn first_ge(self, k: K) -> ?(K, V) = self.entry(t.above(k, true))

    /// The entry of the least key `> k`.
    pub fn first_gt(self, k: K) -> ?(K, V) = self.entry(t.above(k, false))

    /// The entry of the greatest key `<= k`.
    pub fn last_le(self, k: K) -> ?(K, V) = self.entry(t.below(k, true))

    /// The entry of the greatest key `< k`.
    pub fn last_lt(self, k: K) -> ?(K, V) = self.entry(t.below(k, false))

    /// Removes and returns the entry of the least key.
    pub fn pop_first(mut self) -> ?(K, V):
        let i = t.pop_edge(false)
        self.entry(i)

    /// Removes and returns the entry of the greatest key.
    pub fn pop_last(mut self) -> ?(K, V):
        let i = t.pop_edge(true)
        self.entry(i)

/// `k in m`: whether `k` has a value.
impl<K: Ord, V> Contains<K> for Map<K, V>:
    fn contains(self, k: K) -> Bool = t.find(k) >= 0

impl<K: Ord, V> IndexGet<K, V> for Map<K, V>:
    fn at(self, k: K) -> V:
        let i = t.find(k)
        check_key(i >= 0)
        t.vals[i]

impl<K: Ord, V: Default> IndexMut<K, V> for Map<K, V>:
    fn at(mut self, k: K) -> lends mut V:
        var i = t.find(k)
        if i < 0:
            t.put(k, V.default())
            i = t.find(k)
        lend mut self.t.vals[i]

impl<K: Ord, V> IndexSet<K, V> for Map<K, V>:
    fn set(mut self, k: K, v: V):
        t.put(k, v)

impl<K, V> Seq<(K, V)> for Map<K, V>:
    fn iter(self) -> yields (K, V):
        for i in t.nodes(): yield (t.keys[i], t.vals[i])

impl<K, V> Sized for Map<K, V>:
    @get fn len(self) -> Int = t.n

/// A set prints as its elements in order, one space apart, as a list does (R10.3.5).
impl Show for Set<T: Show>:
    fn show(s):
        var out = ""
        for x in s:
            if not out.is_empty: out += " "
            out += "{x}"
        out

/// A map prints as its entries in key order, one `k v` per line, as a list of pairs does
/// (R10.3.5).
impl Show for Map<K: Show, V: Show>:
    fn show(m):
        var out = ""
        for (k, v) in m.items:
            if not out.is_empty: out += "\n"
            out += "{k} {v}"
        out
