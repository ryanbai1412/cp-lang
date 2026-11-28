//! `List<T>`, `[D]T` and `Str`: values in a run of cells, read and written in \(O(1)\) at a key.
//! Core re-exports all three, so no module imports them.
//!
//! start: `List`, `Str`, `Array`, `List.insert`, `List.remove`
//! example: `var xs = [3, 1]; xs.push(2); xs[2] == 2 and xs.len == 3`
use std.raw.{raw_empty, raw_new, raw_get, raw_set, raw_take, raw_cap, check_key}
use std.literal.{ListLiteral, Filled}
use std.seq.{Seq, IndexGet, IndexMut, IndexedMut, Keyed, Positional, Indexed}

fn out(text: Raw<Char>, n: Int) = @intrinsic("out")

/// A growable run of `T`s, `[a, b]`: \(O(1)\) at a position and amortized \(O(1)\) to `push` (R07.1).
@role(list)
pub struct List<T>:
    var buf: Raw<T>
    var n: Int
    var cap: Int

    pub fn new() -> List<T>:
        List(raw_empty(), 0, 0)

    pub fn filled(n: Int, x: T) -> List<T>:
        List(raw_new(n, x), n, n)

    @get pub fn len(self) -> Int:
        n

    pub fn get(self, i: Int) -> T:
        check_key(0 <= i and i < n)
        raw_get(buf, i)

    pub fn each(self) -> yields T:
        for i in 0..n: yield raw_get(buf, i)

    /// Room for `k` elements without another move of the ones held.
    pub fn reserve(mut self, k: Int):
        if k > cap:
            var nb = raw_cap(k)
            for i in 0..n: raw_set(mut nb, i, raw_take(mut buf, i))
            buf = nb
            cap = k

    pub fn swap(mut self, i: Int, j: Int):
        check_key(0 <= i and i < n and 0 <= j and j < n)
        if i != j:
            let x = raw_take(mut buf, i)
            raw_set(mut buf, i, raw_take(mut buf, j))
            raw_set(mut buf, j, x)

    /// `x` at position `i`, the elements from `i` on one later.
    pub fn insert(mut self, i: Int, x: T):
        assert 0 <= i and i <= n, "insert: a position past the end"
        self.push(x)
        var k = n - 1
        while k > i:
            self.swap(k - 1, k)
            k -= 1

    /// The element at `i`, the ones after it one earlier.
    pub fn remove(mut self, i: Int) -> T:
        check_key(0 <= i and i < n)
        for k in i..n - 1: self.swap(k, k + 1)
        n -= 1
        raw_take(mut buf, n)

    pub fn extend(mut self, xs: List<T>):
        self.reserve(n + xs.len)
        for x in xs: self.push(x)

    @get pub fn is_empty(self) -> Bool:
        n == 0

    pub fn pop(mut self) -> ?T:
        if n == 0: return None
        n -= 1
        Some(raw_get(buf, n))

    pub fn truncate(mut self, k: Int):
        check_key(0 <= k)
        if k < n: n = k

    pub fn clear(mut self):
        n = 0

    /// `k` elements: the first ones kept, any new ones `x` (seq/U58).
    pub fn resize(mut self, k: Int, x: T):
        check_key(0 <= k)
        self.reserve(k)
        while n < k: self.push(x)
        self.truncate(k)

    /// Every element `x` (seq/U58).
    pub fn fill(mut self, x: T):
        for i in 0..n:
            raw_take(mut buf, i)
            raw_set(mut buf, i, x)

/// `[D]T`: one `T` for each position of the domain `D`, read and written at a `D` (R04.3).
@role(array)
pub struct Array<D: Domain, T>:
    buf: Raw<T>

    @get pub fn len(self) -> Int:
        D.count

    @get pub fn is_empty(self) -> Bool = D.count == 0

    pub fn filled(x: T) -> Array<D, T>:
        Array(raw_new(D.count, x))

    pub fn get(self, i: D) -> T:
        raw_get(buf, i.ord)

    pub fn each(self) -> yields T:
        for i in 0..D.count: yield raw_get(buf, i)

/// Cell by cell, in walk order (R08.1.6).
impl<D: Domain, T: Eq> Eq for Array<D, T>:
    fn eq(a, b):
        for i in 0..D.count:
            if raw_get(a.buf, i) != raw_get(b.buf, i): return false
        true

/// A list where `[D]T` is wanted: one element per position, in walk order (R04.8.10).
@coerce
fn array_of_list<D: Domain, T>(xs: List<T>) -> [D]T:
    assert xs.len == D.count, "a list for `[D]T` has one element per position"
    Array(xs.buf)

/// A word: a flat sequence of `Char`s, `"ab"` (R02.3).
@role(str)
pub struct Str:
    pub var chars: List<Char>

    /// The empty string; `Str(x)` is `x`'s conversion to a word (R07.8.1).
    pub fn new() -> Str = Str(List<Char>.new())

    @get pub fn len(self) -> Int:
        chars.len

    @get pub fn is_empty(self) -> Bool:
        chars.is_empty

    pub fn get(self, i: Int) -> Char:
        chars.get(i)

    pub fn each(self) -> yields Char:
        for c in chars: yield c

    pub fn push(mut self, c: Char):
        chars.push(c)

    /// Writes the text to standard output as it is.
    pub fn write_out(self) = out(chars.buf, chars.n)

    pub fn append(mut self, s: Str):
        for c in s.chars: chars.push(c)

// A string literal: its characters and how many there are. The literal's storage is lent,
// so the string gets its own.
@role(str_literal)
fn str_of(text: Raw<Char>, n: Int) -> Str:
    var s = Str()
    for i in 0..n: s.push(raw_get(text, i))
    s

/// `s + t`: a new string, `s`'s characters then `t`'s (R02.3.3).
impl Add for Str:
    fn add(self, other: Str) -> Str = str_concat(self, other)

/// `xs + s`: a new list, `xs`'s elements then the ones `s` walks, of a list, a view or any other
/// sequence (R02.3.3, R04a.2.3). \(O(|xs| + |s|)\).
impl<T: Clone, S: Seq<T>> Add<S> for List<T>:
    fn add(self, other: S) -> List<T>:
        var out = List<T>()
        for x in self: out.push(x.clone())
        for x in other: out.push(x)
        out

fn str_concat(a: Str, b: Str) -> Str:
    var out = Str()
    for c in a: out.push(c)
    for c in b: out.push(c)
    out

/// Strings compare character by character, a prefix first (R08.1.5).
impl Eq for Str:
    fn eq(a, b) = str_eq(a, b)
impl Ord for Str:
    fn lt(a, b):
        let n = if a.len < b.len: a.len else: b.len
        for i in 0..n:
            if a.get(i) != b.get(i): return a.get(i) < b.get(i)
        a.len < b.len

fn str_eq(a: Str, b: Str) -> Bool:
    if a.len != b.len: return false
    for i in 0..a.len:
        if a.get(i) != b.get(i): return false
    true

/// The same length and equal values in order.
impl<T: Eq> Eq for List<T>:
    fn eq(a, b):
        if a.n != b.n: return false
        for i in 0..a.n:
            if raw_get(a.buf, i) != raw_get(b.buf, i): return false
        true
/// Lists compare element by element, a prefix first (R08.1.5).
impl<T: Ord> Ord for List<T>:
    fn lt(a, b):
        let n = if a.n < b.n: a.n else: b.n
        for i in 0..n:
            if raw_get(a.buf, i) != raw_get(b.buf, i): return raw_get(a.buf, i) < raw_get(b.buf, i)
        a.n < b.n

/// A new buffer holding a clone of each element.
impl<T: Clone> Clone for List<T>:
    fn clone(a):
        if a.n == 0: return List.new()
        var out = List.filled(a.n, raw_get(a.buf, 0).clone())
        for i in 1..a.n: raw_set(mut out.buf, i, raw_get(a.buf, i).clone())
        out
impl Clone for Str:
    fn clone(a) = Str(a.chars)
impl<D: Domain, T: Clone> Clone for Array<D, T>:
    fn clone(a):
        if D.count == 0: return Array(raw_empty())
        var buf = raw_new(D.count, raw_get(a.buf, 0).clone())
        for i in 1..D.count: raw_set(mut buf, i, raw_get(a.buf, i).clone())
        Array(buf)

impl<T> ListLiteral<T> for List<T>:
    fn with_room(k: Int) -> List<T>:
        var xs = List.new()
        xs.reserve(k)
        xs

    fn push(mut self, x: T):
        if n == cap: self.reserve(if cap == 0: 4 else: cap * 2)
        raw_set(mut buf, n, x)
        n += 1

impl<D: Domain, T> Filled<T> for Array<D, T>:
    fn filled(x: T) -> Array<D, T>:
        [D].filled(x)

/// A list, a string and an array read in O(1) at any key, by position from 0 or, for an
/// array, by its domain's positions (R12.4.5).
impl<T> Positional<T> for List<T>:
    fn nth(self, k: Int) -> T = self.get(k)
impl<T> IndexGet<Int, T> for List<T>:
    fn at(self, i: Int) -> T = self.get(i)
impl<T> Indexed<Int, T> for List<T>:
    @get fn keys(self) -> Range<Int> = 0..self.len
impl Positional<Char> for Str:
    fn nth(self, k: Int) -> Char = self.get(k)
impl IndexGet<Int, Char> for Str:
    fn at(self, i: Int) -> Char = self.get(i)
impl Indexed<Int, Char> for Str:
    @get fn keys(self) -> Range<Int> = 0..self.len
impl<T> IndexMut<Int, T> for List<T>:
    fn at(mut self, i: Int) -> lends mut T:
        check_key(0 <= i and i < self.n)
        lend mut self.buf[i]
impl<T> IndexedMut<Int, T> for List<T>
impl IndexMut<Int, Char> for Str:
    fn at(mut self, i: Int) -> lends mut Char:
        check_key(0 <= i and i < self.chars.n)
        lend mut self.chars.buf[i]
impl IndexedMut<Int, Char> for Str
impl<D: Domain, T> Keyed<D, T> for Array<D, T>:
    fn items(self) -> yields (D, T):
        for i in 0..D.count: yield (D.nth(i), raw_get(self.buf, i))
impl<D: Domain, T> Positional<T> for Array<D, T>:
    fn nth(self, k: Int) -> T = self.get(D.nth(k))
impl<D: Domain, T> IndexGet<D, T> for Array<D, T>:
    fn at(self, i: D) -> T = self.get(i)
impl<D: Domain, T> Indexed<D, T> for Array<D, T>:
    @get fn keys(self) -> Range<D> = D
impl<D: Domain, T> IndexMut<D, T> for Array<D, T>:
    fn at(mut self, i: D) -> lends mut T:
        lend mut self.buf[i.ord]
impl<D: Domain, T> IndexedMut<D, T> for Array<D, T>

/// `Str(s)`: a new word of the characters `s` walks, in walk order. \(O(n)\) (R07.8.1).
impl<S: Seq<Char>> Into<Str> for S:
    fn into(self) -> Str:
        var out = Str()
        for c in self: out.push(c)
        out

/// `Str(c)`: the one-character word `c`; a `Char` is one character of text (R07.8.1).
impl Into<Str> for Char:
    fn into(self) -> Str:
        var out = Str()
        out.push(self)
        out

/// `List(s)`: a new list of `s`'s elements in walk order. \(O(n)\) (R07.8.1).
impl<S: Seq<T>, T> Into<List<T>> for S:
    fn into(self) -> List<T>:
        var xs = List<T>()
        for x in self: xs.push(x)
        xs
