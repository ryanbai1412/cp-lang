//! Double-ended queue: `Deque<T>`.
//!
//! start: `Deque`, `LinkedList`, `Deque.push_front`, `Deque.pop_front`
//! example: `var q: Deque<Int> = Deque(); q.push(2); q.push_front(1); q.pop_front() == Some(1) and q.len == 1`

use std.range.{Len}
use std.seq.{Bidir, Seq}

/// A queue with both ends open: `var q = Deque<Int>()`. `push` / `pop` work at the back,
/// `push_front` / `pop_front` at the front, all amortized \(O(1)\); `q[i]` is the `i`-th element
/// from the front in \(O(1)\). Memory is a ring of at most twice the largest length.
pub struct Deque<T>:
    buf: List<T>
    head: Int
    n: Len
    mask: Int
    want: Int

    /// Create an empty `Deque<T>`.
    ///
    /// cost: \(O(1)\); nothing is allocated until the first push
    /// traps: never
    /// example: `var q: Deque<Int> = Deque(); q.len == 0`
    pub fn new() -> Deque<T> = Deque([], 0, 0, -1, 1)

    /// A deque holding `xs`, front to back: the queue of a search, `var q = Deque.of(sources)`,
    /// then `while let u = q.pop_front(): ...`. \(O(n)\).
    ///
    /// The first element of `xs` is the front, so `pop_front` returns the elements in the
    /// order `xs` lists them.
    ///
    /// cost: \(O(n)\)
    /// empty: `Deque<Int>.of([]).len == 0`
    /// traps: never
    /// example: `Deque.of([1, 2, 3]).to_list() == [1, 2, 3]`
    /// example: `var q = Deque.of([1, 2]); q.pop_front() == Some(1)`
    pub fn of(xs: List<T>) -> Deque<T>:
        var q = Deque<T>.new()
        q.reserve(xs.len)
        for x in xs: q.push(x)
        q

    /// Reserve capacity for at least `cap` elements without reallocating. \(O(1)\) when empty, \(O(\mathit{cap})\) otherwise.
    ///
    /// The contents and their order do not change; a `cap` no larger than the current
    /// capacity (including any `cap <= 0`) does nothing.
    ///
    /// cost: \(O(\mathit{cap})\); \(O(1)\) when empty
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); q.reserve(100); q.to_list() == [1, 2]`
    pub fn reserve(mut self, cap: Int):
        var c = 1
        while c < cap: c *= 2
        if c <= mask + 1: return
        if buf.is_empty:
            want max= c
        else:
            self.grow_to(c, buf[head])

    /// Number of elements currently in the deque.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Deque.of([4, 5]).len == 2`
    @get pub fn len(self) -> Int:
        n
    /// Whether the deque is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Deque.of([7]).is_empty == false`
    @get pub fn is_empty(self) -> Bool:
        n == 0

    /// Remove all elements from the deque.
    ///
    /// The capacity is kept, so refilling does not reallocate.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); q.clear(); q.len == 0`
    pub fn clear(mut self):
        buf = []
        head = 0
        n = 0
        mask = -1
        want = 1

    /// Move the deque's contents out in \(O(1)\), leaving `self` empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); let r = q.take(); q.len == 0 and r.to_list() == [1, 2]`
    pub fn take(mut self) -> Deque<T>:
        let out = Deque(buf, head, n, mask, want)
        buf = []
        head = 0
        n = 0
        mask = -1
        want = 1
        out

    fn grow_to(mut self, cap: Int, fill: T):
        let m = mask
        var nb = List.filled(cap, fill)
        unchecked:
            for i in 0..n:
                nb[i] = buf[(head + i) & m]
        swap(mut buf, mut nb)
        head = 0
        mask = cap - 1

    /// Append `x` to the back of the deque.
    ///
    /// cost: \(O(1)\) amortized; a full ring doubles
    /// traps: never
    /// example: `var q = Deque.of([1]); q.push(2); q.to_list() == [1, 2]`
    pub fn push(mut self, x: T):
        if n > mask:
            let cap = if mask < 0: want else: 2 * (mask + 1)
            self.grow_to(cap, x)
        unchecked:
            buf[(head + n) & mask] = x
        n += 1

    /// Prepend `x` to the front of the deque.
    ///
    /// Every existing element moves one place back: `q[0]` is now `x`.
    ///
    /// cost: \(O(1)\) amortized; a full ring doubles
    /// traps: never
    /// example: `var q = Deque.of([1]); q.push_front(0); q.to_list() == [0, 1]`
    pub fn push_front(mut self, x: T):
        if n > mask:
            let cap = if mask < 0: want else: 2 * (mask + 1)
            self.grow_to(cap, x)
        unchecked:
            head = (head - 1) & mask
            buf[head] = x
        n += 1

    /// Remove and return the back, or `None` when empty.
    ///
    /// cost: \(O(1)\)
    /// empty: `var q = Deque<Int>(); q.pop() == None and q.len == 0`
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); q.pop() == Some(2) and q.to_list() == [1]`
    pub fn pop(mut self) -> ?T:
        if n == 0: return None
        n -= 1
        unchecked:
            return Some(buf[(head + n) & mask])

    /// Remove and return the front, or `None` when empty.
    ///
    /// cost: \(O(1)\)
    /// empty: `var q = Deque<Int>(); q.pop_front() == None and q.len == 0`
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); q.pop_front() == Some(1) and q.to_list() == [2]`
    @lint(while_not_empty_pop, from: "while not self.is_empty:\n    let x = self.pop_front()!\n    b", to: "while let x = self.pop_front():\n    b", holes: "x, b")
    @lint(while_not_empty_pop, from: "while self.len > 0:\n    let x = self.pop_front()!\n    b", to: "while let x = self.pop_front():\n    b", holes: "x, b")
    @lint(while_not_empty_pop, from: "while self.len != 0:\n    let x = self.pop_front()!\n    b", to: "while let x = self.pop_front():\n    b", holes: "x, b")
    pub fn pop_front(mut self) -> ?T:
        if n == 0: return None
        unchecked:
            let x = buf[head]
            head = (head + 1) & mask
            n -= 1
            return Some(x)

    /// \(O(k)\) for `k` removed. Remove and return the longest front run on which `ok` holds,
    /// in order: a queue sorted by key releases everything below a rising threshold.
    ///
    /// `ok` is called front to back and the run stops at the first element where it is false;
    /// that element and everything behind it stay. `ok` is called at most `k + 1` times.
    ///
    /// cost: \(O(k)\) for `k` removed
    /// empty: `var q = Deque<Int>(); q.pop_front_while(x => x < 4) == []`
    /// order: front first, the order the elements sat in; a new `List`, keyed from 0
    /// traps: never
    /// example: `var q = Deque.of([1, 2, 5, 3]); q.pop_front_while(x => x < 4) == [1, 2] and q.to_list() == [5, 3]`
    pub fn pop_front_while(mut self, ok: T -> Bool) -> List<T>:
        var out: List<T> = []
        while n > 0 and ok(self.get(0)): out.push(self.pop_front()!)
        out

    /// Pop from the back while `ok` holds; the popped values, back first.
    /// \(O(\mathit{popped})\).
    ///
    /// The mirror of `pop_front_while`: `ok` is called back to front and the run stops at the
    /// first element where it is false.
    ///
    /// cost: \(O(k)\) for `k` removed
    /// empty: `var q = Deque<Int>(); q.pop_while(x => x < 4) == []`
    /// order: back first, the order they were popped; a new `List`, keyed from 0
    /// traps: never
    /// example: `var q = Deque.of([5, 1, 2]); q.pop_while(x => x < 4) == [2, 1] and q.to_list() == [5]`
    pub fn pop_while(mut self, ok: T -> Bool) -> List<T>:
        var out: List<T> = []
        while n > 0 and ok(self.get(n - 1)): out.push(self.pop()!)
        out

    /// The front, or `None` when empty.
    ///
    /// cost: \(O(1)\)
    /// empty: `Deque<Int>().first == None`
    /// ties: `Deque.of([4, 4]).first == Some(4)`
    /// traps: never
    /// example: `Deque.of([4, 5]).first == Some(4)`
    @get pub fn first(self) -> ?T:
        if n == 0: return None
        Some(self.get(0))

    /// The back, or `None` when empty. `dq.last(k)` is the element `k`
    /// back from it, so `dq.last(1)` is the one before the back.
    ///
    /// `dq.last(k)` is `dq[dq.len - 1 - k]` when `k < dq.len`, and `None` otherwise.
    ///
    /// cost: \(O(1)\)
    /// empty: `let q = Deque<Int>(); q.last == None and q.last(3) == None`
    /// ties: `let q = Deque.of([4, 4, 5]); q.last(1) == Some(4) and q.last(2) == Some(4)`
    /// traps: when `k < 0`
    /// example: `Deque.of([4, 5, 6]).last == Some(6)`
    /// example: `Deque.of([4, 5, 6]).last(1) == Some(5)`
    /// example: `Deque.of([4]).last(1) == None`
    @get pub fn last(self, k: 0.. = 0) -> ?T:
        if k < 0 or k >= n: return None
        Some(self.get(n - 1 - k))

    /// Each element with its place from the front: `for (i, x) in dq.items`. \(O(1)\) to make,
    /// \(O(n)\) to walk.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) per step
    /// empty: `Deque<Int>().items.collect() == []`
    /// order: front to back; the places are `0..len`
    /// traps: never
    /// example: `Deque.of([7, 8]).items.collect() == [(0, 7), (1, 8)]`
    @get pub fn items(self) -> yields (Int, T):
        for i in 0..n: yield (i, self.get(i))

    /// The `i`-th element from the front.
    ///
    /// `q[i]` calls this; place `0` is the front and `len - 1` the back.
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is not in `0..len`
    /// example: `Deque.of([7, 8]).get(1) == 8`
    /// example: `var q = Deque.of([1]); q.push_front(0); q[0] == 0`
    pub fn get(self, i: Int) -> T:
        assert 0 <= i < n
        unchecked:
            return buf[(head + i) & mask]

    /// Overwrite the `i`-th element from the front (W8 `IndexSet` backing `dq[i] = x`).
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is not in `0..len`
    /// example: `var q = Deque.of([7, 8]); q[0] = 9; q.to_list() == [9, 8]`
    pub fn set(mut self, i: Int, x: T):
        assert 0 <= i < n
        unchecked:
            buf[(head + i) & mask] = x

    /// Materialize the elements front to back as a `List<T>`.
    ///
    /// cost: \(O(n)\), one new `List`
    /// empty: `Deque<Int>().to_list() == []`
    /// order: front to back; a new `List`, keyed from 0
    /// traps: never
    /// example: `var q = Deque.of([2]); q.push_front(1); q.to_list() == [1, 2]`
    pub fn to_list(self) -> List<T>:
        let out: List<T> = [self.get(i) for i in 0..n]
        out

    /// Yield elements front to back without allocating.
    ///
    /// `for x in q` uses this.
    ///
    /// cost: \(O(1)\) per step
    /// empty: `Deque<Int>().each().collect() == []`
    /// order: front to back
    /// traps: never
    /// example: `Deque.of([1, 2]).each().collect() == [1, 2]`
    pub fn each(self) -> yields T:
        for i in 0..n: yield self.get(i)

    /// Yield elements back to front without allocating.
    ///
    /// cost: \(O(1)\) per step
    /// empty: `Deque<Int>().each_rev().collect() == []`
    /// order: back to front
    /// traps: never
    /// example: `Deque.of([1, 2]).each_rev().collect() == [2, 1]`
    pub fn each_rev(self) -> yields T:
        var i = n
        while i > 0:
            i -= 1
            yield self.get(i)

    /// Rotate elements left by `k` positions (`k` mod `len`).
    ///
    /// The element at place `k` becomes the front. `k` is reduced with Euclidean `%`, so a
    /// negative `k` rotates right: `rotate_left(-1)` is `rotate_right(1)`.
    ///
    /// cost: \(O(\min(k, n - k))\) after reducing `k` mod `n`
    /// empty: `var q = Deque<Int>(); q.rotate_left(3); q.rotate_left(-1); q.to_list() == []`
    /// traps: never
    /// example: `var q = Deque.of([1, 2, 3, 4]); q.rotate_left(1); q.to_list() == [2, 3, 4, 1]`
    /// example: `var q = Deque.of([1, 2, 3, 4]); q.rotate_left(-1); q.to_list() == [4, 1, 2, 3]`
    pub fn rotate_left(mut self, k: Int):
        if n == 0: return
        let s = k % n
        if s <= n - s:
            for _ in 0..s: self.push(self.pop_front()!)
        else:
            for _ in 0..n - s: self.push_front(self.pop()!)

    /// Rotate elements right by `k` positions (`k` mod `len`).
    ///
    /// The back `k` elements move to the front, in order. `rotate_right(k)` is
    /// `rotate_left(-k)`.
    ///
    /// cost: \(O(\min(k, n - k))\) after reducing `k` mod `n`
    /// empty: `var q = Deque<Int>(); q.rotate_right(3); q.rotate_right(-1); q.to_list() == []`
    /// traps: never
    /// example: `var q = Deque.of([1, 2, 3, 4]); q.rotate_right(1); q.to_list() == [4, 1, 2, 3]`
    /// example: `var q = Deque.of([1, 2, 3]); q.rotate_right(4); q.to_list() == [3, 1, 2]`
    pub fn rotate_right(mut self, k: Int):
        if n == 0: return
        self.rotate_left(-k)

/// Doubly-linked list supporting \(O(1)\) end pushes/pops and splicing (`splice_back`, `splice_front`) that moves only the shorter list.
pub struct LinkedList<T>:
    _len: Int
    head: Int
    tail: Int
    nx: List<Int>
    pv: List<Int>
    val: List<T>
    flip: Bool

    fn fwd(self, i: Int) -> Int = if flip: pv[i] else: nx[i]

    fn bwd(self, i: Int) -> Int = if flip: nx[i] else: pv[i]

    fn set_fwd(mut self, i: Int, j: Int):
        if flip: pv[i] = j else: nx[i] = j

    fn set_bwd(mut self, i: Int, j: Int):
        if flip: nx[i] = j else: pv[i] = j

    fn node(mut self, x: T) -> Int:
        val.push(x)
        nx.push(-1)
        pv.push(-1)
        _len += 1
        val.len - 1

    /// Create an empty `LinkedList<T>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var l: LinkedList<Int> = LinkedList(); l.len == 0`
    pub fn new() -> LinkedList<T>:
        LinkedList(0, -1, -1, [], [], [], false)

    /// Create a singleton `LinkedList<T>` containing `x`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `LinkedList.of(5).each().collect() == [5]`
    pub fn of(x: T) -> LinkedList<T>:
        var l = LinkedList<T>.new()
        l.push(x)
        l

    /// Build a `LinkedList<T>` from `xs`, front to back.
    ///
    /// cost: \(O(n)\)
    /// empty: `LinkedList<Int>.from_list([]).len == 0`
    /// traps: never
    /// example: `LinkedList.from_list([1, 2]).each().collect() == [1, 2]`
    pub fn from_list(xs: List<T>) -> LinkedList<T>:
        var l = LinkedList<T>.new()
        for x in xs: l.push(x)
        l

    /// Number of elements currently in the list.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `LinkedList.from_list([1, 2, 3]).len == 3`
    @get pub fn len(self) -> Int:
        _len

    /// Whether the list is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `LinkedList.of(1).is_empty == false`
    @get pub fn is_empty(self) -> Bool:
        _len == 0

    /// The front element, or `None` when the list is empty (as `Deque.first`).
    ///
    /// cost: \(O(1)\)
    /// empty: `LinkedList<Int>().first == None`
    /// ties: `LinkedList.from_list([4, 4]).first == Some(4)`
    /// traps: never
    /// example: `LinkedList.from_list([1, 2]).first == Some(1)`
    @get pub fn first(self) -> ?T:
        if head < 0: None else: Some(val[head])

    /// The back element, or `None` when the list is empty (as `Deque.last`).
    ///
    /// cost: \(O(1)\)
    /// empty: `LinkedList<Int>().last == None`
    /// ties: `LinkedList.from_list([4, 4]).last == Some(4)`
    /// traps: never
    /// example: `LinkedList.from_list([1, 2]).last == Some(2)`
    @get pub fn last(self) -> ?T:
        if tail < 0: None else: Some(val[tail])

    /// Append `x` to the back.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var l = LinkedList.of(1); l.push(2); l.each().collect() == [1, 2]`
    pub fn push(mut self, x: T):
        let k = self.node(x)
        if tail < 0:
            head = k
        else:
            self.set_fwd(tail, k)
            self.set_bwd(k, tail)
        tail = k

    /// Prepend `x` to the front.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var l = LinkedList.of(1); l.push_front(0); l.each().collect() == [0, 1]`
    pub fn push_front(mut self, x: T):
        let k = self.node(x)
        if head < 0:
            tail = k
        else:
            self.set_bwd(head, k)
            self.set_fwd(k, head)
        head = k

    /// Remove and return the back element, or `None` if empty.
    ///
    /// cost: \(O(1)\)
    /// empty: `var l = LinkedList<Int>(); l.pop() == None`
    /// traps: never
    /// example: `var l = LinkedList.from_list([1, 2]); l.pop() == Some(2) and l.len == 1`
    pub fn pop(mut self) -> ?T:
        if tail < 0: return None
        let x = val[tail]
        let t = self.bwd(tail)
        if t < 0: head = -1 else: self.set_fwd(t, -1)
        tail = t
        _len -= 1
        Some(x)

    /// Remove and return the front element, or `None` if empty.
    ///
    /// cost: \(O(1)\)
    /// empty: `var l = LinkedList<Int>(); l.pop_front() == None`
    /// traps: never
    /// example: `var l = LinkedList.from_list([1, 2]); l.pop_front() == Some(1) and l.first == 2`
    pub fn pop_front(mut self) -> ?T:
        if head < 0: return None
        let x = val[head]
        let h = self.fwd(head)
        if h < 0: tail = -1 else: self.set_bwd(h, -1)
        head = h
        _len -= 1
        Some(x)

    /// Splice all elements of `other` onto the back of `self`, leaving `other` empty.
    ///
    /// The elements keep their order: `self`'s first, then `other`'s. Only the shorter list's
    /// elements are moved, so merging lists smaller into larger costs \(O(n \log n)\) in total.
    ///
    /// cost: \(O(\min(\lvert \mathit{self} \rvert, \lvert \mathit{other} \rvert))\)
    /// traps: never
    /// example: `var a = LinkedList.from_list([1, 2]); var b = LinkedList.from_list([3, 4]); a.splice_back(mut b); a.each().collect() == [1, 2, 3, 4] and b.is_empty`
    pub fn splice_back(mut self, mut other: LinkedList<T>):
        if other.len <= _len:
            for x in other.each(): self.push(x)
        else:
            swap(mut self, mut other)
            for x in other.each_rev(): self.push_front(x)
        other.clear()

    /// Splice all elements of `other` onto the front of `self`, leaving `other` empty.
    ///
    /// The elements keep their order: `other`'s first, then `self`'s. Only the shorter list's
    /// elements are moved, as in `splice_back`.
    ///
    /// cost: \(O(\min(\lvert \mathit{self} \rvert, \lvert \mathit{other} \rvert))\)
    /// traps: never
    /// example: `var a = LinkedList.from_list([1, 2]); var b = LinkedList.from_list([3, 4]); a.splice_front(mut b); a.each().collect() == [3, 4, 1, 2] and b.is_empty`
    pub fn splice_front(mut self, mut other: LinkedList<T>):
        if other.len <= _len:
            for x in other.each_rev(): self.push_front(x)
        else:
            swap(mut self, mut other)
            for x in other.each(): self.push(x)
        other.clear()

    /// Reverse the list in place. \(O(1)\) via reversible links.
    ///
    /// cost: \(O(1)\)
    /// empty: `let xs: List<Int> = []; var a = LinkedList.from_list(xs); a.reverse(); a.each().collect() == []`
    /// traps: never
    /// example: `var a = LinkedList.from_list([1, 2, 3]); a.reverse(); a.each().collect() == [3, 2, 1]`
    pub fn reverse(mut self):
        (head, tail) = (tail, head)
        flip = not flip

    /// Remove all elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var a = LinkedList.from_list([1, 2]); a.clear(); a.is_empty`
    pub fn clear(mut self):
        (_len, head, tail, flip) = (0, -1, -1, false)
        nx = []
        pv = []
        val = []

    /// Yield elements front to back.
    ///
    /// cost: \(O(1)\) per step
    /// empty: `LinkedList<Int>().each().collect() == []`
    /// order: front to back
    /// traps: never
    /// example: `LinkedList.from_list([1, 2]).each().collect() == [1, 2]`
    pub fn each(self) -> yields T:
        var u = head
        while u >= 0:
            yield val[u]
            u = self.fwd(u)

    /// Yield elements back to front.
    ///
    /// cost: \(O(1)\) per step
    /// empty: `LinkedList<Int>().each_rev().collect() == []`
    /// order: back to front
    /// traps: never
    /// example: `LinkedList.from_list([1, 2]).each_rev().collect() == [2, 1]`
    pub fn each_rev(self) -> yields T:
        var u = tail
        while u >= 0:
            yield val[u]
            u = self.bwd(u)

impl Seq<T> for Deque<T>:
    fn iter(self) -> yields T:
        for x in self.each(): yield x

impl Bidir<T> for Deque<T>:
    fn iter_rev(self) -> yields T:
        for x in self.each_rev(): yield x

impl Seq<T> for LinkedList<T>:
    fn iter(self) -> yields T:
        for x in self.each(): yield x

impl Bidir<T> for LinkedList<T>:
    fn iter_rev(self) -> yields T:
        for x in self.each_rev(): yield x

@test
fn a_deque_matches_a_list():
    var q: Deque<Int> = []
    var xs: List<Int> = []
    for _ in 0..400:
        let k = rand(0, 5)
        if k == 0:
            let x = rand(0, 99)
            q.push(x)
            xs.push(x)
        elif k == 1:
            let x = rand(0, 99)
            q.push_front(x)
            xs = [x] + xs
        elif k == 2:
            assert q.pop() == (if xs.is_empty: None else: xs.pop())
        elif k == 3:
            let want = xs.first
            if not xs.is_empty: xs.remove(0)
            assert q.pop_front() == want
        else:
            assert q.len == xs.len and q.is_empty == xs.is_empty
            assert q.first == xs.first and q.last == xs.last
            for i in 0..xs.len: assert q[i] == xs[i]
    q.clear()
    assert q.is_empty and q.pop() == None and q.pop_front() == None
    for x in [10, 20, 30]: q.push(x)
    var moved = q.take()
    assert q.is_empty and moved.len == 3 and moved.pop_front() == 10 and moved.pop() == 30

@test
fn a_deque_as_a_bfs_queue():
    // 0-1 BFS on a line: stepping right is free, stepping left costs 1
    var dist: List<?Int> = [None, None, None, None, None]
    var q = Deque<Int>()
    dist[2] = 0
    q.push(2)
    while let u = q.pop_front():
        if u + 1 < 5 and (dist[u + 1] == None or dist[u + 1]! > dist[u]!):
            dist[u + 1] = dist[u]
            q.push_front(u + 1)
        if u >= 1 and (dist[u - 1] == None or dist[u - 1]! > dist[u]! + 1):
            dist[u - 1] = dist[u]! + 1
            q.push(u - 1)
    assert dist == [2, 1, 0, 0, 0]

@test
fn a_deque_of_a_list_is_a_bfs_queue():
    domain V = 0..5
    var dist: [V]?Int = [V] _ => None
    let ends: List<V> = [V(0), V(4)]
    for v in ends: dist[v] = Some(0)
    var q = Deque.of(ends)
    assert q.len == 2 and q.first == Some(V(0)) and q.last == Some(V(4))
    while let u = q.pop_front():
        for d in [-1, 1]:
            let wi = u.ord + d
            if 0 <= wi < 5 and dist[V(wi)] == None:
                dist[V(wi)] = Some(dist[u]! + 1)
                q.push(V(wi))
    assert [dist[v]! for v in V] == [0, 1, 2, 1, 0]
    let none_yet: List<Int> = []
    assert Deque.of(none_yet).is_empty

@test
fn a_deque_reserve_preserves_order():
    var q: Deque<Int> = []
    q.reserve(64)
    for i in 0..20: q.push(i)
    for i in 1..=10: q.push_front(-i)
    q.reserve(256)
    q.reserve(4)
    assert q.len == 30
    assert q[0] == -10 and q[10] == 0 and q[29] == 19
    assert q.pop_front() == -10 and q.pop() == 19

impl Show for Deque<T>:
    fn show(q):
        var out = ""
        for x in q.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for LinkedList<T>:
    fn show(l):
        var out = ""
        for x in l.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out
