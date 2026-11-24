//! String algorithms. They only compare elements, so they work on a `Str`
//! (ASCII bytes) and on any `List<T>` alike. Positions are 0-based.

use std.math.{pow_mod}
use std.seq.{collect, nth, last_true}

/// `self` written `n` times in a row, as a new `Str`: `"0".repeat(4)` is `"0000"`.
///
/// cost: \(O(\lvert \mathit{self} \rvert \cdot n)\)
/// empty: an empty `self`, or `n == 0`, gives `""`
/// traps: when `n < 0`
/// example: `"ab".repeat(3) == "ababab"`
/// example: `"x".repeat(0) == ""`
pub fn repeat(self: Str, n: Int) -> Str:
    assert n >= 0, "`repeat` needs a count of 0 or more"
    Str(self[i % self.len] for i in 0..n * self.len)

/// True when `pat` occurs in `self` starting at position `at` (default 0, so a prefix test): the
/// `pat.len` elements of `self` from `at` equal those of `pat` in order, `false` when they run past
/// the end.
///
/// Works on any `Indexed` pair (`Str`, `List<T>`, `[D]T`, views); `at` counts positions from 0, and
/// only the values are compared, never the keys.
///
/// cost: \(O(\min(\lvert \mathit{self} \rvert - \mathit{at}, \lvert \mathit{pat} \rvert))\)
/// empty: an empty `pat` matches at every `at` in `0..=self.len`; an empty `self` starts only with an empty `pat`
/// traps: when `at` is outside `0..=self.len`
/// example: `"banana".starts_with("ban") == true`
/// example: `"banana".starts_with("nan", at: 2) == true`
/// example: `[1, 2, 3].starts_with([2]) == false`
pub fn starts_with<S: Indexed, P: Indexed>(self: S, pat: P, at: Int = 0) -> Bool:
    with lint(unproven_list_index: allow)
    assert 0 <= at <= self.len, "`starts_with` needs `at` in `0..=self.len`"
    if at + pat.len > self.len: return false
    for i in 0..pat.len:
        if self.nth(at + i) != pat.nth(i): return false
    true

/// True when `self` closes with `pat`: the last `pat.len` elements of `self` equal those of
/// `pat` in order. Works on any `Indexed` pair (`Str`, `List<T>`, `[D]T`, views); only the
/// values are compared, never the keys.
///
/// cost: \(O(\min(\lvert \mathit{self} \rvert, \lvert \mathit{pat} \rvert))\)
/// empty: an empty `pat` is a suffix of everything; an empty `self` ends only with an empty `pat`
/// traps: never
/// example: `[1, 2, 3].ends_with([2, 3]) == true`
/// example: `"banana".ends_with("nab") == false`
pub fn ends_with<S: Indexed, P: Indexed>(self: S, pat: P) -> Bool:
    with lint(unproven_list_index: allow)
    let d = self.len - pat.len
    if d < 0: return false
    for i in 0..pat.len:
        if self.nth(d + i) != pat.nth(i): return false
    true

/// The key where `p` first stands in `self`, or `None`. `p` is a `Str` or one `Char`.
///
/// The result is a key of `self`, so on a slice it is valid on the source: `s[l..].search(c)`
/// needs no `+ l`. An empty `p` stands at the first key of `self` (`None` when `self` is
/// empty): a match starts at an element.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert p \rvert)\)
/// empty: an empty `p` gives the first key, `None` on an empty `self`; an empty `self` gives `None`
/// ties: the leftmost occurrence
/// traps: never
/// example: `"banana".search("nan") == Some(2)`
/// example: `"banana".search('n') == Some(2)`
/// example: `"banana"[2..].search('a') == Some(3)`
/// example: `"banana".search("x") == None`
pub fn search<S: Indexed<K, T>, K, T, P>(self: S, p: P) -> ?K:
    for k in find_all(self, "" + p): return Some(k)
    None

/// The key where `p` last starts in `self`, or `None`. `p` is a `Str` or one `Char`.
///
/// Like `find`, the result is a key of `self`. An empty `p` stands at the last key: a match
/// starts at an element, never at the end.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert p \rvert)\)
/// empty: an empty `p` gives the last key, `None` on an empty `self`; an empty `self` gives `None`
/// ties: the rightmost occurrence
/// traps: never
/// example: `"banana".rsearch("an") == Some(3)`
/// example: `"banana".rsearch('a') == Some(5)`
/// example: `"ab".rsearch("") == Some(1)`
pub fn rsearch<S: Indexed<K, T>, K, T, P>(self: S, p: P) -> ?K:
    var last: ?K = None
    for k in find_all(self, "" + p): last = Some(k)
    last

/// The lowercase letter of an ASCII capital; any other character is returned as it is.
///
/// cost: \(O(1)\)
/// ties: None; one character in, one out
/// traps: never
/// example: `'A'.lower == 'a'`
/// example: `'7'.lower == '7'`
@get pub fn lower(self: Char) -> Char = if self.is_upper: Char(Int(self) + 32) else: self

/// The capital of an ASCII lowercase letter; any other character is returned as it is.
///
/// cost: \(O(1)\)
/// ties: None; one character in, one out
/// traps: never
/// example: `'a'.upper == 'A'`
/// example: `'?'.upper == '?'`
@get pub fn upper(self: Char) -> Char = if self.is_lower: Char(Int(self) - 32) else: self

/// The value `0..=9` of an ASCII decimal digit: `'7'.digit` is `7`.
///
/// cost: \(O(1)\)
/// traps: when `self` is not in `'0'..='9'` (test with `is_digit` first)
/// example: `'7'.digit == 7`
/// example: `'0'.digit == 0`
@get pub fn digit(self: Char) -> Int:
    assert self.is_digit, "`digit` needs a char in '0'..='9'"
    Int(self) - Int('0')

/// True when `self` is an ASCII decimal digit, `'0'..='9'`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `'5'.is_digit == true`
/// example: `'x'.is_digit == false`
@get pub fn is_digit(self: Char) -> Bool = self >= '0' and self <= '9'

/// True when `self` is an ASCII lowercase letter, `'a'..='z'`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `'q'.is_lower == true`
/// example: `'Q'.is_lower == false`
@get pub fn is_lower(self: Char) -> Bool = self >= 'a' and self <= 'z'

/// True when `self` is an ASCII capital letter, `'A'..='Z'`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `'Q'.is_upper == true`
/// example: `'q'.is_upper == false`
@get pub fn is_upper(self: Char) -> Bool = self >= 'A' and self <= 'Z'

/// True when `self` is an ASCII letter, `'a'..='z'` or `'A'..='Z'`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `'Z'.is_alpha == true`
/// example: `'_'.is_alpha == false`
@get pub fn is_alpha(self: Char) -> Bool = self.is_lower or self.is_upper

/// True when `self` is an ASCII letter or decimal digit.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `'9'.is_alnum == true`
/// example: `' '.is_alnum == false`
@get pub fn is_alnum(self: Char) -> Bool = self.is_alpha or self.is_digit

/// True when `self` is one of the four ASCII whitespace characters `' '`, `'\t'`, `'\n'`, `'\r'`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `'\t'.is_space == true`
/// example: `'x'.is_space == false`
@get pub fn is_space(self: Char) -> Bool:
    let c = Int(self)
    c == 32 or (9 <= c <= 13)

/// A copy of `self` with every ASCII capital replaced by its lowercase letter (`Char.lower`).
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `""`
/// traps: never
/// example: `"AbC1".to_lower() == "abc1"`
pub fn to_lower(self: Str) -> Str = Str([c.lower for c in self])

/// A copy of `self` with every ASCII lowercase letter replaced by its capital (`Char.upper`).
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `""`
/// traps: never
/// example: `"AbC1".to_upper() == "ABC1"`
pub fn to_upper(self: Str) -> Str = Str([c.upper for c in self])

/// A copy of `self` with leading and trailing whitespace (`Char.is_space`) removed; inner
/// whitespace is kept.
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `""`; so is a string of only whitespace
/// traps: never
/// example: `"  a b \n".trim() == "a b"`
/// example: `" \t ".trim() == ""`
pub fn trim(self: Str) -> Str = self.trim_start().trim_end()

/// A copy of `self` with leading whitespace (`Char.is_space`) removed.
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `""`; so is a string of only whitespace
/// traps: never
/// example: `"  a b \n".trim_start() == "a b \n"`
pub fn trim_start(self: Str) -> Str:
    var i = 0
    while i < self.len and self[i].is_space: i += 1
    Str([self[j] for j in i..self.len])

/// A copy of `self` with trailing whitespace (`Char.is_space`) removed.
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `""`; so is a string of only whitespace
/// traps: never
/// example: `"  a b \n".trim_end() == "  a b"`
pub fn trim_end(self: Str) -> Str:
    var n = self.len
    while n > 0 and self[n - 1].is_space: n -= 1
    Str([self[j] for j in 0..n])

/// `self` cut at the first `sep` into `(before, after)`, neither holding that `sep`; `None`
/// when `sep` does not occur. Later separators stay in `after`.
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `None`
/// traps: never
/// example: `"key=val=x".split_once('=') == Some(("key", "val=x"))`
/// example: `"abc".split_once('=') == None`
pub fn split_once(self: Str, sep: Char) -> ?(Str, Str):
    for i in 0..self.len:
        if self[i] == sep:
            return Some((Str([self[j] for j in 0..i]), Str([self[j] for j in i + 1..self.len])))
    None

/// The maximal runs of non-whitespace characters of `self` (`Char.is_space` separates), as
/// new strings. No piece is empty.
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `[]`; so is a string of only whitespace
/// order: left to right; a new `List`, keyed from 0
/// traps: never
/// example: `" a  b\tc ".split_whitespace() == ["a", "b", "c"]`
/// example: `"   ".split_whitespace() == []`
pub fn split_whitespace(self: Str) -> List<Str>:
    var out: List<Str> = []
    var cur: List<Char> = []
    for c in self:
        if c.is_space:
            if cur.len > 0:
                out.push(Str(cur))
                cur = []
        else:
            cur.push(c)
    if cur.len > 0: out.push(Str(cur))
    out

/// The elements of `parts` in order with `self` between each adjacent pair: `", ".join(xs)`.
///
/// Each element is written as string interpolation `"{x}"` writes it, so `Str`, `Char` and
/// numbers all join without a `map`.
///
/// cost: \(O(\text{total output length})\)
/// empty: `""`
/// traps: never
/// example: `", ".join(["a", "b", "c"]) == "a, b, c"`
/// example: `" ".join([1, 2, 3]) == "1 2 3"`
/// example: `"-".join(["x"]) == "x"`
pub fn join<S: Seq>(self: Str, parts: S) -> Str:
    var out = ""
    var first = true
    for x in parts:
        if not first: out += self
        out += "{x}"
        first = false
    out

/// A copy of `self` with every occurrence of `from` replaced by `to`.
///
/// Occurrences are taken greedily left to right and never overlap; the text that `to`
/// inserts is not scanned again.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert \mathit{output} \rvert)\)
/// empty: an empty `self` gives `""`
/// traps: when `from` is empty
/// example: `"aaaa".replace("aa", "b") == "bb"`
/// example: `"aaa".replace("aa", "b") == "ba"`
/// example: `"banana".replace("an", "") == "ba"`
pub fn replace(self: Str, from: Str, to: Str) -> Str:
    let m = from.len
    assert m > 0, "`replace` needs a non-empty `from`"
    var pi = List.filled(m, 0)
    for i in 1..m:
        var k = pi[i - 1]
        while k > 0 and from[i] != from[k]: k = pi[k - 1]
        if from[i] == from[k]: k += 1
        pi[i] = k
    var out: List<Char> = []
    var k = 0
    for c in self:
        out.push(c)
        while k > 0 and c != from[k]: k = pi[k - 1]
        if c == from[k]: k += 1
        if k == m:
            out.truncate(out.len - m)
            for t in to: out.push(t)
            k = 0
    Str(out)

/// `self` read as a signed integer in `base`, or `None` if malformed or out of `Int` range.
///
/// The whole string must be an optional `-` followed by one or more digits of `base`; digits
/// past 9 are letters in either case (`'a'` and `'A'` are 10). Whitespace, a `+` sign or any
/// other character makes it malformed.
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `None`
/// traps: when `base` is outside `2..=36`
/// example: `"ff".parse_int(base: 16) == Some(255)`
/// example: `"-12".parse_int() == Some(-12)`
/// example: `"12a".parse_int() == None`
pub fn parse_int(self: Str, base: Int = 10) -> ?Int:
    assert 2 <= base <= 36
    var i = 0
    let neg = self.len > 0 and self[0] == '-'
    if self.len > 0 and (self[0] == '-' or self[0] == '+'): i = 1
    if i == self.len: return None
    var x = 0
    while i < self.len:
        let c = self[i].lower
        let d = if c.is_digit: c.digit elif c.is_lower: Int(c) - Int('a') + 10 else: base
        if d >= base: return None
        x = x * base + (if neg: -d else: d)
        i += 1
    Some(x)

/// `self` read as a `T` by the same rules as `read<T>()` (core 10.2), or `None` if a token is
/// malformed, out of `T`'s range, missing, or left over.
///
/// Tokens are separated by whitespace exactly as on standard input, so a tuple or list type
/// reads several tokens.
///
/// cost: \(O(\lvert \mathit{self} \rvert)\)
/// empty: `None` unless `T` reads zero tokens
/// traps: never
/// example: `"-12".parse<Mod<7>>() == Some(2)`
/// example: `"3 4".parse<(Int, Int)>() == Some((3, 4))`
/// example: `"3 4".parse<Int>() == None`
pub fn parse<T: Read>(self: Str) -> ?T = todo

/// The Z-array of `s`: `z[i]` is the length of the longest common prefix of `s` and the
/// suffix starting at `i`; `z` at the first key is `s.len`.
///
/// Keyed by `s.keys`, so `z_function(s[l..])[i]` takes `i` from the slice. Works on any
/// `Indexed` sequence (`Str`, `List<T>`, `[D]T`, `Range<I>`, views) without conversion.
///
/// cost: \(O(n)\)
/// empty: an empty array
/// order: keyed by the keys of `s`
/// traps: never
/// example: `z_function("aabxaab") == [7, 1, 0, 0, 3, 1, 0]`
pub fn z_function<S: Indexed<K, T>, K, T: Eq>(s: S) -> [K]Int:
    with lint(unproven_list_index: allow)
    let n = s.len
    var z = List.filled(n, 0)
    if n > 0: z[0] = n
    var l = 0
    var r = 0
    for i in 1..n:
        var k = if i < r: min(r - i, z[i - l]) else: 0
        while i + k < n and s.nth(k) == s.nth(i + k): k += 1
        z[i] = k
        if i + k > r:
            l = i
            r = i + k
    z.rekey(s.keys)

/// The prefix function (failure function) of `s`: `p[i]` is the length of the longest
/// proper prefix of `s` up to and including `i` that is also a suffix of it.
///
/// Keyed by `s.keys`. Works on any `Indexed` sequence without conversion.
///
/// cost: \(O(n)\)
/// empty: an empty array
/// order: keyed by the keys of `s`
/// traps: never
/// example: `prefix_function("aabaaab") == [0, 1, 0, 1, 2, 2, 3]`
pub fn prefix_function<S: Indexed<K, T>, K, T: Eq>(s: S) -> [K]Int:
    with lint(unproven_list_index: allow)
    return failure_lengths(s).rekey(s.keys)

fn failure_lengths<S: Indexed>(s: S) -> List<Int>:
    var p = List.filled(s.len, 0)
    for i in 1..s.len:
        var k = p[i - 1]
        while k > 0 and s.nth(i) != s.nth(k): k = p[k - 1]
        if s.nth(i) == s.nth(k): k += 1
        p[i] = k
    p

/// The start key of every occurrence of `pat` in `self`, overlapping ones included
/// (Knuth-Morris-Pratt).
///
/// Works on any `Indexed` pair (`Str`, `List<T>`, `[D]T`, views) without conversion; keys
/// are those of `self`. An empty pattern occurs at every key, never at the end: a match starts at an element.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert \mathit{pat} \rvert)\)
/// empty: an empty `pat` yields every key of `self`; an empty `self` yields nothing
/// ties: every occurrence is yielded; None is preferred
/// order: increasing start key
/// traps: never
/// example: `"aaaa".find_all("aa").collect() == [0, 1, 2]`
/// example: `"ab".find_all("").collect() == [0, 1]`
pub fn find_all<S: Indexed<K, T>, P: Indexed<L, T>, K, L, T: Eq>(self: S, pat: P) -> yields K:
    with lint(unproven_list_index: allow)
    if pat.is_empty:
        for k in self.keys: yield k
        return
    let p = failure_lengths(pat)
    var k = 0
    for i in 0..self.len:
        while k > 0 and self.nth(i) != pat.nth(k): k = p[k - 1]
        if self.nth(i) == pat.nth(k): k += 1
        if k == pat.len:
            yield self.keys.nth(i + 1 - k)
            k = p[k - 1]

/// The length of the longest common prefix of `self` and `other`, comparing values in order
/// from the first key of each. `other` is only walked, so it can be any `Seq` (`s.rev`, a
/// generator); `self` is `Indexed`.
///
/// cost: \(O(\min(\lvert \mathit{self} \rvert, \lvert \mathit{other} \rvert))\)
/// empty: `0` when either is empty
/// traps: never
/// example: `"abcd".common_prefix_len("abxd") == 2`
/// example: `"abcx".common_prefix_len("xcba".rev) == 4`
pub fn common_prefix_len<A: Indexed, B: Seq>(self: A, other: B) -> Int:
    with lint(unproven_list_index: allow)
    var i = 0
    for y in other:
        if i == self.len or self.nth(i) != y: break
        i += 1
    i

/// The length of the longest common suffix of `self` and `other`, comparing values in order
/// from the last key of each. Works on any `Indexed` pair.
///
/// cost: \(O(\min(\lvert \mathit{self} \rvert, \lvert \mathit{other} \rvert))\)
/// empty: `0` when either is empty
/// traps: never
/// example: `"xbcd".common_suffix_len("abcd") == 3`
pub fn common_suffix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int:
    with lint(unproven_list_index: allow)
    let (na, nb) = (self.len, other.len)
    let lim = min(na, nb)
    var i = 0
    while i < lim and self.nth(na - 1 - i) == other.nth(nb - 1 - i): i += 1
    i

/// The shortest period of `s`: the smallest `d >= 1` with `s[i] == s[i + d]` wherever both
/// sides exist. It is `s.len` when nothing shorter works.
///
/// cost: \(O(n)\)
/// empty: `0`
/// traps: never
/// example: `period("abcabca") == 3`
/// example: `period("aaaa") == 1`
/// example: `period("abc") == 3`
pub fn period<S: Indexed>(s: S) -> Int = if s.is_empty: 0 else: s.len - failure_lengths(s).last

/// True when `s` reads the same forwards and backwards. Works on any `Indexed` sequence.
///
/// cost: \(O(n)\)
/// empty: `true`
/// traps: never
/// example: `is_palindrome("abba") == true`
/// example: `is_palindrome([1, 2]) == false`
pub fn is_palindrome<S: Indexed>(s: S) -> Bool:
    with lint(unproven_list_index: allow)
    let n = s.len
    for i in 0..n / 2:
        if s.nth(i) != s.nth(n - 1 - i): return false
    true

/// The length of every non-empty proper border of `s` (a prefix that is also a suffix,
/// shorter than `s`). Works on any `Indexed` sequence.
///
/// cost: \(O(n)\)
/// empty: yields nothing; so does a single element
/// order: longest first
/// traps: never
/// example: `borders("abacaba").collect() == [3, 1]`
/// example: `borders("aaa").collect() == [2, 1]`
pub fn borders<S: Indexed>(s: S) -> yields Int:
    with lint(unproven_list_index: allow)
    if s.is_empty: return
    let pi = failure_lengths(s)
    var k = pi[s.len - 1]
    while k > 0:
        yield k
        k = pi[k - 1]

/// For each key `i` of `self`, the length of the longest prefix of `pat` that ends at `i`
/// (the KMP matcher's state after reading `self[i]`).
///
/// A full match counts as `pat.len`; matching then continues from the longest border of
/// `pat`, so `res[i] == pat.len` exactly where an occurrence ends. Keyed by `self.keys`.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert \mathit{pat} \rvert)\)
/// empty: an empty `self` gives an empty array; an empty `pat` gives all zeros
/// order: keyed by the keys of `self`
/// traps: never
/// example: `"abab".prefix_matches("ab") == [1, 2, 1, 2]`
pub fn prefix_matches<S: Indexed<K, T>, P: Indexed<J, T>, K, J, T: Eq>(self: S, pat: P) -> [K]Int:
    with lint(unproven_list_index: allow)
    var res = List.filled(self.len, 0)
    if pat.is_empty: return res.rekey(self.keys)
    let p = failure_lengths(pat)
    var k = 0
    for i in 0..self.len:
        if k == pat.len: k = p[k - 1]
        while k > 0 and self.nth(i) != pat.nth(k): k = p[k - 1]
        if self.nth(i) == pat.nth(k): k += 1
        res[i] = k
    res.rekey(self.keys)

/// The length of the longest suffix of `a` that is also a prefix of `b`, at most
/// `min(|a|, |b|)`; how far `b` can slide onto the end of `a`.
///
/// cost: \(O(\lvert a \rvert + \lvert b \rvert)\)
/// empty: `0` when either is empty
/// traps: never
/// example: `overlap("abcab", "abd") == 2`
/// example: `overlap("abc", "abc") == 3`
pub fn overlap<A: Indexed, B: Indexed>(a: A, b: B) -> Int:
    with lint(unproven_list_index: allow)
    let keep = min(a.len, b.len)
    if keep == 0: return 0
    let pi = failure_lengths([b.nth(i) for i in 0..keep])
    var k = 0
    for i in a.len - keep..a.len:
        while k > 0 and (k == keep or a.nth(i) != b.nth(k)): k = pi[k - 1]
        if a.nth(i) == b.nth(k): k += 1
    k

/// The start key of each factor of the Lyndon factorization of `s` (Duval): the unique way
/// to cut `s` into Lyndon words `w1 >= w2 >= ...` in lexicographic order.
///
/// cost: \(O(n)\)
/// empty: `[]`
/// order: increasing keys; the first is the first key of `s`
/// traps: never
/// example: `lyndon_factorization("banana") == [0, 1, 3, 5]`
pub fn lyndon_factorization<S: Indexed<K, T>, K, T>(s: S) -> List<K>:
    with lint(unproven_list_index: allow)
    let n = s.len
    var starts: List<K> = []
    var i = 0
    unchecked:
        while i < n:
            var j = i + 1
            var k = i
            while j < n and s.nth(k) <= s.nth(j):
                if s.nth(k) < s.nth(j): k = i else: k += 1
                j += 1
            let step = j - k
            while i <= k:
                starts.push(s.keys.nth(i))
                i += step
    starts

/// The key at which the lexicographically smallest cyclic rotation of `s` starts (Booth).
///
/// cost: \(O(n)\)
/// empty: `None`
/// ties: the smallest key, when `s` is periodic
/// traps: never
/// example: `min_rotation("baca") == Some(3)`
/// example: `min_rotation("abab") == Some(0)`
pub fn min_rotation<S: Indexed<K, T>, K, T>(s: S) -> ?K:
    with lint(unproven_list_index: allow)
    let n = s.len
    if n == 0: return None
    var (a, b) = (0, 1)
    var k = 0
    unchecked:
        while a < n and b < n and k < n:
            let ai = a + k
            let bi = b + k
            let x = s.nth(if ai < n: ai else: ai - n)
            let y = s.nth(if bi < n: bi else: bi - n)
            if x == y:
                k += 1
            else:
                if x > y: a += k + 1 else: b += k + 1
                if a == b: b += 1
                k = 0
    Some(s.keys.nth(min(a, b)))

/// The key at which the lexicographically greatest cyclic rotation of `s` starts.
///
/// cost: \(O(n)\)
/// empty: `None`
/// ties: the smallest key, when `s` is periodic
/// traps: never
/// example: `max_rotation("baca") == Some(2)`
/// example: `max_rotation("abab") == Some(1)`
pub fn max_rotation<S: Indexed<K, T>, K, T>(s: S) -> ?K:
    let n = s.len
    if n == 0: return None
    var (a, b) = (0, 1)
    var k = 0
    unchecked:
        while a < n and b < n and k < n:
            let ai = a + k
            let bi = b + k
            let x = s.nth(if ai < n: ai else: ai - n)
            let y = s.nth(if bi < n: bi else: bi - n)
            if x == y:
                k += 1
            else:
                if x < y: a += k + 1 else: b += k + 1
                if a == b: b += 1
                k = 0
    Some(s.keys.nth(min(a, b)))

/// \(O(\lvert \mathit{pat} \rvert \cdot \lvert A \rvert)\). Prefix (KMP) automaton over alphabet domain `A`, for \(O(1)\) transitions in DP.
///
/// not a container: an automaton; `len` counts states, which are walked by `step`.
pub struct KmpAutomaton<A>:
    alpha: 0..=2_000_000_000
    goto: List<0..=2_000_000_000>
    pi: List<0..=2_000_000_000>

    /// The KMP automaton of `pat` over the alphabet domain `A`: one state per matched prefix
    /// length `0..=pat.len`, with a full transition table. Call it as `KmpAutomaton<A>(pat)`.
    ///
    /// cost: \(O(\lvert \mathit{pat} \rvert \cdot \lvert A \rvert)\)
    /// empty: an empty `pat` gives one state, `0`, which matches everywhere
    /// traps: when an element of `pat` is not in `A`
    /// example: `KmpAutomaton<Char>("aba").len == 4`
    pub fn new<S: Seq>(pat: S) -> KmpAutomaton<A>:
        with lint(unproven_list_index: allow)
        let k = A.count
        var p: List<Int> = []
        for a in pat:
            let c = a.ord
            assert 0 <= c < k, "KmpAutomaton: character outside alphabet"
            p.push(c)
        let m = p.len
        var pi: List<0..=2_000_000_000> = List.filled(m + 1, 0)
        var goto: List<0..=2_000_000_000> = List.filled((m + 1) * k, 0)
        unchecked:
            for i in 1..m:
                var j = pi[i]
                while j > 0 and p[i] != p[j]: j = pi[j]
                if p[i] == p[j]: j += 1
                pi[i + 1] = j
            for i in 0..=m:
                for c in 0..k:
                    if i > 0 and (i == m or c != p[i]):
                        goto[i * k + c] = goto[pi[i] * k + c]
                    else:
                        goto[i * k + c] = if i < m and c == p[i]: i + 1 else: 0
        KmpAutomaton(k, goto, pi)

    /// The number of states, `pat.len + 1`; the states are `0..=pat.len` and `pat.len` means a
    /// full match.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `KmpAutomaton<Char>("ab").len == 3`
    @get pub fn len(self) -> Int = pi.len

    /// The matched prefix length after reading `a` in `state`. From the full-match state the
    /// automaton falls back along `link` first, so matching continues and overlaps are found.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let k = KmpAutomaton<Char>("aba"); k.step(3, 'b') == 2 and k.step(0, 'b') == 0`
    pub fn step(self, state: Int, a: A) -> Int:
        with lint(unproven_list_index: allow)
        assert 0 <= state < pi.len, "KmpAutomaton.step: state out of range"
        let c = a.ord
        assert 0 <= c < alpha, "KmpAutomaton.step: character outside alphabet"
        goto[state * alpha + c]

    /// The failure link of `state`: the length of the longest proper border of `pat[..state]`,
    /// and `0` at state `0`. It equals `prefix_function(pat)[state - 1]` for `state >= 1`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let k = KmpAutomaton<Char>("aba"); k.link(3) == 1 and k.link(2) == 0`
    pub fn link(self, state: Int) -> Int:
        with lint(unproven_list_index: allow)
        assert 0 <= state < pi.len, "KmpAutomaton.link: state out of range"
        pi[state]

    /// The start position (0-based, counted in `text`'s iteration order) of every occurrence of
    /// `pat` in `text`, overlapping ones included.
    ///
    /// cost: \(O(\lvert \mathit{text} \rvert)\)
    /// empty: an empty `text` yields nothing unless `pat` is empty; an empty `pat` yields `0..=|text|`
    /// ties: every occurrence is yielded; None is preferred
    /// order: increasing start
    /// traps: when an element of `text` is not in `A`
    /// example: `KmpAutomaton<Char>("aba").find_all("ababa").collect() == [0, 2]`
    pub fn find_all<S: Seq>(self, text: S) -> yields Int:
        with lint(unproven_list_index: allow)
        let m = pi.len - 1
        if m == 0:
            var pos = 0
            yield 0
            for a in text:
                let c = a.ord
                assert 0 <= c < alpha, "KmpAutomaton.find_all: character outside alphabet"
                pos += 1
                yield pos
            return
        var state = 0
        var pos = 0
        for a in text:
            let c = a.ord
            assert 0 <= c < alpha, "KmpAutomaton.find_all: character outside alphabet"
            state = goto[state * alpha + c]
            pos += 1
            if state == m: yield pos - m

/// The start position of every occurrence of `pat` in `self`, overlapping ones included,
/// by bit-parallel Shift-And. Faster than `find_all` for short patterns on small alphabets.
///
/// The starts are keys of `self`, as for `find_all`.
///
/// cost: \(O(\lvert \mathit{self} \rvert \cdot \mathit{ceil}(\lvert \mathit{pat} \rvert / 64))\)
/// empty: an empty `pat` yields every key of `self`; an empty `self` yields nothing
/// order: increasing start
/// traps: never
/// example: `"abab".shift_and("ab").collect() == [0, 2]`
pub fn shift_and<S: Indexed<K, T>, P: Indexed<L, T>, K, L, T: Eq>(self: S, pat: P) -> yields K:
    for i in find_all(self, pat): yield i

/// The start key of every place where `pat` matches `self` when each `wildcard` in
/// `pat` matches any one element. A `wildcard` in `self` is an ordinary element.
///
/// Keys are those of `self`, as for `find_all`; an empty pattern matches at every key, never at the end.
///
/// cost: \(O(\lvert \mathit{self} \rvert \cdot \lvert \mathit{pat} \rvert)\) worst case, comparing directly
/// empty: an empty `pat` yields every key of `self`; an empty `self` yields nothing
/// ties: every match is yielded; None is preferred
/// order: increasing start key
/// traps: never
/// example: `"abcab".find_all_wildcard("a?").collect() == [0, 3]`
/// example: `"a?c".find_all_wildcard("ab").collect() == []`
pub fn find_all_wildcard<S: Indexed<K, Char>, P: Indexed<L, Char>, K, L>(
    self: S,
    pat: P,
    wildcard: Char = '?',
) -> yields K:
    let (n, m) = (self.len, pat.len)
    if m == 0:
        for k in self.keys: yield k
        return
    for i in 0..=n - m:
        var ok = true
        for j in 0..m:
            let c = pat.nth(j)
            if c != wildcard and c != self.nth(i + j):
                ok = false
                break
        if ok: yield self.keys.nth(i)

/// \(O(n)\) space, \(O(1)\) queries. Manacher palindrome radii. `odd[i]` is the number of palindromes centred on `s[i]`;
/// `even[i]` is the number centred between `s[i - 1]` and `s[i]`.
pub struct Palindromes(pub odd: List<Int>, even: List<Int>, first: Int = 0):
    /// The length of the sequence the radii were computed for.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `manacher("aba").len == 3`
    @get pub fn len(self) -> Int = odd.len

    /// True when the sequence the radii were computed for is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `manacher("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = odd.is_empty

    /// The length of the longest odd-length palindrome centred on the element at key `i` of the
    /// sequence the radii were computed for, so a slice is asked in its own keys (R04b.5.6).
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is not one of the sequence's keys
    /// example: `manacher("abacaba").longest_at(3) == 7`
    /// example: `manacher("xaba"[1..]).longest_at(2) == 3`
    pub fn longest_at<K>(self, i: K) -> Int:
        with lint(unproven_list_index: allow)
        let j = i.ord - first
        assert 0 <= j < odd.len, "`longest_at(i)` needs `i` to be one of the sequence's keys"
        2 * odd[j] - 1

    /// The length of the longest even-length palindrome centred on the cut `i` (between
    /// positions `i - 1` and `i`), `2 * even[i]`; `0` at the cuts `0` and `self.len`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is outside `0..=self.len`
    /// example: `manacher("abba").longest_between(2) == 4`
    pub fn longest_between(self, i: Int) -> Int:
        with lint(unproven_list_index: allow)
        assert 0 <= i <= odd.len, "`longest_between(i)` needs `0 <= i <= len`"
        if i == odd.len: 0 else: 2 * even[i]

    /// True when the positions `r` (`l..r`, `l..=r`, or a `Range<D>`) of the original sequence
    /// form a palindrome: `p.is_palindrome(l..r) == is_palindrome(s[l..r])`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `r` reaches outside `0..=self.len`
    /// example: `let p = manacher("abacaba"); p.is_palindrome(0..3) and not p.is_palindrome(1..4)`
    pub fn is_palindrome<R>(self, r: R) -> Bool:
        if r.is_empty: return true
        let l = r.start.ord
        let hi = r.end.ord
        let n = odd.len
        assert 0 <= l <= hi <= n, "`is_palindrome(r)` needs `0 <= lo <= hi <= len`"
        let m = hi - l
        if m <= 1: return true
        if m % 2 == 1: odd[(l + hi) / 2] * 2 - 1 >= m else: even[(l + hi) / 2] * 2 >= m

    /// The number of non-empty palindromic substrings, each occurrence counted:
    /// `sum(odd) + sum(even)`.
    ///
    /// cost: \(O(n)\)
    /// traps: never
    /// example: `manacher("aaa").count() == 6`
    pub fn count(self) -> Int:
        var total = sum(x for x in odd)
        for x in even: total += x
        total

    /// The half-open range `(l, r)` of a longest palindromic substring.
    ///
    /// cost: \(O(n)\)
    /// ties: the leftmost (smallest `l`)
    /// traps: never
    /// example: `manacher("abacdc").longest() == (0, 3)`
    /// example: `manacher("").longest() == (0, 0)`
    pub fn longest(self) -> (l: Int, r: Int):
        var (best_l, best_r) = (0, 0)
        unchecked:
            for i in 0..odd.len:
                let olen = odd[i] * 2 - 1
                if olen > best_r - best_l:
                    best_l = i - odd[i] + 1
                    best_r = i + odd[i]
                let elen = even[i] * 2
                if elen > best_r - best_l:
                    best_l = i - even[i]
                    best_r = i + even[i]
        (l: best_l, r: best_r)

/// The palindrome radii of `s` by Manacher's algorithm: `odd[i]` is the largest `k` with
/// `s[i - k + 1..i + k]` a palindrome, and `even[i]` the largest `k` with `s[i - k..i + k]`
/// one. Works on any `Indexed` sequence; positions count from 0.
///
/// cost: \(O(n)\)
/// empty: both lists empty
/// traps: never
/// example: `manacher("abba").count() == 6`
/// example: `manacher("aba").odd == [1, 2, 1]`
pub fn manacher<S: Indexed>(s: S) -> Palindromes:
    with lint(unproven_list_index: allow)
    let n = s.len
    var odd = List.filled(n, 0)
    var even = List.filled(n, 0)
    var (l, r) = (0, -1)
    unchecked:
        for i in 0..n:
            var k = if i > r: 1 else: min(odd[l + r - i], r - i + 1)
            while i - k >= 0 and i + k < n and s.nth(i - k) == s.nth(i + k): k += 1
            odd[i] = k
            if i + k - 1 > r: (l, r) = (i - k + 1, i + k - 1)
        (l, r) = (0, -1)
        for i in 0..n:
            var k = if i > r: 0 else: min(even[l + r - i + 1], r - i + 1)
            while i - k - 1 >= 0 and i + k < n and s.nth(i - k - 1) == s.nth(i + k): k += 1
            even[i] = k
            if i + k - 1 > r: (l, r) = (i - k, i + k - 1)
    Palindromes(odd, even, if n == 0: 0 else: s.keys.first!.ord)

fn induced_sort(
    vec: List<Int>,
    counts: List<Int>,
    is_l: List<Bool>,
    lms: List<Int>,
    mut sa: List<Int>,
):
    let n = vec.len
    let alpha = counts.len
    var bkt = List.filled(alpha, 0)
    unchecked:
        var sum = 0
        for c in 0..alpha:
            sum += counts[c]
            bkt[c] = sum
        for i in 0..n: sa[i] = -1
        for k in (0..lms.len).rev:
            let pos = lms[k]
            let c = vec[pos]
            bkt[c] -= 1
            sa[bkt[c]] = pos
        sum = 0
        for c in 0..alpha:
            bkt[c] = sum
            sum += counts[c]
        for k in 0..n:
            let p = sa[k] - 1
            if p >= 0 and is_l[p]:
                let c = vec[p]
                sa[bkt[c]] = p
                bkt[c] += 1
        sum = 0
        for c in 0..alpha:
            sum += counts[c]
            bkt[c] = sum
        for k in (1..n).rev:
            let p = sa[k] - 1
            if p >= 0 and not is_l[p]:
                let c = vec[p]
                bkt[c] -= 1
                sa[bkt[c]] = p

fn sa_is(vec: List<Int>, alpha: Int) -> List<Int>:
    with lint(unproven_list_index: allow)
    let n = vec.len
    var sa = List.filled(n, 0)
    if n <= 1: return sa
    var counts = List.filled(alpha, 0)
    var is_l = List.filled(n, false)
    var lms = List<Int>()
    unchecked:
        for c in vec: counts[c] += 1
        for i in (0..n - 1).rev:
            is_l[i] = vec[i] > vec[i + 1] or (vec[i] == vec[i + 1] and is_l[i + 1])
            if is_l[i] and not is_l[i + 1]: lms.push(i + 1)
    unchecked:
        for i in 0..lms.len / 2:
            let t = lms[i]
            lms[i] = lms[lms.len - 1 - i]
            lms[lms.len - 1 - i] = t
    induced_sort(vec, counts, is_l, lms, mut sa)
    let m = lms.len
    var sorted_lms = List.filled(m, 0)
    var name = 0
    unchecked:
        var k_lms = 0
        for x in sa:
            if x >= 1 and not is_l[x] and is_l[x - 1]:
                sorted_lms[k_lms] = x
                k_lms += 1
        sa[n - 1] = 0
        for k in 1..m:
            let i = sorted_lms[k - 1]
            let j = sorted_lms[k]
            var diff = vec[i] != vec[j]
            if not diff:
                var (a, b) = (i + 1, j + 1)
                while true:
                    if vec[a] != vec[b]:
                        diff = true
                        break
                    let la = not is_l[a] and is_l[a - 1]
                    let lb = not is_l[b] and is_l[b - 1]
                    if la or lb:
                        diff = not (la and lb)
                        break
                    a += 1
                    b += 1
            if diff: name += 1
            sa[j] = name
    if name + 1 < m:
        var lms_vec = List.filled(m, 0)
        unchecked:
            for i in 0..m: lms_vec[i] = sa[lms[i]]
        let lms_sa = sa_is(lms_vec, name + 1)
        unchecked:
            for i in 0..m: sorted_lms[i] = lms[lms_sa[i]]
    induced_sort(vec, counts, is_l, sorted_lms, mut sa)
    sa

fn sa_order<S: Indexed>(s: S) -> List<Int>:
    with lint(unproven_list_index: allow)
    let n = s.len
    if n <= 1: return [i for i in 0..n]
    var vec = List.filled(n + 1, 0)
    var alpha = 0
    var uniq = [s.nth(0)]
    var ids = [0]
    var small = true
    unchecked:
        for i in 1..n:
            let x = s.nth(i)
            var (lo, hi) = (0, uniq.len)
            while lo < hi:
                let mid = (lo + hi) >> 1
                if uniq[mid] < x: lo = mid + 1 else: hi = mid
            if lo < uniq.len and uniq[lo] == x:
                vec[i] = ids[lo]
            elif uniq.len == 256:
                small = false
                break
            else:
                let id = uniq.len
                uniq.push(x)
                ids.push(id)
                for j in (lo + 1..uniq.len).rev:
                    uniq[j] = uniq[j - 1]
                    ids[j] = ids[j - 1]
                uniq[lo] = x
                ids[lo] = id
                vec[i] = id
    if small:
        alpha = uniq.len
        if alpha == 1: return [n - 1 - i for i in 0..n]
        var remap = List.filled(alpha, 0)
        unchecked:
            for r in 0..alpha: remap[ids[r]] = r + 1
            for i in 0..n: vec[i] = remap[vec[i]]
    else:
        var order = [i for i in 0..n]
        order.sort(by: i => s.nth(i))
        alpha = 1
        unchecked:
            vec[order[0]] = 1
            for j in 1..n:
                if s.nth(order[j - 1]) < s.nth(order[j]): alpha += 1
                vec[order[j]] = alpha
        if alpha == n: return order
    var sa = sa_is(vec, alpha + 1)
    unchecked:
        for i in 0..n: sa[i] = sa[i + 1]
    let _ = sa.pop()
    sa

/// \(O(n)\) SA-IS + Kasai LCP (\(O(n \log n)\) with RMQ). Suffix array, inverse rank, adjacent LCP, and \(O(1)\) LCP/comparison queries.
pub struct SuffixArray:
    pub sa: List<Int>
    rank: List<Int>
    pub lcp: List<Int>
    rmq: List<Int>
    bits: Int
    dcum: List<Int>
    toks: List<Int>

    /// The suffix array of `s` with its inverse and LCP array: `sa[r]` is the start of the suffix
    /// of rank `r`, `rank[sa[r]] == r`, and `lcp[r]` is the LCP of the suffixes of ranks `r` and `r
    /// + 1`.
    ///
    /// Call it as `SuffixArray(s)`; positions count from 0.
    ///
    /// Elements are compared by `Ord`, so a shorter suffix that is a prefix of a longer one
    /// ranks first.
    ///
    /// cost: \(O(n)\) for `Str` and small integer alphabets, \(O(n \log n)\) otherwise
    /// empty: `sa`, `rank` and `lcp` all empty
    /// traps: never
    /// example: `SuffixArray("banana").sa == [5, 3, 1, 0, 4, 2]`
    /// example: `SuffixArray("banana").lcp == [1, 3, 0, 0, 2]`
    pub fn new<S: Indexed>(s: S) -> SuffixArray:
        let n = s.len
        let sa = sa_order(s)
        var rank = List.filled(n, 0)
        unchecked:
            for j in 0..n: rank[sa[j]] = j
        var lcp = List.filled(max(n - 1, 0), 0)
        unchecked:
            var k = 0
            for i in 0..n:
                if rank[i] == n - 1:
                    k = 0
                    continue
                let j = sa[rank[i] + 1]
                while i + k < n and j + k < n and s.nth(i + k) == s.nth(j + k): k += 1
                lcp[rank[i]] = k
                if k > 0: k -= 1
        // Each sparse-table cell packs `lcp[i] << bits | i`, so one `min` picks the
        // smallest lcp and, among equal ones, the leftmost position.
        let m = lcp.len
        let bits = bit_width(m)
        let levels = bits
        var rmq = List.filled(levels * m, 0)
        unchecked:
            for i in 0..m: rmq[i] = (lcp[i] << bits) | i
            for b in 1..levels:
                let step = 1 << (b - 1)
                for i in 0..m - (1 << b) + 1:
                    rmq[b * m + i] = min(rmq[(b - 1) * m + i], rmq[(b - 1) * m + i + step])
        var dcum = List.filled(n + 1, 0)
        unchecked:
            for r in 0..n:
                let shared = if r == 0: 0 else: lcp[r - 1]
                dcum[r + 1] = dcum[r] + (n - sa[r]) - shared
        var toks = List.filled(n, 0)
        unchecked:
            for i in 0..n: toks[i] = s.nth(i).hash_token
        SuffixArray(sa, rank, lcp, rmq, bits, dcum, toks)

    /// The length `n` of the sequence the suffix array was built over.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SuffixArray("banana").len == 6`
    @get pub fn len(self) -> Int = sa.len

    /// True when the suffix array was built over an empty sequence.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SuffixArray("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = sa.is_empty

    /// The length of the longest common prefix of the suffixes starting at `i` and `j`. The
    /// empty suffix at `n` is allowed, and `lcp_between(i, i) == n - i`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` or `j` is outside `0..=self.len`
    /// example: `SuffixArray("banana").lcp_between(1, 3) == 3`
    pub fn lcp_between(self, i: Int, j: Int) -> Int:
        let n = sa.len
        assert 0 <= i and i <= n and 0 <= j and j <= n, "`lcp_between(i, j)` needs `0 <= i, j <= len`"
        if i == j: return n - i
        if i == n or j == n: return 0
        self.packed_min(min(rank[i], rank[j]), max(rank[i], rank[j])) >> bits

    /// The ranks `lo..hi` of the suffixes that start with `pat`. When there are None it is the
    /// empty range at the rank where `pat` would sort.
    ///
    /// cost: \(O(\lvert \mathit{pat} \rvert \log n)\)
    /// empty: an empty `pat` gives `0..n`
    /// order: a range of ranks, increasing
    /// traps: never
    /// example: `SuffixArray("banana").rank_range("ana") == 1..3`
    /// example: `SuffixArray("banana").rank_range("c") == 4..4`
    pub fn rank_range<P: Indexed>(self, pat: P) -> Range<Int>:
        with lint(unproven_list_index: allow)
        let n = sa.len
        let m = pat.len
        if m == 0: return 0..n
        var ptok = List.filled(m, 0)
        unchecked:
            for k in 0..m: ptok[k] = pat.nth(k).hash_token
        var lo = 0
        var hi = n
        unchecked:
            while lo < hi:
                let mid = (lo + hi) / 2
                let start = sa[mid]
                let lim = min(n - start, m)
                var k = 0
                while k < lim and toks[start + k] == ptok[k]: k += 1
                let less = if k < lim: toks[start + k] < ptok[k] else: n - start < m
                if less: lo = mid + 1 else: hi = mid
        let left = lo
        lo = left
        hi = n
        unchecked:
            while lo < hi:
                let mid = (lo + hi) / 2
                let start = sa[mid]
                let lim = min(n - start, m)
                var k = 0
                while k < lim and toks[start + k] == ptok[k]: k += 1
                let greater = if k < lim: toks[start + k] > ptok[k] else: false
                if greater: hi = mid else: lo = mid + 1
        left..lo

    /// The start of every occurrence of `pat`, read off `sa` over `rank_range(pat)`.
    ///
    /// cost: \(O(\lvert \mathit{pat} \rvert \log n + \mathit{occurrences})\)
    /// empty: an empty `pat` yields every start `0..n` (not the end position `n`)
    /// ties: every occurrence is yielded; None is preferred
    /// order: suffix-array order (by rank), not by position
    /// traps: never
    /// example: `SuffixArray("banana").find_all("ana").collect() == [3, 1]`
    pub fn find_all<P: Indexed>(self, pat: P) -> yields Int:
        with lint(unproven_list_index: allow)
        for r in self.rank_range(pat): yield sa[r]

    /// The number of occurrences of `pat`, overlapping ones included: `rank_range(pat).len`.
    ///
    /// cost: \(O(\lvert \mathit{pat} \rvert \log n)\)
    /// empty: an empty `pat` gives `n`
    /// traps: never
    /// example: `SuffixArray("banana").count("an") == 2`
    pub fn count<P: Indexed>(self, pat: P) -> Int = self.rank_range(pat).len

    /// The ranks `lo..hi` of the suffixes whose LCP with the suffix at `i` is at least `len`;
    /// always contains `rank[i]`.
    ///
    /// cost: \(O(\log n)\)
    /// order: a range of ranks, increasing; it holds `rank[i]`
    /// traps: when `i` is outside `0..n` or `len` outside `0..=n - i`
    /// example: `SuffixArray("banana").lcp_range(1, 3) == 1..3`
    pub fn lcp_range(self, i: Int, len: Int) -> Range<Int>:
        let n = sa.len
        assert 0 <= i and i < n and 0 <= len and len <= n - i, "`lcp_range(i, len)` needs `0 <= i < n` and `0 <= len <= n - i`"
        let p = rank[i]
        var (lo, hi) = (0, p)
        while lo < hi:
            let mid = (lo + hi) / 2
            if lcp[self.arg_min(mid, p)] >= len: hi = mid else: lo = mid + 1
        let left = lo
        (lo, hi) = (p, n - 1)
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if lcp[self.arg_min(p, mid)] >= len: lo = mid else: hi = mid - 1
        left..lo + 1

    /// The lexicographic comparison of `s[l1..r1]` and `s[l2..r2]`: `-1`, `0` or `1`. A proper
    /// prefix compares less.
    ///
    /// cost: \(O(1)\)
    /// traps: when a range is not within `0..=n` or has its end before its start
    /// example: `SuffixArray("banana").compare(1, 3, 3, 5) == 0`
    /// example: `SuffixArray("banana").compare(1, 2, 1, 3) == -1`
    pub fn compare(self, l1: Int, r1: Int, l2: Int, r2: Int) -> Int:
        let n = sa.len
        assert 0 <= l1 <= r1 <= n, "`compare` needs `0 <= l1 <= r1 <= len`"
        assert 0 <= l2 <= r2 <= n, "`compare` needs `0 <= l2 <= r2 <= len`"
        let (n1, n2) = (r1 - l1, r2 - l2)
        let m = min(n1, n2)
        if m == 0 or self.lcp_between(l1, l2) >= m: return if n1 < n2: -1 elif n1 > n2: 1 else: 0
        if rank[l1] < rank[l2]: -1 else: 1

    fn arg_min(self, lo: Int, hi: Int) -> Int = self.packed_min(lo, hi) & ((1 << bits) - 1)

    fn packed_min(self, lo: Int, hi: Int) -> Int:
        with lint(unproven_list_index: allow)
        let b = bit_width(hi - lo) - 1
        let m = lcp.len
        min(rmq[b * m + lo], rmq[b * m + hi - (1 << b)])

    /// The number of distinct non-empty substrings: `n * (n + 1) / 2 - sum(lcp)`.
    ///
    /// cost: \(O(n)\)
    /// traps: never
    /// example: `SuffixArray("banana").distinct() == 15`
    pub fn distinct(self) -> Int = dcum[sa.len]

    /// The longest prefix common to every suffix with rank in `r`: the minimum of `lcp[j]` for
    /// `j in r.start..r.end - 1`, or the suffix's own length when `r` holds one rank.
    ///
    /// cost: \(O(1)\)
    /// empty: an empty `r` traps
    /// traps: when `r` is empty or reaches outside `0..n`
    /// example: `SuffixArray("banana").lcp_of_ranks(1..3) == 3`
    /// example: `SuffixArray("banana").lcp_of_ranks(3..4) == 6`
    pub fn lcp_of_ranks(self, r: Range<Int>) -> Int:
        assert 0 <= r.start and r.start < r.end and r.end <= sa.len, "`lcp_of_ranks(r)` needs a non-empty `r` within `0..len`"
        if r.end - r.start == 1: sa.len - sa[r.start] else: lcp[self.arg_min(r.start, r.end - 1)]

    /// Where the rank range `r` splits in the suffix tree: the `k` in `r.start + 1..r.end` whose
    /// adjacent LCP `lcp[k - 1]` equals `lcp_of_ranks(r)`, so `r.start..k` and `k..r.end` are
    /// the first child and the rest.
    ///
    /// cost: \(O(1)\)
    /// empty: an `r` with fewer than two ranks traps
    /// ties: the smallest such `k`
    /// traps: when `r.len < 2` or `r` reaches outside `0..n`
    /// example: `SuffixArray("banana").split(0..3) == 1`
    /// example: `SuffixArray("banana").split(0..6) == 3`
    pub fn split(self, r: Range<Int>) -> Int:
        assert 0 <= r.start and r.start + 2 <= r.end and r.end <= sa.len, "`split(r)` needs at least two ranks within `0..len`"
        self.arg_min(r.start, r.end - 1) + 1

    /// The `k`-th (0-based) lexicographically smallest distinct non-empty substring, as the
    /// positions `l..r` of one occurrence, or `None` when `k >= distinct()`.
    ///
    /// cost: \(O(\log n)\)
    /// ties: the occurrence in the suffix of smallest rank that holds it
    /// order: `k` counts in increasing lexicographic order; the range is increasing positions
    /// traps: when `k < 0`
    /// example: `SuffixArray("banana").kth_distinct(1) == Some(3..5)`
    /// example: `SuffixArray("banana").kth_distinct(15) == None`
    pub fn kth_distinct(self, k: Int) -> ?Range<Int>:
        assert k >= 0, "`kth_distinct(k)` needs `k >= 0`"
        let n = sa.len
        if k >= dcum[n]: return None
        let lo = (1..=n - 1).last_true(mid => dcum[mid] <= k) ?? 0
        let shared = if lo == 0: 0 else: lcp[lo - 1]
        Some(sa[lo]..sa[lo] + shared + (k - dcum[lo]) + 1)

/// The suffix array of `s`; the same as `SuffixArray(s)`.
///
/// cost: \(O(n)\) for `Str` and small integer alphabets, \(O(n \log n)\) otherwise
/// empty: an empty suffix array
/// traps: never
/// example: `suffix_array([3, 1, 2]).sa == [1, 2, 0]`
pub fn suffix_array<S: Indexed>(s: S) -> SuffixArray = SuffixArray.new(s)

/// \(O(1)\) value. One edge of the compressed suffix tree of a sequence, built in \(O(n)\) from SA + LCP.
pub struct SuffixTreeEdge(
    pub top_size: Int,
    pub bottom_size: Int,
    pub parent_len: Int,
    pub edge_len: Int,
    pub rank_range: Range<Int>,
)

/// Every edge of the compressed suffix tree of `s` (built from the suffix array and LCP
/// array), one `SuffixTreeEdge` per edge of non-zero length.
///
/// For an edge into node `v`: `parent_len` is the string depth of its upper node,
/// `edge_len` the number of characters on it, `rank_range` the ranks of the suffixes in
/// `v`'s subtree (how often each substring on the edge occurs is `rank_range.len`),
/// `top_size` the total `edge_len` of all edges below `v`, and `bottom_size` that plus this
/// edge's `edge_len`. Summing `edge_len` gives `SuffixArray(s).distinct()`.
///
/// cost: \(O(n)\)
/// empty: `[]`
/// order: bottom-up: each edge comes after every edge below it, subtrees in rank order
/// traps: never
/// example: `suffix_tree_edges("aa") == [SuffixTreeEdge(0, 1, 1, 1, 1..2), SuffixTreeEdge(1, 2, 0, 1, 0..2)]`
pub fn suffix_tree_edges<S: Indexed>(s: S) -> List<SuffixTreeEdge>:
    with lint(unproven_list_index: allow)
    let n = s.len
    if n == 0: return []
    let sa_obj = SuffixArray.new(s)
    let sa = sa_obj.sa
    let lcp = sa_obj.lcp
    var out: List<SuffixTreeEdge> = []
    var st_depth: List<Int> = []
    var st_start: List<Int> = []
    var st_sum: List<Int> = []
    for i in 0..n:
        let leaf_d = n - sa[i]
        if st_depth.is_empty or st_depth.last! < leaf_d:
            st_depth.push(leaf_d)
            st_start.push(i)
            st_sum.push(0)
        let h = if i + 1 < n: lcp[i] else: 0
        while not st_depth.is_empty and st_depth.last! > h:
            let d = st_depth.pop()!
            let start = st_start.pop()!
            let csum = st_sum.pop()!
            let top_d = if st_depth.is_empty: 0 else: st_depth.last!
            let parent_d = if top_d > h: top_d else: h
            let elen = d - parent_d
            let bottom = csum + elen
            out.push(SuffixTreeEdge(csum, bottom, parent_d, elen, start..(i + 1)))
            if not st_depth.is_empty and st_depth.last! == parent_d:
                st_sum[st_sum.len - 1] += bottom
            elif parent_d > 0:
                st_depth.push(parent_d)
                st_start.push(start)
                st_sum.push(bottom)
    out

/// Every run (maximal repetition) of `s`, by the Lyndon-runs theorem: each `(l, r, period)` is a
/// half-open range `s[l..r]` with smallest period `period`, `r - l >= 2 * period`, that cannot be
/// extended left or right with the same period.
///
/// Positions count from 0.
///
/// cost: \(O(n \log n)\) time and memory
/// empty: `[]`
/// order: by `(l, r)`; a new `List`, keyed from 0
/// traps: never
/// example: `lyndon_runs("aabaab") == [(0, 2, 1), (0, 6, 3), (3, 5, 1)]`
/// example: `lyndon_runs("abc") == []`
pub fn lyndon_runs<S: Indexed>(s: S) -> List<(l: Int, r: Int, period: Int)>:
    with lint(unproven_list_index: allow)
    let n = s.len
    if n < 2: return []
    let fwd = [s.nth(i) for i in 0..n]
    let sa = SuffixArray.new(fwd)
    let rev = [s.nth(i) for i in (0..n).rev]
    let rsa = SuffixArray.new(rev)
    var raw = List<(l: Int, r: Int, period: Int)>()
    unchecked:
        for rev_ord in [false, true]:
            var st = List<Int>()
            for i in (0..n).rev:
                while not st.is_empty:
                    let top = st.last!
                    let common = sa.lcp_between(i, top)
                    if top + common == n: break
                    let less = (fwd[i + common] < fwd[top + common]) != rev_ord
                    if less: st.pop() else: break
                let j = if st.is_empty: n else: st.last!
                st.push(i)
                let p = j - i
                let left = if i == 0: 0 else: rsa.lcp_between(n - i, n - j)
                let right = if j == n: 0 else: sa.lcp_between(i, j)
                let l = i - left
                let r = j + right
                if r - l >= 2 * p: raw.push((l:, r:, period: p))
    raw.sort()
    var out = List<(l: Int, r: Int, period: Int)>()
    for run in raw:
        if out.is_empty or out.last!.l != run.l or out.last!.r != run.r:
            out.push(run)
    out

// ---- generalized rolling hash

const RH_M1 = 1_000_000_007
const RH_M2 = 1_000_000_009

/// \(O(1)\) space and comparison. A 64-bit double-mod polynomial hash value (`Copy + Eq + Ord + Hash`).
/// Can itself be fed into another `RollingHash` (e.g. hashing rows into a column hash for 2D grids).
pub struct HashVal(pub h1: Int, pub h2: Int):
    /// One integer that identifies the pair, `(h1 << 31) | h2`, so a `RollingHash` can hash a
    /// sequence of `HashVal` (rows of a grid, say).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `HashVal(1, 2).hash_token == 2147483650`
    @get pub fn hash_token(self) -> Int = (h1 << 31) | h2

/// \(O(1)\) per element. Elements accepted by `RollingHash`: `Char`, `Int`, `Bool`, `HashVal`, or any user type
/// implementing `hash_token`.
pub trait Hashable:
    @get fn hash_token(self) -> Int = Int(self)

@get fn ord(self: Int) -> Int = self

/// \(O(n)\) build, \(O(1)\) slice hash. Generalized double-mod polynomial rolling hash over `Str`, `List<T>`, or any `[D]T`
/// (`Char`, `Int`, `Bool`, `HashVal`).
///
/// Built in \(O(n)\); substring queries `.get(l..r)` / `.slice(l, r)` and `.combine(a, b, b_len)`
/// are \(O(1)\). Each constructor draws independent random salted bases unless `bases:` is passed
/// explicitly, so 2D row and column hashes never collide on transposes.
///
/// not a container: `len` is the hashed string's length; it holds hashes, not elements.
pub struct RollingHash:
    pub bases: (Int, Int)
    p1: List<Int>
    p2: List<Int>
    pow1: List<Int>
    pow2: List<Int>

    /// A double-modulus polynomial rolling hash of `seq`; call it as `RollingHash(seq)`.
    ///
    /// Modulo `M1 = 1_000_000_007` with base `b1` and `M2 = 1_000_000_009` with base `b2`, the
    /// hash of `x_0 .. x_{k-1}` is the sum of `(t(x_i) % M + 1) * b^(k - 1 - i)` where `t` is
    /// `hash_token` (a `Char`'s code, a `Bool`'s `0`/`1`). The bases are drawn at random unless
    /// `bases:` gives them; hashes only compare under the same bases.
    ///
    /// cost: \(O(n)\)
    /// empty: an empty hash; `push` can grow it
    /// traps: when a given base is outside `2..M`
    /// example: `let h = RollingHash("abab"); h.get(0..2) == h.get(2..4)`
    pub fn new<S: Seq>(seq: S, bases: ?(Int, Int) = None) -> RollingHash:
        with lint(unproven_list_index: allow)
        let (b1, b2) = bases ?? (rand(1 << 20, 1 << 29), rand(1 << 20, 1 << 29))
        let n = seq.len
        var p1 = List.filled(n + 1, 0)
        var p2 = List.filled(n + 1, 0)
        var pow1 = List.filled(n + 1, 1)
        var pow2 = List.filled(n + 1, 1)
        var (h1, h2, w1, w2) = (0, 0, 1, 1)
        var idx = 0
        for x in seq:
            idx += 1
            let t = x.hash_token
            let x1 = if 0 <= t and t < 1_000_000_006: t + 1 else: t % 1_000_000_006 + 1
            let x2 = if 0 <= t and t < 1_000_000_008: t + 1 else: t % 1_000_000_008 + 1
            h1 = (h1 * b1 + x1) % 1_000_000_007
            h2 = (h2 * b2 + x2) % 1_000_000_009
            w1 = w1 * b1 % 1_000_000_007
            w2 = w2 * b2 % 1_000_000_009
            unchecked:
                p1[idx] = h1
                p2[idx] = h2
                pow1[idx] = w1
                pow2[idx] = w2
        RollingHash((b1, b2), p1, p2, pow1, pow2)

    /// Appends `x` to the end of the hashed sequence.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: never
    /// example: `var h = RollingHash("ab"); h.push('a'); h.all == h.hash_of("aba")`
    pub fn push<T: Hashable>(mut self, x: T):
        let (b1, b2) = bases
        let n = p1.len - 1
        let t = x.hash_token
        let x1 = if 0 <= t < 1_000_000_006: t + 1 else: t % 1_000_000_006 + 1
        let x2 = if 0 <= t < 1_000_000_008: t + 1 else: t % 1_000_000_008 + 1
        p1.push((p1[n] * b1 + x1) % 1_000_000_007)
        p2.push((p2[n] * b2 + x2) % 1_000_000_009)
        pow1.push(pow1[n] * b1 % 1_000_000_007)
        pow2.push(pow2[n] * b2 % 1_000_000_009)

    /// Removes the last element of the hashed sequence.
    ///
    /// cost: \(O(1)\)
    /// traps: when the hash is empty
    /// example: `var h = RollingHash("abc"); h.pop(); h.all == h.hash_of("ab")`
    pub fn pop(mut self):
        assert p1.len > 1, "`pop` on an empty hash"
        let _ = p1.pop()
        let _ = p2.pop()
        let _ = pow1.pop()
        let _ = pow2.pop()

    /// The hash `seq` would have under `self`'s bases, without building a `RollingHash` of it;
    /// compare it with any slice of `self`.
    ///
    /// cost: \(O(\lvert \mathit{seq} \rvert)\)
    /// empty: `HashVal(0, 0)`
    /// traps: never
    /// example: `let h = RollingHash("xab"); h.hash_of("ab") == h.get(1..3)`
    pub fn hash_of<S: Seq>(self, seq: S) -> HashVal:
        let (b1, b2) = bases
        var (h1, h2) = (0, 0)
        for x in seq:
            let t = x.hash_token
            let x1 = if 0 <= t < 1_000_000_006: t + 1 else: t % 1_000_000_006 + 1
            let x2 = if 0 <= t < 1_000_000_008: t + 1 else: t % 1_000_000_008 + 1
            h1 = (h1 * b1 + x1) % 1_000_000_007
            h2 = (h2 * b2 + x2) % 1_000_000_009
        HashVal(h1, h2)

    /// The hash of the concatenation of `parts`, each given as its hash and length, under
    /// `self`'s bases; `combine` folded left to right.
    ///
    /// cost: \(O(\lvert \mathit{parts} \rvert)\) amortized
    /// empty: `HashVal(0, 0)`
    /// traps: when a `len` is negative
    /// example: `let h = RollingHash("abcd"); h.concat([(h.get(0..2), 2), (h.get(2..4), 2)]) == h.all`
    pub fn concat(self, parts: List<(hash: HashVal, len: Int)>) -> HashVal:
        var acc = HashVal(0, 0)
        for part in parts: acc = self.combine(acc, part.hash, part.len)
        acc

    /// The number of elements hashed.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `RollingHash("abc").len == 3`
    @get pub fn len(self) -> Int = p1.len - 1

    /// True when no elements are hashed.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `RollingHash("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = p1.len == 1

    /// The hash of the whole sequence, `self.get(0..self.len)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let h = RollingHash("abc"); h.all == h.get(0..3)`
    @get pub fn all(self) -> HashVal = self.slice(0, p1.len - 1)

    fn raw_slice(self, l: Int, r: Int) -> HashVal:
        with lint(unproven_list_index: allow)
        let d = r - l
        unchecked:
            return HashVal(
                (p1[r] - p1[l] * pow1[d]) % 1_000_000_007,
                (p2[r] - p2[l] * pow2[d]) % 1_000_000_009,
            )
        HashVal(0, 0)

    fn eq_sub(self, a: Int, b: Int, len: Int) -> Bool:
        with lint(unproven_list_index: allow)
        unchecked:
            if ((p1[a + len] - p1[b + len]) - (p1[a] - p1[b]) * pow1[len]) % 1_000_000_007 != 0:
                return false
            return ((p2[a + len] - p2[b + len]) - (p2[a] - p2[b]) * pow2[len]) % 1_000_000_009 == 0
        false

    /// The hash of the half-open slice `l..r` (0-based); `HashVal(0, 0)` when `l == r`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `l > r`, `l < 0` or `r > self.len`
    /// example: `let h = RollingHash("abab"); h.slice(0, 2) == h.slice(2, 4) and h.slice(1, 1) == HashVal(0, 0)`
    pub fn slice(self, l: Int, r: Int) -> HashVal:
        assert 0 <= l <= r <= p1.len - 1, "`slice(l, r)` needs `0 <= l <= r <= len`"
        if l == r: return HashVal(0, 0)
        self.raw_slice(l, r)

    /// The hash of a range of positions: `l..r`, `l..=r`, `..k`, `k..`, or a `Range<D>` when the
    /// hash was built over a `[D]T`. The same value as `slice`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the range reaches outside the hashed sequence
    /// example: `let h = RollingHash("abab"); h.get(0..2) == h.get(2..=3)`
    pub fn get<R>(self, r: R) -> HashVal:
        if r.is_empty: return HashVal(0, 0)
        self.slice(r.start.ord, r.end.ord)

    /// The hash of `a`'s sequence followed by `b`'s, where `b` has length `b_len`:
    /// `a * base^b_len + b` per modulus.
    ///
    /// cost: \(O(1)\) when `b_len <= len`, else \(O(\log b_len)\)
    /// traps: when `b_len < 0`
    /// example: `let h = RollingHash("abc"); h.combine(h.get(0..1), h.get(1..3), 2) == h.all`
    pub fn combine(self, a: HashVal, b: HashVal, b_len: Int) -> HashVal:
        with lint(unproven_list_index: allow)
        assert b_len >= 0, "RollingHash.combine: negative length"
        let fits = b_len < pow1.len
        let q1 = if fits: pow1[b_len] else: pow_mod(bases.0, b_len, 1_000_000_007)
        let q2 = if fits: pow2[b_len] else: pow_mod(bases.1, b_len, 1_000_000_009)
        HashVal((a.h1 * q1 + b.h1) % 1_000_000_007, (a.h2 * q2 + b.h2) % 1_000_000_009)

    /// The length of the longest common prefix of the suffixes at `a` and `b`, by binary search on
    /// hashes (so it can be wrong on a collision); `of: other` puts `b` in `other`, built with
    /// the same bases.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `a` is outside `0..=self.len`, `b` outside `0..=` the len of `of` (or `self`), or
    /// the bases differ
    /// example: `RollingHash("abcabd").lcp(0, 3) == 2`
    /// example: `RollingHash("abcd", bases: Some((131, 137))).lcp(1, 1, of: RollingHash("xbcx", bases: Some((131, 137)))) == 2`
    pub fn lcp(self, a: Int, b: Int, of: ?RollingHash = None) -> Int:
        if let o = of: return self.lcp_of(a, o, b)
        let n = p1.len - 1
        assert 0 <= a and a <= n and 0 <= b and b <= n, "`lcp` needs both starts in `0..=len`"
        var lo = 0
        var hi = min(n - a, n - b)
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if self.eq_sub(a, b, mid): lo = mid
            else: hi = mid - 1
        lo

    /// The length of the longest common suffix of the prefixes `s[..a]` and `s[..b]`, by binary
    /// search on hashes.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `a` or `b` is outside `0..=self.len`
    /// example: `RollingHash("xabyab").lcs(3, 6) == 2`
    pub fn lcs(self, a: Int, b: Int) -> Int:
        let n = p1.len - 1
        assert 0 <= a and a <= n and 0 <= b and b <= n, "`lcs` needs both ends in `0..=len`"
        var lo = 0
        var hi = min(a, b)
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if self.eq_sub(a - mid, b - mid, mid): lo = mid
            else: hi = mid - 1
        lo

    fn lcp_of(self, a: Int, other: RollingHash, b: Int) -> Int:
        assert 0 <= a and a <= self.len and 0 <= b and b <= other.len, "`lcp(a, b, of:)` needs both starts in range"
        var lo = 0
        var hi = min(self.len - a, other.len - b)
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            let x = self.raw_slice(a, a + mid)
            let y = other.raw_slice(b, b + mid)
            if (x.h1 - y.h1) % 1_000_000_007 == 0 and (x.h2 - y.h2) % 1_000_000_009 == 0: lo = mid
            else: hi = mid - 1
        lo

/// \(O(R \cdot C)\) build, \(O(1)\) rectangle hash. Double-mod 2D polynomial prefix hash over a grid with row domain `R` and column domain `C`.
/// Built in \(O(R \cdot C)\); `rect(rows, cols)` is \(O(1)\) and returns a `HashVal`.
pub struct Hash2D<R, C>(bases: (Int, Int)):
    stride: Int
    pre1: List<Int>
    pre2: List<Int>
    pow_r1: List<Int>
    pow_c1: List<Int>
    pow_r2: List<Int>
    pow_c2: List<Int>

    /// A 2D polynomial prefix hash of the grid whose cell `(r, c)` has the token `code(r, c)`,
    /// for every `r` in `R` and `c` in `C`; call it as `Hash2D<R, C>(code)`.
    ///
    /// Each modulus uses one base along rows and one along columns; they are drawn at random
    /// unless `bases:` gives them. A token enters as `token % M + 1`, as in `RollingHash`.
    ///
    /// cost: \(O(\lvert R \rvert \cdot \lvert C \rvert)\)
    /// empty: an empty `R` or `C` gives a hash whose only rectangle is empty
    /// traps: when a given base is outside `2..M`
    /// example: `domain R = 0..2; domain C = 0..4; let h = Hash2D<R, C>((r, c) => c.ord % 2); h.rect(R(0)..=R(1), C(0)..=C(1)) == h.rect(R(0)..=R(1), C(2)..=C(3))`
    pub fn new(code: (R, C) -> Int, bases: ?(Int, Int) = None) -> Hash2D<R, C>:
        with lint(unproven_list_index: allow)
        let (br1, bc1) = bases ?? (rand(1 << 20, 1 << 29), rand(1 << 20, 1 << 29))
        let br2 = (br1 ^ 324508639) % (RH_M2 - 2) + 2
        let bc2 = (bc1 ^ 715827883) % (RH_M2 - 2) + 2
        let rows = R.count
        let cols = C.count
        let stride = cols + 1
        var pre1: List<0..=1_000_000_009> = List.filled((rows + 1) * stride, 0)
        var pre2: List<0..=1_000_000_009> = List.filled((rows + 1) * stride, 0)
        let brc1 = br1 * bc1 % RH_M1
        let brc2 = br2 * bc2 % RH_M2
        for r in R:
            let i = r.ord
            let row0 = i * stride
            let row1 = row0 + stride
            for c in C:
                let j = c.ord
                let v = code(r, c)
                let v1 = v % (RH_M1 - 1) + 1
                let v2 = v % (RH_M2 - 1) + 1
                unchecked:
                    pre1[row1 + j + 1] = (
                        pre1[row0 + j + 1] * br1 + pre1[row1 + j] * bc1 - pre1[row0 + j] * brc1 + v1
                    ) % RH_M1
                    pre2[row1 + j + 1] = (
                        pre2[row0 + j + 1] * br2 + pre2[row1 + j] * bc2 - pre2[row0 + j] * brc2 + v2
                    ) % RH_M2
        var pow_r1: List<0..=1_000_000_009> = List.filled(rows + 1, 1)
        var pow_r2: List<0..=1_000_000_009> = List.filled(rows + 1, 1)
        unchecked:
            for i in 0..rows:
                pow_r1[i + 1] = pow_r1[i] * br1 % RH_M1
                pow_r2[i + 1] = pow_r2[i] * br2 % RH_M2
        var pow_c1: List<0..=1_000_000_009> = List.filled(cols + 1, 1)
        var pow_c2: List<0..=1_000_000_009> = List.filled(cols + 1, 1)
        unchecked:
            for j in 0..cols:
                pow_c1[j + 1] = pow_c1[j] * bc1 % RH_M1
                pow_c2[j + 1] = pow_c2[j] * bc2 % RH_M2
        Hash2D((br1, bc1), stride, pre1, pre2, pow_r1, pow_c1, pow_r2, pow_c2)

    /// The hash of the rectangle `rows` x `cols`; equal rectangles of equal shape hash equal.
    /// A rectangle with no rows or no columns hashes to `HashVal(0, 0)`.
    ///
    /// cost: \(O(1)\)
    /// empty: an empty `rows` or `cols` gives `HashVal(0, 0)`
    /// traps: never (the ranges are typed by `R` and `C`)
    /// example: `domain R = 0..2; domain C = 0..4; let h = Hash2D<R, C>((r, c) => r.ord); h.rect(R(0)..=R(0), C(0)..=C(1)) != h.rect(R(1)..=R(1), C(0)..=C(1))`
    pub fn rect(self, rows: Range<R>, cols: Range<C>) -> HashVal:
        with lint(unproven_list_index: allow)
        if rows.is_empty or cols.is_empty: return HashVal(0, 0)
        let (r1, r2, c1, c2) = (rows.l.ord, rows.r.ord, cols.l.ord, cols.r.ord)
        assert (0 <= r1 <= r2 < pow_r1.len)
        assert (0 <= c1 <= c2 < stride)
        let o1 = r1 * stride
        let o2 = r2 * stride
        unchecked:
            let hr1 = pow_r1[r2 - r1]
            let hc1 = pow_c1[c2 - c1]
            let h1 = (
                pre1[o2 + c2] - pre1[o1 + c2] * hr1 - pre1[o2 + c1] * hc1 + pre1[o1 + c1] * hr1 % RH_M1 * hc1
            ) % RH_M1
            let hr2 = pow_r2[r2 - r1]
            let hc2 = pow_c2[c2 - c1]
            let h2 = (
                pre2[o2 + c2] - pre2[o1 + c2] * hr2 - pre2[o2 + c1] * hc2 + pre2[o1 + c1] * hr2 % RH_M2 * hc2
            ) % RH_M2
            return HashVal(h1, h2)

const RH61_MOD: U64 = U64(2305843009213693951)

fn rh61_add(a: U64, b: U64) -> U64:
    let s = a + b
    if s >= RH61_MOD: s - RH61_MOD else: s

fn rh61_sub(a: U64, b: U64) -> U64:
    if a >= b: a - b else: a + RH61_MOD - b

fn rh61_mul(a: U64, b: U64) -> U64:
    let p = I128(a) * I128(b)
    let lo = U64(p & I128(RH61_MOD))
    let hi = U64(p >> 61)
    rh61_add(lo, hi)

fn rh61_tok(t: Int) -> U64:
    let m1: Int = 2305843009213693950
    let r = t % m1
    U64(r + 1)

/// \(O(1)\) value. Single-modulus `2^61 - 1` (Mersenne prime) polynomial hash value (`Copy + Eq + Ord + Hash`).
pub struct Hash61(v: U64)

/// \(O(n)\) build, \(O(1)\) query. Single-modulus `2^61 - 1` polynomial rolling hash with randomized base.
///
/// not a container: `len` is the hashed string's length; it holds hashes, not elements.
pub struct RollingHash61:
    base: U64
    pre: List<U64>
    pow: List<U64>

    /// A single-modulus polynomial rolling hash of `seq` modulo the Mersenne prime `2^61 - 1`; call
    /// it as `RollingHash61(seq)`.
    ///
    /// Same scheme as `RollingHash` with one modulus: each token enters as `token % M + 1`; the
    /// base is drawn at random unless `base:` gives it.
    ///
    /// cost: \(O(n)\)
    /// empty: an empty hash; `push` can grow it
    /// traps: when a given base is outside `2..2^61 - 1`
    /// example: `let h = RollingHash61("abab"); h.get(0..2) == h.get(2..4)`
    pub fn new<S: Seq>(seq: S, base: ?U64 = None) -> RollingHash61:
        with lint(unproven_list_index: allow)
        let b = base ?? U64(rand(1 << 20, 1 << 29))
        assert U64(2) <= b < RH61_MOD, "RollingHash61: base must be in 2..2^61 - 1"
        var pre: List<U64> = [U64(0)]
        var pow: List<U64> = [U64(1)]
        var h = U64(0)
        var w = U64(1)
        for x in seq:
            let tv = rh61_tok(x.hash_token)
            h = rh61_add(rh61_mul(h, b), tv)
            w = rh61_mul(w, b)
            pre.push(h)
            pow.push(w)
        RollingHash61(b, pre, pow)

    /// The number of elements hashed.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `RollingHash61("abc").len == 3`
    @get pub fn len(self) -> Int = pre.len - 1

    /// True when no elements are hashed.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `RollingHash61("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = pre.len == 1

    fn slice61(self, l: Int, r: Int) -> Hash61:
        with lint(unproven_list_index: allow)
        assert 0 <= l <= r <= pre.len - 1, "RollingHash61: range out of bounds"
        if l == r: return Hash61(U64(0))
        Hash61(rh61_sub(pre[r], rh61_mul(pre[l], pow[r - l])))

    /// The hash of the whole sequence, `self.get(0..self.len)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let h = RollingHash61("abc"); h.all == h.get(0..3)`
    @get pub fn all(self) -> Hash61 = self.slice61(0, pre.len - 1)

    /// The hash of a range of positions (`l..r`, `l..=r`, `..k`, `k..`, or a `Range<D>` when
    /// built over a `[D]T`); an empty range hashes to `Hash61(0)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the range reaches outside the hashed sequence
    /// example: `let h = RollingHash61("abab"); h.get(0..2) == h.get(2..=3)`
    pub fn get<R>(self, r: R) -> Hash61:
        if r.is_empty: return Hash61(U64(0))
        self.slice61(r.start.ord, r.end.ord)

    /// The hash of `a`'s sequence followed by `b`'s, where `b` has length `b_len`:
    /// `a * base^b_len + b` modulo `2^61 - 1`.
    ///
    /// cost: \(O(1)\) amortized (the power table grows past `len` on demand)
    /// traps: when `b_len < 0`
    /// example: `let h = RollingHash61("abc"); h.combine(h.get(0..1), h.get(1..3), 2) == h.all`
    pub fn combine(self, a: Hash61, b: Hash61, b_len: Int) -> Hash61:
        with lint(unproven_list_index: allow)
        assert b_len >= 0, "RollingHash61.combine: negative length"
        var p = if b_len < pow.len: pow[b_len] else: U64(1)
        if b_len >= pow.len:
            var e = b_len
            var cur = base
            while e > 0:
                if e % 2 == 1: p = rh61_mul(p, cur)
                cur = rh61_mul(cur, cur)
                e /= 2
        Hash61(rh61_add(rh61_mul(a.v, p), b.v))

    /// Appends `x` to the end of the hashed sequence.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: never
    /// example: `var h = RollingHash61("ab"); h.push('a'); h.push('b'); h.get(0..2) == h.get(2..4)`
    pub fn push<T: Hashable>(mut self, x: T):
        with lint(unproven_list_index: allow)
        let tv = rh61_tok(x.hash_token)
        let n = pre.len - 1
        pre.push(rh61_add(rh61_mul(pre[n], base), tv))
        if pow.len <= n + 1: pow.push(rh61_mul(pow[n], base))

    /// Removes the last element of the hashed sequence.
    ///
    /// cost: \(O(1)\)
    /// traps: when the hash is empty
    /// example: `var h = RollingHash61("abab"); h.pop(); h.len == 3`
    pub fn pop(mut self):
        assert pre.len > 1, "RollingHash61.pop: empty hash"
        let _ = pre.pop()

    /// The length of the longest common prefix of the suffixes at `a` and `b`, by binary search on
    /// hashes; `of: other` puts `b` in `other`, built with the same bases.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `a` is outside `0..=self.len`, `b` outside `0..=` the len of `of` (or `self`), or
    /// the bases differ
    /// example: `RollingHash61("abcabd").lcp(0, 3) == 2`
    /// example: `RollingHash61("abcd", base: Some(U64(131))).lcp(1, 1, of: RollingHash61("xbcx", base: Some(U64(131)))) == 2`
    pub fn lcp(self, a: Int, b: Int, of: ?RollingHash61 = None) -> Int:
        if let o = of: return self.lcp_of(a, o, b)
        let n = pre.len - 1
        assert 0 <= a and a <= n and 0 <= b and b <= n, "RollingHash61.lcp: index out of range"
        var lo = 0
        var hi = n - max(a, b)
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if self.slice61(a, a + mid) == self.slice61(b, b + mid): lo = mid else: hi = mid - 1
        lo

    /// The length of the longest common suffix of the prefixes `s[..a]` and `s[..b]`, by binary
    /// search on hashes.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `a` or `b` is outside `0..=self.len`
    /// example: `RollingHash61("xabyab").lcs(3, 6) == 2`
    pub fn lcs(self, a: Int, b: Int) -> Int:
        let n = pre.len - 1
        assert 0 <= a and a <= n and 0 <= b and b <= n, "RollingHash61.lcs: index out of range"
        var lo = 0
        var hi = min(a, b)
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if self.slice61(a - mid, a) == self.slice61(b - mid, b): lo = mid else: hi = mid - 1
        lo

    fn lcp_of(self, a: Int, other: RollingHash61, b: Int) -> Int:
        assert base == other.base, "RollingHash61.lcp: hashes must use the same base"
        let na = pre.len - 1
        let nb = other.pre.len - 1
        assert 0 <= a and a <= na and 0 <= b and b <= nb, "RollingHash61.lcp: index out of range"
        var lo = 0
        var hi = min(na - a, nb - b)
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if self.slice61(a, a + mid) == other.slice61(b, b + mid): lo = mid else: hi = mid - 1
        lo

/// \(O(n)\) build, \(O(\log n)\) point update and range hash. Mutable double-mod rolling hash over domain `D`.
/// Supports `h[i] = x` via W8 `IndexSet`.
///
/// not a container: `len` is the hashed string's length; it holds hashes, not elements.
pub struct DynamicHash<D>:
    bases: (Int, Int)
    tok1: List<Int>
    tok2: List<Int>
    fwd1: List<Int>
    fwd2: List<Int>
    bwd1: List<Int>
    bwd2: List<Int>
    pow1: List<Int>
    pow2: List<Int>
    inv1: List<Int>
    inv2: List<Int>

    /// A double-modulus rolling hash of `seq` that supports point updates, kept forwards and
    /// backwards in a segment tree; call it as `DynamicHash(seq)`. Positions are `seq`'s keys
    /// (domain `D`); hashes follow the `RollingHash` scheme and bases.
    ///
    /// cost: \(O(n)\)
    /// empty: an empty hash
    /// traps: when a given base is outside `2..M`
    /// example: `let h = DynamicHash("abab"); h.get(0..2) == h.get(2..4)`
    pub fn new<S: Indexed>(seq: S, bases: ?(Int, Int) = None) -> DynamicHash<D>:
        with lint(unproven_list_index: allow)
        let (b1, b2) = bases ?? (rand(1 << 20, 1 << 29), rand(1 << 20, 1 << 29))
        assert 2 <= b1 and b1 < RH_M1 and 2 <= b2 and b2 < RH_M2, "`DynamicHash` bases must be in `2..M`"
        let n = seq.len
        var pow1 = List.filled(n + 1, 1)
        var pow2 = List.filled(n + 1, 1)
        var inv1 = List.filled(n + 1, 1)
        var inv2 = List.filled(n + 1, 1)
        let ib1 = dh_pow(b1, RH_M1 - 2, RH_M1)
        let ib2 = dh_pow(b2, RH_M2 - 2, RH_M2)
        for i in 1..=n:
            pow1[i] = pow1[i - 1] * b1 % RH_M1
            pow2[i] = pow2[i - 1] * b2 % RH_M2
            inv1[i] = inv1[i - 1] * ib1 % RH_M1
            inv2[i] = inv2[i - 1] * ib2 % RH_M2
        var h = DynamicHash<D>(
            (b1, b2),
            List.filled(n, 0),
            List.filled(n, 0),
            List.filled(n + 1, 0),
            List.filled(n + 1, 0),
            List.filled(n + 1, 0),
            List.filled(n + 1, 0),
            pow1,
            pow2,
            inv1,
            inv2,
        )
        var i = 0
        for x in seq:
            h.put(i, x.hash_token)
            i += 1
        h

    /// The number of hashed positions, `D.count`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `DynamicHash("abc").len == 3`
    @get pub fn len(self) -> Int = tok1.len

    /// True when there are no positions.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `DynamicHash("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = tok1.is_empty

    /// The hash of the whole sequence, `self.get(..)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let h = DynamicHash("abab"); h.all == h.get(0..4)`
    @get pub fn all(self) -> HashVal = self.span(0, tok1.len)

    /// Replaces the element at position `i` with `x`; `h[i] = x` means the same.
    ///
    /// cost: \(O(\log n)\)
    /// traps: never (`i` is a `D`)
    /// example: `var h = DynamicHash("abxb"); h.set(2, 'a'); h.get(0..2) == h.get(2..4)`
    pub fn set<T: Hashable>(mut self, i: D, x: T) = self.put(i.ord, x.hash_token)

    /// The hash of the elements over `r` in order (a cut range `j..i` works too, decision 124).
    ///
    /// cost: \(O(\log n)\)
    /// empty: an empty `r` gives `HashVal(0, 0)`
    /// traps: never (`r` is a `Range<D>`)
    /// example: `let h = DynamicHash("xyxy"); h.get(0..2) == h.get(2..4)`
    pub fn get(self, r: Range<D>) -> HashVal:
        if r.is_empty: return HashVal(0, 0)
        self.span(r.l.ord, r.r.ord)

    /// The hash of the elements over `r` read backwards, comparable with `get`.
    ///
    /// cost: \(O(\log n)\)
    /// empty: an empty `r` gives `HashVal(0, 0)`
    /// traps: never (`r` is a `Range<D>`)
    /// example: `let h = DynamicHash("abba"); h.rev_get(0..2) == h.get(2..4)`
    pub fn rev_get(self, r: Range<D>) -> HashVal:
        with lint(unproven_list_index: allow)
        if r.is_empty: return HashVal(0, 0)
        let (l, e) = (r.l.ord, r.r.ord)
        let s1 = (dh_sum(fwd1, e, RH_M1) - dh_sum(fwd1, l, RH_M1)) % RH_M1
        let s2 = (dh_sum(fwd2, e, RH_M2) - dh_sum(fwd2, l, RH_M2)) % RH_M2
        HashVal(s1 * inv1[l] % RH_M1, s2 * inv2[l] % RH_M2)

    /// True when the elements over `r` read the same both ways: `get(r) == rev_get(r)`.
    ///
    /// cost: \(O(\log n)\)
    /// empty: `true` for an empty `r`
    /// traps: never (`r` is a `Range<D>`)
    /// example: `let h = DynamicHash("abca"); h.is_palindrome(1..2) and not h.is_palindrome(0..4)`
    pub fn is_palindrome(self, r: Range<D>) -> Bool = self.get(r) == self.rev_get(r)

    fn span(self, l: Int, e: Int) -> HashVal:
        with lint(unproven_list_index: allow)
        let n = tok1.len
        let s1 = (dh_sum(bwd1, e, RH_M1) - dh_sum(bwd1, l, RH_M1)) % RH_M1
        let s2 = (dh_sum(bwd2, e, RH_M2) - dh_sum(bwd2, l, RH_M2)) % RH_M2
        HashVal(s1 * inv1[n - e] % RH_M1, s2 * inv2[n - e] % RH_M2)

    fn put(mut self, i: Int, t: Int):
        with lint(unproven_list_index: allow)
        let n = tok1.len
        let x1 = if 0 <= t < RH_M1 - 1: t + 1 else: t % (RH_M1 - 1) + 1
        let x2 = if 0 <= t < RH_M2 - 1: t + 1 else: t % (RH_M2 - 1) + 1
        let d1 = (x1 - tok1[i]) % RH_M1
        let d2 = (x2 - tok2[i]) % RH_M2
        tok1[i] = x1
        tok2[i] = x2
        dh_add(mut fwd1, i, d1 * pow1[i] % RH_M1, RH_M1)
        dh_add(mut fwd2, i, d2 * pow2[i] % RH_M2, RH_M2)
        dh_add(mut bwd1, i, d1 * pow1[n - 1 - i] % RH_M1, RH_M1)
        dh_add(mut bwd2, i, d2 * pow2[n - 1 - i] % RH_M2, RH_M2)

fn dh_pow(b: Int, e: Int, m: Int) -> Int:
    var (r, x, k) = (1, b % m, e)
    while k > 0:
        if k % 2 == 1: r = r * x % m
        x = x * x % m
        k /= 2
    r

fn dh_add(mut tree: List<Int>, i: Int, d: Int, m: Int):
    with lint(unproven_list_index: allow)
    var j = i + 1
    while j < tree.len:
        tree[j] = (tree[j] + d) % m
        j += j & -j

fn dh_sum(tree: List<Int>, e: Int, m: Int) -> Int:
    with lint(unproven_list_index: allow)
    var (s, j) = (0, e)
    while j > 0:
        s = (s + tree[j]) % m
        j -= j & -j
    s

// ---- alphabet-domain automata

/// \(O(n \cdot \lvert A \rvert)\) space, \(O(1)\) amortized push. Online suffix automaton over alphabet domain `A`.
/// \(O(n \cdot \lvert A \rvert)\) flat memory, \(O(n)\) amortized build, \(O(\mathit{states})\) `topo_order` and `occurrences`.
///
/// not a container: an automaton; `len` counts states, which are walked by `step`.
pub struct SuffixAutomaton<A>:
    alpha: 0..=2_000_000_000
    next: List<-1..=2_000_000_000>
    links: List<-1..=2_000_000_000>
    max_len: List<0..=2_000_000_000>
    first_end: List<-1..=2_000_000_000>
    prefix: List<Bool>
    pub last: 0..=2_000_000_000
    letters: List<?A>

    /// An empty suffix automaton over the alphabet domain `A`: only the root state `0`. Call it
    /// as `SuffixAutomaton<A>()`, then `push` letters.
    ///
    /// cost: \(O(\lvert A \rvert)\)
    /// traps: never
    /// example: `SuffixAutomaton<Char>().len == 1`
    pub fn new() -> SuffixAutomaton<A>:
        let k = A.count
        let lts: List<?A> = [None for _ in 0..k]
        SuffixAutomaton(k, List.filled(k, -1), [-1], [0], [-1], [false], 0, lts)

    /// The suffix automaton of `s`: `SuffixAutomaton<A>()` followed by `push` of each element.
    ///
    /// States are numbered in creation order: each `push` creates its new state, then the
    /// clone if one is needed. For `"abb"` the states are `0` (empty), `1` (`a`), `2` (`ab`),
    /// `3` (`abb`, `bb`) and `4` (`b`, the clone).
    ///
    /// cost: \(O(\lvert s \rvert \cdot \lvert A \rvert)\) memory, \(O(\lvert s \rvert)\) amortized time
    /// empty: only the root
    /// traps: when an element of `s` is not in `A`
    /// example: `SuffixAutomaton<Char>.of("abb").len == 5`
    pub fn of<S: Seq>(s: S) -> SuffixAutomaton<A>:
        var sam = SuffixAutomaton<A>.new()
        for a in s: sam.push(a)
        sam

    fn add_empty_state(mut self, length: Int, is_prefix: Bool) -> Int:
        with lint(unproven_list_index: allow)
        let id = links.len
        for _ in 0..alpha: next.push(-1)
        links.push(-1)
        max_len.push(length)
        first_end.push(length - 1)
        prefix.push(is_prefix)
        id

    fn clone_state(mut self, src: Int, length: Int) -> Int:
        with lint(unproven_list_index: allow)
        let id = links.len
        let base = src * alpha
        for j in 0..alpha: next.push(next[base + j])
        links.push(links[src])
        max_len.push(length)
        first_end.push(first_end[src])
        prefix.push(false)
        id

    /// Appends the letter `a` to the string the automaton recognises the substrings of.
    ///
    /// cost: \(O(1)\) amortized (\(O(\lvert A \rvert)\) for a cloned state)
    /// traps: never (`a` is an `A`)
    /// example: `var m = SuffixAutomaton<Char>(); m.push('a'); m.push('b'); m.distinct() == 3`
    pub fn push(mut self, a: A):
        with lint(unproven_list_index: allow)
        let c = a.ord
        letters[c] = Some(a)
        let cur = self.add_empty_state(max_len[last] + 1, true)
        unchecked:
            var p = last
            while p != -1 and next[p * alpha + c] == -1:
                next[p * alpha + c] = cur
                p = links[p]
            if p == -1:
                links[cur] = 0
            else:
                let q = next[p * alpha + c]
                if max_len[q] == max_len[p] + 1:
                    links[cur] = q
                else:
                    let clone = self.clone_state(q, max_len[p] + 1)
                    links[q] = clone
                    links[cur] = clone
                    while p != -1 and next[p * alpha + c] == q:
                        next[p * alpha + c] = clone
                        p = links[p]
            last = cur

    /// The root state, for the empty string: always `0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("ab").root == 0`
    @get pub fn root(self) -> Int = 0

    /// The number of states, the root included; at most `max(1, 2n - 1)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").len == 5`
    @get pub fn len(self) -> Int = links.len

    /// The state reached from `state` by the letter `a`, or `None` when that extension is not a
    /// substring.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.step(0, 'b') == Some(4) and m.step(1, 'a') == None`
    pub fn step(self, state: Int, a: A) -> ?Int:
        with lint(unproven_list_index: allow)
        let t = next[state * alpha + a.ord]
        if t == -1: None else: t

    /// The suffix link of `state`: the state of its longest suffix that lies in another state;
    /// `None` at the root.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.link(3) == Some(4) and m.link(0) == None`
    pub fn link(self, state: Int) -> ?Int:
        with lint(unproven_list_index: allow)
        let p = links[state]
        if p == -1: None else: p

    /// The length of the longest substring in `state`; `0` at the root.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `SuffixAutomaton<Char>.of("abb").longest(3) == 3`
    pub fn longest(self, state: Int) -> Int:
        with lint(unproven_list_index: allow)
        return max_len[state]

    /// The length of the shortest substring in `state`, `longest(link) + 1`; `0` at the root.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `SuffixAutomaton<Char>.of("abb").shortest(3) == 2`
    pub fn shortest(self, state: Int) -> Int:
        with lint(unproven_list_index: allow)
        return if links[state] == -1: 0 else: max_len[links[state]] + 1

    /// The 0-based position where the first occurrence of `state`'s substrings ends; `None` at the
    /// root, whose only substring is empty. A clone keeps the position of the state it was cloned
    /// from.
    ///
    /// cost: \(O(1)\)
    /// ties: the earliest occurrence; later ones are not tracked
    /// traps: when `state` is outside `0..self.len`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.first_pos(4) == Some(1) and m.first_pos(0) == None`
    pub fn first_pos(self, state: Int) -> ?Int:
        with lint(unproven_list_index: allow)
        let e = first_end[state]
        return if e < 0: None else: Some(e)

    /// The state holding `word`, or `None` when `word` is not a substring; the root for an empty
    /// `word`.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\)
    /// empty: an empty `word` gives `Some(0)`
    /// ties: None; a substring lies in exactly one state
    /// traps: when an element of `word` is not in `A`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.find("bb") == Some(3) and m.find("ba") == None`
    pub fn find<S: Seq>(self, word: S) -> ?Int:
        with lint(unproven_list_index: allow)
        var at = 0
        for a in word:
            let t = next[at * alpha + a.ord]
            if t == -1: return None
            at = t
        at

    /// True when `word` is a substring of the automaton's string: `find(word) != None`.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\)
    /// empty: `true` for an empty `word`
    /// traps: when an element of `word` is not in `A`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.contains("ab") and not m.contains("ba")`
    pub fn contains<S: Seq>(self, word: S) -> Bool = self.find(word) != None

    /// The number of distinct non-empty substrings: the sum of `longest(v) - shortest(v) + 1`
    /// over the states other than the root.
    ///
    /// cost: \(O(\mathit{states})\)
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").distinct() == 5`
    pub fn distinct(self) -> Int:
        var total = 0
        unchecked:
            for v in 1..links.len: total += max_len[v] - max_len[links[v]]
        total

    /// Every state, by non-decreasing `longest` (counting sort), so each state comes after its
    /// suffix link; iterate it backwards to push values up the links.
    ///
    /// cost: \(O(\mathit{states})\)
    /// empty: `[0]` for an empty automaton
    /// ties: smaller state number first
    /// order: non-decreasing `longest`; a new `List`, keyed from 0
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").topo_order() == [0, 1, 4, 2, 3]`
    pub fn topo_order(self) -> List<Int>:
        with lint(unproven_list_index: allow)
        let m = links.len
        let top = [max_len[v] for v in 0..m].max() ?? 0
        var cnt = List.filled(top + 1, 0)
        for v in 0..m: cnt[max_len[v]] += 1
        for i in 1..=top: cnt[i] += cnt[i - 1]
        var order = List.filled(m, 0)
        for v in (0..m).rev:
            let l = max_len[v]
            cnt[l] -= 1
            order[cnt[l]] = v
        order

    /// For each state, the number of end positions of its substrings in the automaton's
    /// string; the root counts every position, `n`.
    ///
    /// cost: \(O(\mathit{states})\)
    /// empty: `[0]` for an empty automaton
    /// order: keyed by state
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").occ_counts() == [3, 1, 1, 1, 2]`
    pub fn occ_counts(self) -> List<Int>:
        with lint(unproven_list_index: allow)
        let order = self.topo_order()
        var occ = [if prefix[v]: 1 else: 0 for v in 0..links.len]
        unchecked:
            for i in (1..order.len).rev:
                let v = order[i]
                occ[links[v]] += occ[v]
        occ

    /// A longest common substring of the automaton's string and `other`, as `(len, end)` with
    /// `other[end - len..end]` the substring (positions in `other`, from 0).
    ///
    /// cost: \(O(\lvert \mathit{other} \rvert)\)
    /// empty: `(0, 0)` for an empty `other`; also when nothing is shared
    /// ties: the smallest `end`
    /// traps: when an element of `other` is not in `A`
    /// example: `SuffixAutomaton<Char>.of("abb").lcs("xbbx") == (2, 3)`
    pub fn lcs<S: Seq>(self, other: S) -> (len: Int, end: Int):
        with lint(unproven_list_index: allow)
        var (v, l, best_len, best_end, i) = (0, 0, 0, 0, 0)
        for a in other:
            let c = a.ord
            while v != 0 and next[v * alpha + c] == -1:
                v = links[v]
                l = max_len[v]
            let nxt = next[v * alpha + c]
            if nxt != -1:
                v = nxt
                l += 1
            if l > best_len:
                best_len = l
                best_end = i + 1
            i += 1
        (len: best_len, end: best_end)

    /// Moves the append point back to the root, so the next `push` starts a new word of a
    /// generalized automaton (the substrings of all words added so far).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m = SuffixAutomaton<Char>.of("ab"); m.reset(); m.push('c'); m.contains("c") and not m.contains("bc")`
    pub fn reset(mut self):
        last = 0

    /// Adds `word` as a separate word of a generalized automaton: `reset()`, then `push` each
    /// letter. Afterwards the automaton holds the substrings of every word added.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\) amortized
    /// empty: an empty `word` only resets
    /// traps: when an element of `word` is not in `A`
    /// example: `var m = SuffixAutomaton<Char>(); m.add_word("ab"); m.add_word("b"); m.distinct() == 3`
    pub fn add_word<S: Seq>(mut self, word: S):
        last = 0
        for a in word: self.push(a)

    /// Every state in the order of `topo_order`, lazily.
    ///
    /// cost: \(O(\mathit{states})\)
    /// order: non-decreasing `longest`; ties by smaller state number
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").states().collect() == [0, 1, 4, 2, 3]`
    pub fn states(self) -> yields Int:
        for v in self.topo_order(): yield v

    /// The suffix-link tree as child lists: entry `v` lists the states whose link is `v`.
    /// Rooted at `0`.
    ///
    /// cost: \(O(\mathit{states})\)
    /// order: keyed by state; each child list in increasing state number
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").endpos_tree() == [[1, 4], [], [], [], [2, 3]]`
    pub fn endpos_tree(self) -> List<List<Int>>:
        var kids = [List<Int>() for _ in 0..links.len]
        unchecked:
            for v in 1..links.len: kids[links[v]].push(v)
        kids

    /// The `k`-th (0-based) non-empty substring in lexicographic order, as its letters, or
    /// `None` when there are not that many. With `distinct: false` each substring counts once
    /// per occurrence. A proper prefix comes before its extensions.
    ///
    /// cost: \(O(\mathit{states} \cdot \lvert A \rvert)\)
    /// ties: with `distinct: false`, copies of one substring are adjacent and interchangeable
    /// order: the letters of the substring, first to last
    /// traps: when `k < 0`
    /// example: `SuffixAutomaton<Char>.of("abb").kth_substring(3) == Some(['b'])`
    /// example: `SuffixAutomaton<Char>.of("abb").kth_substring(5, distinct: false) == Some(['b', 'b'])`
    pub fn kth_substring(self, k: Int, distinct: Bool = true) -> ?List<A>:
        with lint(unproven_list_index: allow)
        assert k >= 0, "kth_substring requires k >= 0"
        let m = links.len
        let order = self.topo_order()
        var weight = List.filled(m, 0)
        if distinct:
            for v in 1..m: weight[v] = 1
        else:
            let occ = self.occ_counts()
            for v in 1..m: weight[v] = occ[v]
        var dp = List.filled(m, 0)
        unchecked:
            for i in (0..m).rev:
                let v = order[i]
                var s = weight[v]
                let base = v * alpha
                for c in 0..alpha:
                    let u = next[base + c]
                    if u != -1: s += dp[u]
                dp[v] = s
        if k >= dp[0]: return None
        var rem = k
        var v = 0
        var out: List<A> = []
        while true:
            if v != 0:
                if rem < weight[v]: return Some(out)
                rem -= weight[v]
            let base = v * alpha
            var moved = false
            for c in 0..alpha:
                let u = next[base + c]
                if u != -1:
                    if rem < dp[u]:
                        out.push(letters[c]!)
                        v = u
                        moved = true
                        break
                    rem -= dp[u]
            if not moved: return Some(out)

/// The length of the longest sequence that is a substring of every word in `words`.
///
/// cost: \(O(\mathit{sum} \lvert w \rvert \cdot \lvert A \rvert)\)
/// empty: `0` for no words; one word gives its length
/// traps: never
/// example: `lcs_all(["abcd", "xbcy", "bcz"]) == 2`
pub fn lcs_all<A, S: Seq>(words: List<S>) -> Int:
    with lint(unproven_list_index: allow)
    if words.is_empty: return 0
    let b1 = rand(1 << 20, 1 << 29)
    let b2 = rand(1 << 20, 1 << 29)
    var hashes: List<RollingHash> = []
    var hi = -1
    for w in words:
        let h = RollingHash(w, bases: Some((b1, b2)))
        if hi == -1 or h.len < hi: hi = h.len
        hashes.push(h)
    if hashes.len == 1: return hi
    if hi == 0: return 0
    var lo = 0
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        var cur: Set<HashVal> = Set()
        let h0 = hashes[0]
        for i in 0..=h0.len - mid: cur.insert(h0.slice(i, i + mid))
        for k in 1..hashes.len:
            let hk = hashes[k]
            var next: Set<HashVal> = Set()
            for i in 0..=hk.len - mid:
                let hv = hk.slice(i, i + mid)
                if hv in cur: next.insert(hv)
            cur = next
            if cur.is_empty: break
        if not cur.is_empty: lo = mid else: hi = mid - 1
    lo

/// \(O(n \cdot \lvert A \rvert)\) space, \(O(1)\) amortized push. Online palindromic tree (eertree) over alphabet domain `A`, with `diff` and `series_link`
/// for \(O(n \log n)\) palindromic partition DP.
/// State `0` is the odd root (`pal_len == -1`); state `1` is the even root (`pal_len == 0`).
///
/// not a container: an automaton; `len` counts states, which are walked by `step`.
pub struct PalindromicTree<A>:
    alpha: 0..=2_000_000_000
    next: List<-1..=2_000_000_000>
    slink: List<0..=2_000_000_000>
    plen: List<-1..=2_000_000_000>
    diff_val: List<0..=2_000_000_000>
    ser_link: List<0..=2_000_000_000>
    cnt: List<0..=2_000_000_000>
    chars: List<A>
    last: 0..=2_000_000_000
    front: List<A>
    first: 0..=2_000_000_000
    undo: List<0..=2_000_000_000>
    made: List<0..=2_000_000_000>
    firsts: List<0..=2_000_000_000>
    barrier: Int

    /// An empty palindromic tree over the alphabet domain `A`: just the odd root `0` and the
    /// even root `1`. Call it as `PalindromicTree<A>()`.
    ///
    /// cost: \(O(\lvert A \rvert)\)
    /// traps: never
    /// example: `PalindromicTree<Char>().len == 2`
    pub fn new() -> PalindromicTree<A>:
        let k = A.count
        PalindromicTree(
            k,
            List.filled(2 * k, -1),
            [0, 0],
            [-1, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [],
            1,
            [],
            1,
            [],
            [],
            [],
            0,
        )

    /// The palindromic tree of `s`: `PalindromicTree<A>()` followed by `push` of each element.
    /// New states are numbered from `2` in creation order.
    ///
    /// cost: \(O(\lvert s \rvert \cdot \lvert A \rvert)\) memory, \(O(\lvert s \rvert)\) amortized time
    /// empty: just the two roots
    /// traps: when an element of `s` is not in `A`
    /// example: `PalindromicTree<Char>.of("aba").distinct == 3`
    pub fn of<S: Seq>(s: S) -> PalindromicTree<A>:
        var pt = PalindromicTree<A>.new()
        for a in s: let _ = pt.push(a)
        pt

    /// Appends the letter `a` and returns the state of the longest palindromic suffix of the
    /// new string, creating it if it is new.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: never (`a` is an `A`)
    /// example: `var t = PalindromicTree<Char>(); t.push('a'); t.push('a') == 3`
    pub fn push(mut self, a: A) -> Int:
        with lint(unproven_list_index: allow)
        let c = a.ord
        let pos = self.size()
        chars.push(a)
        unchecked:
            var cur = last
            while true:
                let prev = pos - plen[cur] - 1
                if prev >= 0 and self.at(prev) == a: break
                cur = slink[cur]
            if next[cur * alpha + c] == -1:
                made.push(cur * alpha + c)
                let link_to = if cur == 0:
                    1
                else:
                    var f = slink[cur]
                    while true:
                        let prev = pos - plen[f] - 1
                        if prev >= 0 and self.at(prev) == a: break
                        f = slink[f]
                    next[f * alpha + c]
                self.grow(cur, c, link_to)
            undo.push(last)
            last = next[cur * alpha + c]
            cnt[last] += 1
            if plen[last] == pos + 1:
                firsts.push(first)
                first = last
        last

    fn size(self) -> Int = front.len + chars.len

    /// The character at position `i` of the current string, counting from the front.
    ///
    /// cost: \(O(1)\)
    pub fn at(self, i: Int) -> A:
        with lint(unproven_list_index: allow)
        return if i < front.len: front[front.len - 1 - i] else: chars[i - front.len]

    fn grow(mut self, cur: Int, c: Int, link_to: Int):
        with lint(unproven_list_index: allow)
        unchecked:
            let nw = slink.len
            let nw_len = plen[cur] + 2
            let d = nw_len - plen[link_to]
            let sl = if d == diff_val[link_to]: ser_link[link_to] else: link_to
            for _ in 0..alpha: next.push(-1)
            slink.push(link_to)
            plen.push(nw_len)
            diff_val.push(d)
            ser_link.push(sl)
            cnt.push(0)
            next[cur * alpha + c] = nw

    /// The number of states, both roots included: `distinct + 2`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aba").len == 5`
    @get pub fn len(self) -> Int = slink.len

    /// The number of distinct non-empty palindromic substrings, `len - 2`; at most `n`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aba").distinct == 3`
    @get pub fn distinct(self) -> Int = slink.len - 2

    /// The length of the palindrome of `state`: `-1` at the odd root, `0` at the even root. The odd
    /// root's `-1` is a value, not a sentinel: a child's length is its parent's plus 2.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let t = PalindromicTree<Char>.of("aba"); t.pal_len(4) == 3 and t.pal_len(0) == -1`
    pub fn pal_len(self, state: Int) -> Int:
        with lint(unproven_list_index: allow)
        return plen[state]

    /// The state of the longest proper palindromic suffix of `state`'s palindrome. A one-letter
    /// palindrome links to the even root `1`; both roots link to the odd root `0`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let t = PalindromicTree<Char>.of("aba"); t.link(4) == 2 and t.link(2) == 1 and t.link(1) == 0`
    pub fn link(self, state: Int) -> Int:
        with lint(unproven_list_index: allow)
        return slink[state]

    /// The state of `a + P + a` where `P` is `state`'s palindrome, or `None` when that is not in
    /// the tree. From the odd root it gives the one-letter palindrome `a`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let t = PalindromicTree<Char>.of("aba"); t.step(3, 'a') == Some(4) and t.step(1, 'a') == None`
    pub fn step(self, state: Int, a: A) -> ?Int:
        with lint(unproven_list_index: allow)
        let t = next[state * alpha + a.ord]
        if t == -1: None else: t

    /// The difference `pal_len(state) - pal_len(link(state))`, the period that groups
    /// palindromic suffixes into series; `0` at both roots.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `PalindromicTree<Char>.of("aba").diff(4) == 2`
    pub fn diff(self, state: Int) -> Int:
        with lint(unproven_list_index: allow)
        return diff_val[state]

    /// The nearest state up the suffix links whose `diff` differs from `state`'s: `link(state)`
    /// when the diffs differ, else `series_link(link(state))`. Following it from any state
    /// reaches a root in \(O(\log n)\) steps. Both roots give `0`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `PalindromicTree<Char>.of("aba").series_link(4) == 2`
    /// example: `PalindromicTree<Char>.of("aaaa").series_link(5) == 1`
    pub fn series_link(self, state: Int) -> Int:
        with lint(unproven_list_index: allow)
        return ser_link[state]

    /// For each state, how many times its palindrome occurs in the pushed string, overlaps
    /// included; `0` at both roots. The entries from `2` sum to `manacher(s).count()`.
    ///
    /// cost: \(O(\mathit{states})\)
    /// order: keyed by state
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aba").occ_counts() == [0, 0, 2, 1, 1]`
    pub fn occ_counts(self) -> List<Int>:
        with lint(unproven_list_index: allow)
        var occ = [cnt[v] for v in 0..cnt.len]
        unchecked:
            for v in (2..cnt.len).rev:
                if slink[v] >= 2: occ[slink[v]] += occ[v]
        occ

    /// Prepends the letter `a` (a double-ended tree) and returns the state of the longest
    /// palindromic prefix of the new string, creating it if it is new.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: never (`a` is an `A`)
    /// example: `var t = PalindromicTree<Char>.of("ba"); t.push_front('a') == 4`
    pub fn push_front(mut self, a: A) -> Int:
        let c = a.ord
        front.push(a)
        barrier = undo.len
        let n = self.size()
        unchecked:
            var cur = first
            while true:
                let after = plen[cur] + 1
                if after < n and self.at(after) == a: break
                cur = slink[cur]
            if next[cur * alpha + c] == -1:
                let link_to = if cur == 0:
                    1
                else:
                    var f = slink[cur]
                    while true:
                        let after = plen[f] + 1
                        if after < n and self.at(after) == a: break
                        f = slink[f]
                    next[f * alpha + c]
                self.grow(cur, c, link_to)
            first = next[cur * alpha + c]
            cnt[first] += 1
            if plen[first] == n: last = first
        first

    /// Undoes the most recent `push` not yet undone, removing the state it created if any, so a
    /// DFS over a tree of strings can back out.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: when there is no `push` to undo
    /// example: `var t = PalindromicTree<Char>.of("ab"); t.pop(); t.distinct == 1 and t.last_state == 2`
    pub fn pop(mut self):
        assert not undo.is_empty, "`pop` needs a `push` to undo"
        assert undo.len > barrier, "`pop` can't undo a `push` made before a `push_front`"
        unchecked:
            if plen[last] == self.size(): first = firsts.pop()!
            cnt[last] -= 1
            if cnt[last] == 0:
                next[made.pop()!] = -1
                for _ in 0..alpha: next.pop()
                slink.pop()
                plen.pop()
                diff_val.pop()
                ser_link.pop()
                cnt.pop()
        chars.pop()
        last = undo.pop()!

    /// The state of the longest palindromic suffix of the current string; the even root `1`
    /// when the string is empty.
    ///
    /// cost: \(O(1)\)
    /// ties: None; the longest suffix is unique
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aba").last_state == 4`
    @get pub fn last_state(self) -> Int = last

    /// True when `word` is a palindrome occurring in the pushed string, that is, a state of the
    /// tree.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\)
    /// empty: `true` (the even root)
    /// traps: never
    /// example: `let t = PalindromicTree<Char>.of("aba"); t.contains("aba") and not t.contains("ab")`
    pub fn contains<S: Seq>(self, word: S) -> Bool:
        let w = word.collect()
        let m = w.len
        if m == 0: return true
        unchecked:
            if not is_palindrome(w): return false
            var at = if m % 2 == 1: 0 else: 1
            for i in ((m - 1) / 2 + 1..m):
                let t = next[at * alpha + w[i].ord]
                if t == -1: return false
                at = t
            if m % 2 == 1:
                let t = next[w[(m - 1) / 2].ord]
                if t == -1: return false
        true

    /// `dp[i]` is the least number of palindromes the first `i` pushed letters split into, for
    /// `i in 0..=n`, by the series-link DP.
    ///
    /// cost: \(O(n \log n)\)
    /// ties: None; only the count is returned
    /// order: keyed by prefix length `0..=n`
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aab").min_palindrome_partition() == [0, 1, 1, 2]`
    pub fn min_palindrome_partition(self) -> List<Int>:
        let n = self.size()
        var t = PalindromicTree<A>.new()
        var dp = List.filled(n + 1, 0)
        var series = List.filled(slink.len, 0)
        unchecked:
            for i in 1..=n:
                var v = t.push(self.at(i - 1))
                dp[i] = i
                while t.plen[v] > 0:
                    series[v] = dp[i - (t.plen[t.ser_link[v]] + t.diff_val[v])]
                    if t.diff_val[v] == t.diff_val[t.slink[v]]: series[v] min= series[t.slink[v]]
                    dp[i] min= series[v] + 1
                    v = t.ser_link[v]
        dp

/// Dynamic trie over alphabet domain `A`. \(O(\lvert \mathit{word} \rvert)\) per operation; flat `|A|`-stride table.
pub struct Trie<A>:
    alpha: 0..=2_000_000_000
    next: List<-1..=2_000_000_000>
    ends: List<0..=2_000_000_000>
    pass: List<0..=2_000_000_000>

    /// An empty trie over the alphabet domain `A`: one root node `0`, no words. Call it as
    /// `Trie<A>()`. Nodes are numbered in creation order.
    ///
    /// cost: \(O(\lvert A \rvert)\)
    /// traps: never
    /// example: `Trie<Char>().len == 1`
    pub fn new() -> Trie<A>:
        let k = A.count
        Trie(k, List.filled(k, -1), [0], [0])

    /// The root node, for the empty word: always `0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Trie<Char>().root == 0`
    @get pub fn root(self) -> Int = 0

    /// The number of nodes, the root included. `remove` keeps nodes, so this never shrinks
    /// except by `clear`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.add("ac"); t.len == 4`
    @get pub fn len(self) -> Int = ends.len

    /// True when no words are stored (every count is zero), whatever nodes remain.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("a"); t.remove("a"); t.is_empty`
    @get pub fn is_empty(self) -> Bool = pass[0] == 0

    /// The child of `node` along `a`, or `None` when it was never created.
    ///
    /// cost: \(O(1)\)
    /// traps: when `node` is outside `0..self.len`
    /// example: `var t = Trie<Char>(); t.add("ab"); t.step(0, 'a') == Some(1) and t.step(1, 'z') == None`
    pub fn step(self, node: Int, a: A) -> ?Int:
        with lint(unproven_list_index: allow)
        let t = next[node * alpha + a.ord]
        if t == -1: None else: t

    /// The node of `word`, or `None` when no stored word starts with `word`
    /// (`count_prefix(word) == 0`).
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\)
    /// empty: `Some(0)` unless the trie is empty
    /// ties: None; a word has one node
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.find("a") == Some(1) and t.find("b") == None`
    pub fn find<S: Seq>(self, word: S) -> ?Int:
        with lint(unproven_list_index: allow)
        var at = 0
        for a in word:
            let t = next[at * alpha + a.ord]
            if t == -1: return None
            at = t
        at

    /// Inserts one copy of `word` and returns its terminal node; `count(word)` and
    /// `count_prefix(p)` for every prefix `p` of `word` grow by one.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\)
    /// empty: an empty `word` is stored at the root
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab") == 2`
    pub fn add<S: Seq>(mut self, word: S) -> Int:
        with lint(unproven_list_index: allow)
        var at = 0
        pass[0] += 1
        for a in word:
            let idx = at * alpha + a.ord
            if next[idx] == -1:
                let nw = ends.len
                for _ in 0..alpha: next.push(-1)
                ends.push(0)
                pass.push(0)
                next[idx] = nw
            at = next[idx]
            pass[at] += 1
        ends[at] += 1
        at

    /// Removes one copy of `word`, reversing one `add`, and returns `true`; returns `false` and
    /// changes nothing when `count(word) == 0`.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\)
    /// empty: removes a copy of the empty word, if stored
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.remove("ab") and not t.remove("ab")`
    pub fn remove<S: Seq>(mut self, word: S) -> Bool:
        with lint(unproven_list_index: allow)
        let node = self.find(word)
        if node == None or ends[node!] == 0: return false
        var at = 0
        pass[0] -= 1
        for a in word:
            at = next[at * alpha + a.ord]
            pass[at] -= 1
        ends[at] -= 1
        true

    /// The number of copies of exactly `word` stored.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\)
    /// empty: copies of the empty word
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.add("ab"); t.count("ab") == 2 and t.count("a") == 0`
    pub fn count<S: Seq>(self, word: S) -> Int:
        with lint(unproven_list_index: allow)
        let node = self.find(word)
        if node == None: 0 else: ends[node!]

    /// The number of stored words (with multiplicity) that start with `prefix`.
    ///
    /// cost: \(O(\lvert \mathit{prefix} \rvert)\)
    /// empty: every stored word
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.add("ac"); t.count_prefix("a") == 2`
    pub fn count_prefix<S: Seq>(self, prefix: S) -> Int:
        with lint(unproven_list_index: allow)
        let node = self.find(prefix)
        if node == None: 0 else: pass[node!]

    /// The number of stored words that end at `node`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `node` is outside `0..self.len`
    /// example: `var t = Trie<Char>(); let v = t.add("ab"); t.ends_at(v) == 1`
    pub fn ends_at(self, node: Int) -> Int:
        with lint(unproven_list_index: allow)
        return ends[node]

    /// The number of stored words that pass through or end at `node`; at the root, all of them.
    ///
    /// cost: \(O(1)\)
    /// traps: when `node` is outside `0..self.len`
    /// example: `var t = Trie<Char>(); t.add("ab"); t.add("ac"); t.pass_at(1) == 2`
    pub fn pass_at(self, node: Int) -> Int:
        with lint(unproven_list_index: allow)
        return pass[node]

    /// True when at least one copy of exactly `word` is stored: `count(word) > 0`.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert)\)
    /// empty: whether the empty word is stored
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.contains("ab") and not t.contains("a")`
    pub fn contains<S: Seq>(self, word: S) -> Bool:
        if let node = self.find(word): ends[node] > 0 else: false

    /// Removes every word and node, leaving just the root.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.clear(); t.is_empty and t.len == 1`
    pub fn clear(mut self):
        next = List.filled(alpha, -1)
        ends = [0]
        pass = [0]

    /// The `k`-th (0-based) stored word in lexicographic order, copies counted, as its letters;
    /// `None` when fewer than `k + 1` words are stored. A word comes before its extensions.
    ///
    /// cost: \(O(\mathit{depth} \cdot \lvert A \rvert)\)
    /// ties: copies of one word are adjacent and interchangeable
    /// order: the letters of the word, first to last
    /// traps: when `k < 0`
    /// example: `var t = Trie<Char>(); t.add("b"); t.add("ab"); t.add("a"); t.kth(1) == Some(['a', 'b']) and t.kth(3) == None`
    pub fn kth(self, k: Int) -> ?List<A>:
        assert k >= 0, "`kth(k)` needs `k >= 0`"
        if k >= pass[0]: return None
        var left = k
        var node = 0
        var word: List<A> = []
        while left >= ends[node]:
            left -= ends[node]
            for c in 0..alpha:
                let child = next[node * alpha + c]
                if child == -1: continue
                if left < pass[child]:
                    word.push(A.try_at(c)!)
                    node = child
                    break
                left -= pass[child]
        Some(word)

    /// The number of stored words (with multiplicity) lexicographically smaller than `word`.
    ///
    /// cost: \(O(\lvert \mathit{word} \rvert \cdot \lvert A \rvert)\)
    /// empty: `0` for an empty `word`
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("b"); t.add("ab"); t.add("a"); t.count_less("b") == 2`
    pub fn count_less<S: Seq>(self, word: S) -> Int:
        var total = 0
        var at = 0
        unchecked:
            for a in word:
                total += ends[at]
                let c = a.ord
                for d in 0..c:
                    let t = next[at * alpha + d]
                    if t != -1: total += pass[t]
                at = next[at * alpha + c]
                if at == -1: return total
        total

/// \(O(\mathit{total} \cdot \lvert A \rvert)\) build, \(O(1)\) step. Aho-Corasick multi-pattern automaton over alphabet domain `A`.
/// Built in \(O(\mathit{sum} \lvert p \rvert \cdot \lvert A \rvert)\) flat memory and time; `step`, `link`, and `hit` are \(O(1)\).
///
/// not a container: an automaton; `len` counts states, which are walked by `step`.
pub struct AhoCorasick<A>:
    alpha: 0..=2_000_000_000
    goto: List<-1..=2_000_000_000>
    fail: List<0..=2_000_000_000>
    ends: List<List<0..=2_000_000_000>>
    exit: List<-1..=2_000_000_000>
    order: List<0..=2_000_000_000>
    lens: List<0..=2_000_000_000>
    pat_state: List<0..=2_000_000_000>

    /// The Aho-Corasick automaton of `patterns`; call it as `AhoCorasick<A>(patterns)`. Pattern
    /// `p` is `patterns[p]`. States are the trie nodes of the patterns, numbered in creation
    /// order with the root `0`; equal patterns share a state.
    ///
    /// cost: \(O(\mathit{sum} \lvert p \rvert \cdot \lvert A \rvert)\)
    /// empty: no patterns gives just the root; an empty pattern ends at the root and matches at every `at` in `0..=|text|`
    /// traps: when an element of a pattern is not in `A`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).len == 4`
    pub fn new<S: Seq>(patterns: List<S>) -> AhoCorasick<A>:
        with lint(unproven_list_index: allow)
        let k = A.count
        var goto: List<-1..=2_000_000_000> = List.filled(k, -1)
        var ends: List<List<0..=2_000_000_000>> = [[]]
        var lens: List<0..=2_000_000_000> = List.filled(patterns.len, 0)
        var pat_state: List<0..=2_000_000_000> = List.filled(patterns.len, 0)
        for i in 0..patterns.len:
            var at = 0
            var plen = 0
            for a in patterns[i]:
                let idx = at * k + a.ord
                if goto[idx] == -1:
                    let nw = ends.len
                    for _ in 0..k: goto.push(-1)
                    ends.push([])
                    goto[idx] = nw
                at = goto[idx]
                plen += 1
            lens[i] = plen
            pat_state[i] = at
            ends[at].push(i)
        let n = ends.len
        var fail: List<0..=2_000_000_000> = List.filled(n, 0)
        var exit: List<-1..=2_000_000_000> = List.filled(n, -1)
        var order: List<0..=2_000_000_000> = List.filled(n, 0)
        var head = 0
        var tail = 1
        while head < tail:
            let s = order[head]
            head += 1
            let base = s * k
            let fbase = fail[s] * k
            for c in 0..k:
                let past = if s == 0: 0 else: goto[fbase + c]
                let t = goto[base + c]
                if t != -1:
                    fail[t] = past
                    exit[t] = if ends[past].is_empty: exit[past] else: past
                    order[tail] = t
                    tail += 1
                else:
                    goto[base + c] = past
        AhoCorasick(k, goto, fail, ends, exit, order, lens, pat_state)

    /// The root state, before any letter is read: always `0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `AhoCorasick<Char>(["a"]).root == 0`
    @get pub fn root(self) -> Int = 0

    /// The number of states, the root included.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).len == 4`
    @get pub fn len(self) -> Int = fail.len

    /// The state after reading `a` in `state`: the longest pattern prefix that is a suffix of
    /// what has been read. Total, so it never fails.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let ac = AhoCorasick<Char>(["a", "ab", "b"]); ac.step(2, 'b') == 3 and ac.step(1, 'a') == 1`
    pub fn step(self, state: Int, a: A) -> Int:
        with lint(unproven_list_index: allow)
        return goto[state * alpha + a.ord]

    /// The failure link of `state`: the state of its longest proper suffix that is a pattern
    /// prefix; `0` at the root.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let ac = AhoCorasick<Char>(["a", "ab", "b"]); ac.link(2) == 3 and ac.link(0) == 0`
    pub fn link(self, state: Int) -> Int:
        with lint(unproven_list_index: allow)
        return fail[state]

    /// True when some pattern ends at `state` itself or at a state up its failure links:
    /// `words_at(state)` is non-empty.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let ac = AhoCorasick<Char>(["ab", "b"]); ac.hit(2) and not ac.hit(1)`
    pub fn hit(self, state: Int) -> Bool:
        with lint(unproven_list_index: allow)
        return not ends[state].is_empty or exit[state] != -1

    /// Every pattern that ends at `state` or up its failure links, by index.
    ///
    /// cost: \(O(\mathit{output})\)
    /// order: longest pattern first; equal patterns by increasing index; a new `List`, keyed from 0
    /// traps: when `state` is outside `0..self.len`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).words_at(2) == [1, 2]`
    pub fn words_at(self, state: Int) -> List<Int>:
        with lint(unproven_list_index: allow)
        var found: List<Int> = []
        var at = state
        while at != -1:
            for w in ends[at]: found.push(w)
            at = exit[at]
        found

    /// Every occurrence of every pattern in `text`, as `(at, pattern)` with `at` the 0-based
    /// start: exactly the pairs with `text[at..at + patterns[pattern].len] == patterns[pattern]`.
    ///
    /// cost: \(O(\lvert \mathit{text} \rvert + \mathit{output})\)
    /// empty: an empty `text` yields only matches of an empty pattern
    /// ties: at one end position, longest pattern first, then increasing index (`words_at`)
    /// order: by end position
    /// traps: when an element of `text` is not in `A`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).find_all("ab").collect() == [(0, 0), (0, 1), (1, 2)]`
    pub fn find_all<S: Seq>(self, text: S) -> yields (at: Int, pattern: Int):
        with lint(unproven_list_index: allow)
        var state = 0
        var i = 0
        for a in text:
            state = goto[state * alpha + a.ord]
            i += 1
            var at = state
            while at != -1:
                for w in ends[at]: yield (at: i - lens[w], pattern: w)
                at = exit[at]

    /// For each pattern, how many times it occurs in `text`, overlaps included: the number of
    /// `find_all` pairs with that index.
    ///
    /// cost: \(O(\lvert \mathit{text} \rvert + \mathit{states})\)
    /// empty: all zeros for an empty `text` (an empty pattern still counts once)
    /// order: keyed by pattern index
    /// traps: when an element of `text` is not in `A`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).counts("abb") == [1, 1, 2]`
    pub fn counts<S: Seq>(self, text: S) -> List<Int>:
        with lint(unproven_list_index: allow)
        var visits = List.filled(fail.len, 0)
        var state = 0
        for a in text:
            state = goto[state * alpha + a.ord]
            visits[state] += 1
        for i in (1..order.len).rev: visits[fail[order[i]]] += visits[order[i]]
        var total = List.filled(lens.len, 0)
        for s in 0..fail.len:
            for w in ends[s]: total[w] = visits[s]
        total

    /// Every state in BFS order of the trie (root first, then by depth, equal depths by state
    /// number), so each state comes after its failure link; iterate backwards for bottom-up DP.
    ///
    /// cost: \(O(\mathit{states})\)
    /// order: non-decreasing depth; ties by smaller state number
    /// traps: never
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).fail_order() == [0, 1, 3, 2]`
    pub fn fail_order(self) -> List<Int> = order.collect()

    /// The nearest state strictly up the failure links of `state` at which a pattern ends, or
    /// `None` when there is None.
    ///
    /// cost: \(O(1)\)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let ac = AhoCorasick<Char>(["a", "ab", "b"]); ac.exit_link(2) == Some(3) and ac.exit_link(1) == None`
    pub fn exit_link(self, state: Int) -> ?Int:
        assert 0 <= state < fail.len, "`exit_link(state)` needs `0 <= state < len`"
        if exit[state] == -1: None else: Some(exit[state])

    /// The state where `patterns[pattern]` ends.
    ///
    /// cost: \(O(1)\)
    /// traps: when `pattern` is outside `0..patterns.len`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).pattern_state(1) == 2`
    pub fn pattern_state(self, pattern: Int) -> Int:
        assert 0 <= pattern < pat_state.len, "`pattern_state(pattern)` needs a pattern index"
        pat_state[pattern]

/// \(O(\lvert \mathit{pat} \rvert \cdot \lvert A \rvert \cdot \log n)\) amortized insert/delete, \(O(\lvert \mathit{text} \rvert \cdot \log n)\) query.
/// Online multi-pattern Aho-Corasick via log-decomposed static buckets.
///
/// not a container: an automaton; `len` counts patterns, which are walked by `step`.
pub struct DynamicAhoCorasick<A>:
    alpha: 0..=2_000_000_000
    pub n: Int
    stored: Trie<A>
    added: AcBuckets<A>
    removed: AcBuckets<A>

    /// An empty online pattern set over the alphabet domain `A`; call it as
    /// `DynamicAhoCorasick<A>()`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `DynamicAhoCorasick<Char>().is_empty`
    pub fn new() -> DynamicAhoCorasick<A> = DynamicAhoCorasick<A>(
        A.count,
        0,
        Trie<A>(),
        AcBuckets<A>([], [], []),
        AcBuckets<A>([], [], []),
    )

    /// The number of patterns stored, copies counted.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("a"); d.add("a"); d.len == 2`
    @get pub fn len(self) -> Int = n

    /// True when no patterns are stored.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("a"); d.remove("a"); d.is_empty`
    @get pub fn is_empty(self) -> Bool = n == 0

    /// Stores one more copy of `pattern`.
    ///
    /// cost: \(O(\lvert \mathit{pattern} \rvert \cdot \lvert A \rvert \cdot \log n)\) amortized
    /// empty: an empty pattern matches at every `at` in `0..=|text|`
    /// traps: when an element of `pattern` is not in `A`
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("ab"); d.len == 1`
    pub fn add<S: Seq>(mut self, pattern: S):
        let word: List<A> = pattern.collect()
        let _ = stored.add(word)
        added.add(word)
        n += 1

    /// Removes one stored copy of `pattern`.
    ///
    /// cost: \(O(\lvert \mathit{pattern} \rvert \cdot \lvert A \rvert \cdot \log n)\) amortized
    /// empty: removes a copy of the empty pattern
    /// traps: when no copy of `pattern` is stored
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("ab"); d.remove("ab"); d.count_matches("ab") == 0`
    pub fn remove<S: Seq>(mut self, pattern: S):
        let word: List<A> = pattern.collect()
        assert stored.remove(word), "`remove(pattern)` needs a stored copy of `pattern`"
        removed.add(word)
        n -= 1

    /// The total number of occurrences of all stored patterns in `text`, overlaps and copies
    /// counted.
    ///
    /// cost: \(O(\lvert \mathit{text} \rvert \cdot \log n)\)
    /// empty: `0` for an empty `text` unless an empty pattern is stored
    /// traps: when an element of `text` is not in `A`
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("ab"); d.add("b"); d.count_matches("abb") == 3`
    pub fn count_matches<S: Seq>(self, text: S) -> Int = added.count(text) - removed.count(text)

struct AcBuckets<A>:
    pats: List<List<List<A>>>
    acs: List<AhoCorasick<A>>
    hits: List<List<Int>>

    fn add(mut self, word: List<A>):
        with lint(unproven_list_index: allow)
        var group: List<List<A>> = [word]
        while not pats.is_empty and pats.last!.len == group.len:
            for w in pats.last!: group.push(w)
            let _ = pats.pop()
            let _ = acs.pop()
            let _ = hits.pop()
        let ac = AhoCorasick<A>(group)
        var hit = List.filled(ac.len, 0)
        for v in ac.fail_order():
            hit[v] = ac.ends[v].len + (if v == 0: 0 else: hit[ac.fail[v]])
        pats.push(group)
        acs.push(ac)
        hits.push(hit)

    fn count<S: Seq>(self, text: S) -> Int:
        with lint(unproven_list_index: allow)
        var total = 0
        for b in 0..acs.len:
            var state = 0
            total += hits[b][0]
            for a in text:
                state = acs[b].step(state, a)
                total += hits[b][state]
        total

// ---- tests

fn random_word(n: Int, letters: Int) -> List<Char> = [
    Char(Int('a') + rand(0, letters - 1)) for _ in 0..n
]

fn common_prefix<T>(s: List<T>, a: Int, b: Int) -> Int:
    var k = 0
    while a + k < s.len and b + k < s.len and s[a + k] == s[b + k]: k += 1
    k

fn suffix_less<T>(s: List<T>, a: Int, b: Int) -> Bool:
    let k = common_prefix(s, a, b)
    if b + k == s.len: return false
    a + k == s.len or s[a + k] < s[b + k]

@test
fn z_and_prefix_function_match_their_definitions():
    assert z_function("aaabaab") == [7, 2, 1, 0, 2, 1, 0]
    assert prefix_function("abacaba") == [0, 0, 1, 0, 1, 2, 3]
    assert z_function("").len == 0 and prefix_function("").len == 0
    for _ in 0..40:
        let s = random_word(rand(1, 14), 2)
        let n = s.len
        let z = z_function(s)
        let p = prefix_function(s)
        for i in 0..n:
            assert z[i] == common_prefix(s, 0, i)
            assert p[i] == [k for k in 0..=i if s[0..k] == s[i + 1 - k..i + 1]].max()!
        assert period(s) == [d for d in 1..=n if s[0..n - d] == s[d..n]].min()!

@test
fn find_all_matches_a_scan():
    assert find_all("abababa", "aba").collect() == [0, 2, 4]
    assert ("abababa".find_all("aba")).collect() == [0, 2, 4]
    assert find_all("abc", "").collect() == [0, 1, 2]
    assert find_all("ab", "abc").collect() == []
    for _ in 0..40:
        let text = random_word(rand(0, 20), 2)
        let pat = random_word(rand(1, 4), 2)
        let want = [i for i in 0..=text.len - pat.len if text.starts_with(pat, at: i)]
        assert find_all(text, pat).collect() == want
    // any list of comparable values, not only characters
    assert find_all([1, 2, 1, 2, 1], [1, 2, 1]).collect() == [0, 2]

@test
fn suffix_array_and_lcp_match_sorting_the_suffixes():
    assert SuffixArray("banana").sa == [5, 3, 1, 0, 4, 2]
    assert SuffixArray("banana").lcp == [1, 3, 0, 0, 2]
    assert SuffixArray("").sa == [] and SuffixArray("x").sa == [0] and SuffixArray("x").lcp == []
    assert SuffixArray("aaaaa").sa == [4, 3, 2, 1, 0]
    assert SuffixArray("abcdef").sa == [0, 1, 2, 3, 4, 5]
    assert SuffixArray("fedcba").sa == [5, 4, 3, 2, 1, 0]
    for _ in 0..40:
        let s = random_word(rand(1, 18), rand(1, 3))
        let sa = SuffixArray(s).sa
        var want = [i for i in 0..s.len]
        want.sort(by: i => [s[j] for j in i..s.len])
        assert sa == want
        let lcp = SuffixArray(s).lcp
        for j in 0..s.len - 1: assert lcp[j] == common_prefix(s, sa[j], sa[j + 1])
    // Large alphabet (>256 distinct values) and negative ints exercise the fallback path
    let big_uniq = [300 - i for i in 0..270]
    assert SuffixArray(big_uniq).sa == [269 - i for i in 0..270]
    var big_dup = [i % 260 - 130 for i in 0..280]
    big_dup.push(-130)
    big_dup.push(-129)
    let sa_big = SuffixArray(big_dup).sa
    assert sa_big.len == big_dup.len
    for j in 0..sa_big.len - 1: assert suffix_less(big_dup, sa_big[j], sa_big[j + 1])

@test
fn rolling_hash_over_str_list_and_domain_arrays():
    // 1. Str substring hashes match brute-force equality
    for _ in 0..20:
        let s = Str(random_word(rand(1, 16), 3))
        let h = RollingHash(s)
        assert h.len == s.len
        for l in 0..s.len:
            for r in l..=s.len:
                let hv = h.get(l..r)
                assert hv == h.slice(l, r)
                if l < r: assert hv == h.get(l..=r - 1)
        for _ in 0..30:
            let len = rand(1, s.len)
            let a = rand(0, s.len - len)
            let b = rand(0, s.len - len)
            assert (h.get(a..a + len) == h.get(b..b + len)) == (s[a..a + len] == s[b..b + len])
        // combine matches full range for every split
        for mid in 0..=s.len:
            assert h.combine(h.get(..mid), h.get(mid..s.len), s.len - mid) == h.all
    // 2. Domain-indexed [D]Int array (including negative values and zeros)
    domain D = 1..=8
    let arr = [D] i => [0, 0, -5, 7, 0, 0, -5, 7][i.ord]
    let hd = RollingHash(arr)
    assert hd.get(D(1)..=D(4)) == hd.get(D(5)..=D(8))
    assert hd.get(D(1)..=D(1)) != hd.get(D(1)..=D(2))  // [0] vs [0, 0] never collide
    assert hd.get(D(3)..D(3)) == HashVal(0, 0)
    assert hd.get(..D(5)) == hd.get(D(5)..)
    // 3. Shared bases compare across two distinct sequences
    let h1 = RollingHash("abacaba")
    let h2 = RollingHash("xxabayy", bases: h1.bases)
    assert h1.get(0..3) == h2.get(2..5) and h1.get(4..7) == h2.get(2..5)
    assert h1.get(0..3) != h2.get(0..3)
    // 4. Nested 2D hashing: [R]HashVal fed into a second RollingHash with independent salted bases
    domain R = 0..3
    domain C = 0..3
    let g = [R, C] (r, c) => [[1, 2, 3], [4, 5, 6], [7, 8, 9]][r.ord][c.ord]
    let gt = [C, R] (c, r) => g[r, c]
    let row_b = RollingHash([g[R(0), c] for c in C]).bases
    let row_hashes = [R] r => RollingHash([g[r, c] for c in C], bases: row_b).all
    let col_hashes = [C] c => RollingHash([gt[c, r] for r in R], bases: row_b).all
    let h_grid = RollingHash(row_hashes)
    let h_trans = RollingHash(col_hashes, bases: h_grid.bases)
    assert h_grid.bases != row_b
    assert h_grid.all != h_trans.all
    // Same matrix hashes equal under matching row & outer bases
    let row_hashes2 = [R] r => RollingHash([g[r, c] for c in C], bases: row_b).all
    assert RollingHash(row_hashes2, bases: h_grid.bases).all == h_grid.all

@test
fn repeat_writes_a_string_over():
    assert "ab".repeat(3) == "ababab"
    assert "0".repeat(4) == "0000"
    assert "ab".repeat(0) == "" and "".repeat(5) == ""

@test
fn starts_ends_and_search_on_text():
    let s = "Banana"
    assert s.starts_with("Ban") and not s.starts_with("an") and s.starts_with("")
    assert s.ends_with("ana") and not s.ends_with("Bananas") and s.ends_with("")
    assert s.search("nan") == Some(2) and s.search("x") == None and s.search("") == Some(0)

@test
fn lower_and_upper_change_only_letters():
    assert Str(c.lower for c in "aZ-9") == "az-9"
    assert Str(c.upper for c in "aZ-9") == "AZ-9"

@test
fn digit_is_the_value_of_a_decimal_character():
    assert '0'.digit == 0 and '7'.digit == 7 and sum(c.digit for c in "409") == 13

@test
fn count_counts_one_character():
    assert "banana".count(c => c == 'a') == 3 and "banana".count(c => c == 'z') == 0 and "".count(c => c == 'a') == 0

@test
fn rsearch_gives_the_last_place():
    assert "banana".rsearch('a') == Some(5)
    assert "banana".rsearch('b') == Some(0)
    assert "banana".rsearch('x') == None
    assert "".rsearch('a') == None

@test
fn borders_prefix_matches_and_overlap_match_brute():
    assert borders("abacaba").collect() == [3, 1]
    assert borders("aaaa").collect() == [3, 2, 1]
    assert borders("").collect() == []
    assert "ababab".prefix_matches("aba") == [1, 2, 3, 2, 3, 2]
    assert overlap("abcab", "cabx") == 3
    assert overlap("", "abc") == 0 and overlap("abc", "") == 0
    for _ in 0..40:
        let s = random_word(rand(0, 12), 2)
        let n = s.len
        let want_b = [k for k in (1..n).rev if s[0..k] == s[n - k..n]]
        assert borders(s).collect() == want_b
        let pat = random_word(rand(1, 5), 2)
        let pm = s.prefix_matches(pat)
        for i in 0..n:
            var want = 0
            for k in 1..=min(pat.len, i + 1):
                if s[i + 1 - k..i + 1] == pat[0..k]: want = k
            assert pm[i] == want
        var want_ov = 0
        for k in 1..=min(n, pat.len):
            if s[n - k..n] == pat[0..k]: want_ov = k
        assert overlap(s, pat) == want_ov

@test
fn lyndon_factorization_and_min_rotation_match_definitions():
    assert lyndon_factorization("") == []
    assert lyndon_factorization("abab") == [0, 2]
    assert lyndon_factorization("cba") == [0, 1, 2]
    assert min_rotation("") == None
    assert min_rotation("cba") == Some(2)
    assert min_rotation("baca") == Some(3)
    assert min_rotation("aaaa") == Some(0)
    assert max_rotation("baca") == Some(2)
    for _ in 0..40:
        let s = random_word(rand(1, 14), 3)
        let n = s.len
        let starts = lyndon_factorization(s)
        assert starts[0] == 0
        var prev: ?List<Char> = None
        for idx in 0..starts.len:
            let lo = starts[idx]
            let hi = if idx + 1 < starts.len: starts[idx + 1] else: n
            let w = [s[j] for j in lo..hi]
            // Each factor is Lyndon: strictly less than all its non-empty proper suffixes
            for cut in 1..w.len:
                assert w < [w[j] for j in cut..w.len]
            if let p = prev: assert p >= w
            prev = w
        // min_rotation matches brute force over all cyclic shifts
        var best = 0
        var best_rot = [s[j] for j in 0..n]
        for r in 1..n:
            let rot = [s[(r + j) % n] for j in 0..n]
            if rot < best_rot:
                best = r
                best_rot = rot
        assert min_rotation(s) == Some(best)

@test
fn manacher_and_is_palindrome_match_brute():
    assert is_palindrome("") and is_palindrome("a") and is_palindrome("abba")
    assert not is_palindrome("ab") and not is_palindrome("abca")
    let mp = manacher("abacaba")
    assert mp.odd == [1, 2, 1, 4, 1, 2, 1]
    assert mp.longest() == (l: 0, r: 7)
    assert manacher("abba").even == [0, 0, 2, 0]
    assert manacher("").count() == 0 and manacher("").longest() == (l: 0, r: 0)
    for _ in 0..40:
        let s = random_word(rand(0, 14), 2)
        let n = s.len
        let p = manacher(s)
        var total = 0
        var best = (l: 0, r: 0)
        for l in 0..n:
            for r in l..=n:
                let ok = is_palindrome([s[j] for j in l..r])
                assert p.is_palindrome(l..r) == ok
                if r > l and ok:
                    total += 1
                    if r - l > best.r - best.l: best = (l:, r:)
        assert p.count() == total
        assert p.longest() == best

@test
fn suffix_array_struct_queries_match_brute():
    let empty = SuffixArray("")
    assert empty.len == 0 and empty.distinct() == 0 and empty.count("a") == 0
    for _ in 0..30:
        let s = random_word(rand(1, 16), 3)
        let n = s.len
        let sa = SuffixArray(s)
        assert sa.len == n
        for i in 0..n:
            for j in 0..n:
                assert sa.lcp_between(i, j) == common_prefix(s, i, j)
        // matches and count
        for _ in 0..10:
            let pat = random_word(rand(0, 4), 3)
            let r = sa.rank_range(pat)
            let want = if pat.is_empty: [i for i in 0..n] else: [
                i for i in 0..=n - pat.len if s.starts_with(pat, at: i)
            ]
            assert sa.count(pat) == want.len
            var got = [sa.sa[idx] for idx in r]
            got.sort()
            assert got == want
        // lcp_range
        for _ in 0..10:
            let i = rand(0, n - 1)
            let len = rand(0, n - i)
            let r = sa.lcp_range(i, len)
            var want: List<Int> = [sa.rank[j] for j in 0..n if common_prefix(s, i, j) >= len]
            want.sort()
            assert r.collect() == want
        // compare
        for _ in 0..20:
            let l1 = rand(0, n)
            let r1 = rand(l1, n)
            let l2 = rand(0, n)
            let r2 = rand(l2, n)
            let a = [s[j] for j in l1..r1]
            let b = [s[j] for j in l2..r2]
            let want = if a < b: -1 elif a > b: 1 else: 0
            assert sa.compare(l1, r1, l2, r2) == want
        // distinct substrings
        var seen = Set<List<Char>>()
        for l in 0..n:
            for r in l + 1..=n: seen.add([s[j] for j in l..r])
        assert sa.distinct() == seen.len

@test
fn rolling_hash_lcp_lcs_and_hash2d_match_brute():
    for _ in 0..25:
        let s = random_word(rand(1, 14), 2)
        let t = random_word(rand(1, 14), 2)
        let hs = RollingHash(s)
        let ht = RollingHash(t, bases: hs.bases)
        for i in 0..=s.len:
            for j in 0..=s.len:
                assert hs.lcp(i, j) == common_prefix(s, i, j)
                var cs = 0
                while cs < i and cs < j and s[i - 1 - cs] == s[j - 1 - cs]: cs += 1
                assert hs.lcs(i, j) == cs
        for _ in 0..20:
            let a = rand(0, s.len)
            let b = rand(0, t.len)
            var k = 0
            while a + k < s.len and b + k < t.len and s[a + k] == t[b + k]: k += 1
            assert hs.lcp(a, b, of: ht) == k
    // Hash2D matches brute rectangle equality
    domain R = 0..5
    domain C = 0..6
    let g = [R, C] (r, c) => rand(0, 1)
    let h2 = Hash2D<R, C>((r, c) => g[r, c])
    for _ in 0..100:
        let (hgt, wid) = (rand(1, 3), rand(1, 3))
        let (r1, c1, r2, c2) = (
            rand(0, 5 - hgt),
            rand(0, 6 - wid),
            rand(0, 5 - hgt),
            rand(0, 6 - wid),
        )
        let same = (0..hgt).all(
            i => (0..wid).all(j => g[R(r1 + i), C(c1 + j)] == g[R(r2 + i), C(c2 + j)])
        )
        let a = h2.rect(R(r1)..=R(r1 + hgt - 1), C(c1)..=C(c1 + wid - 1))
        let b = h2.rect(R(r2)..=R(r2 + hgt - 1), C(c2)..=C(c2 + wid - 1))
        assert (a == b) == same

@test
fn suffix_automaton_matches_brute_and_linear_occurrences():
    domain A = 0..2
    for _ in 0..30:
        let n = rand(0, 10)
        let s = [A(rand(0, 1)) for _ in 0..n]
        let sam = SuffixAutomaton<A>.of(s)
        let occ = sam.occ_counts()
        let order = sam.topo_order()
        assert order.len == sam.len
        for i in 1..order.len: assert sam.longest(order[i - 1]) <= sam.longest(order[i])
        for v in 1..sam.len:
            assert sam.shortest(v) == sam.longest(sam.link(v)!) + 1
            assert sam.shortest(v) <= sam.longest(v)
        for len in 1..=4:
            for code in 0..(1 << len):
                let w = [A((code >> i) % 2) for i in 0..len]
                let times = sum(Int((0..len).all(j => s[i + j] == w[j])) for i in 0..=n - len)
                assert sam.contains(w) == (times > 0)
                if let v = sam.find(w):
                    assert occ[v] == times
                    if len == sam.longest(v):
                        var first = -1
                        for i in 0..=n - len:
                            if (0..len).all(j => s[i + j] == w[j]):
                                first = i + len - 1
                                break
                        assert sam.first_pos(v) == Some(first)
        var seen = Set<List<A>>()
        for l in 0..n:
            for r in l + 1..=n: seen.add([s[j] for j in l..r])
        assert sam.distinct() == seen.len
        assert sam.len <= max(2 * n, 1)
        let m = rand(0, 10)
        let t = [A(rand(0, 1)) for _ in 0..m]
        let (lcs_len, lcs_end) = sam.lcs(t)
        var brute_lcs = 0
        for i in 0..m:
            for j in i + 1..=m:
                if [t[k] for k in i..j] in seen: brute_lcs max= j - i
        assert lcs_len == brute_lcs
        if lcs_len > 0: assert [t[k] for k in lcs_end - lcs_len..lcs_end] in seen

@test
fn palindromic_tree_distinct_occurrences_and_series_dp():
    domain A = 0..2
    for _ in 0..30:
        let n = rand(0, 12)
        let s = [A(rand(0, 1)) for _ in 0..n]
        var pt = PalindromicTree<A>.new()
        let inf = 1_000_000
        var dp: List<Int> = List.filled(n + 1, inf)
        var series_dp: List<Int> = List.filled(n + 3, inf)
        dp[0] = 0
        for i in 0..n:
            var v = pt.push(s[i])
            while pt.pal_len(v) > 0:
                series_dp[v] = dp[i + 1 - pt.pal_len(pt.series_link(v)) - pt.diff(v)]
                if pt.diff(v) == pt.diff(pt.link(v)):
                    series_dp[v] min= series_dp[pt.link(v)]
                dp[i + 1] min= series_dp[v] + 1
                v = pt.series_link(v)
        // Brute distinct palindromes and total palindromic substrings
        var pals = Set<List<A>>()
        var total_pals = 0
        for l in 0..n:
            for r in l + 1..=n:
                let sub = [s[j] for j in l..r]
                if is_palindrome(sub):
                    pals.add(sub)
                    total_pals += 1
        assert pt.distinct == pals.len
        let occ = pt.occ_counts()
        assert sum(occ[v] for v in 2..pt.len) == total_pals
        // Brute minimum palindromic partition DP
        var brute: List<Int> = List.filled(n + 1, inf)
        brute[0] = 0
        for r in 1..=n:
            for l in 0..r:
                if is_palindrome([s[j] for j in l..r]): brute[r] min= brute[l] + 1
        assert dp[n] == brute[n]

@test
fn trie_add_remove_and_prefix_counts():
    domain A = 0..3
    for _ in 0..25:
        var t = Trie<A>()
        var words: List<List<A>> = []
        for _ in 0..30:
            let w = [A(rand(0, 2)) for _ in 0..rand(0, 4)]
            if rand(0, 3) > 0:
                let node = t.add(w)
                assert t.ends_at(node) >= 1 and t.pass_at(node) >= 1
                words.push(w)
            else:
                let i = words.find(w)
                assert t.remove(w) == (i != None)
                if i != None: words = [words[j] for j in 0..words.len if j != i!]
            let q = [A(rand(0, 2)) for _ in 0..rand(0, 3)]
            assert t.count(q) == sum(Int(w == q) for w in words)
            assert t.count_prefix(q) == sum(
                Int(w.len >= q.len and (0..q.len).all(i => w[i] == q[i])) for w in words
            )

@test
fn aho_corasick_matches_counts_and_avoidance_dp():
    domain A = 0..2
    for _ in 0..30:
        let patterns = [[A(rand(0, 1)) for _ in 0..rand(1, 4)] for _ in 0..rand(1, 5)]
        let text = [A(rand(0, 1)) for _ in 0..rand(0, 14)]
        let ac = AhoCorasick<A>(patterns)
        var want: List<(Int, Int)> = []
        for i in 0..text.len:
            for w in 0..patterns.len:
                let k = patterns[w].len
                if i + k <= text.len and (0..k).all(
                    j => text[i + j] == patterns[w][j]
                ): want.push((i, w))
        let got = [(m.at, m.pattern) for m in ac.find_all(text)]
        assert got.sorted() == want.sorted()
        assert ac.counts(text) == [sum(Int(p.1 == w) for p in want) for w in 0..patterns.len]
        assert ac.len <= 1 + sum(w.len for w in patterns)
    // Avoidance DP test
    let pats = [[A(0), A(1)], [A(1), A(1)]]
    let ac = AhoCorasick<A>(pats)
    var ways = List.filled(ac.len, 0)
    ways[ac.root] = 1
    for _ in 0..6:
        var nxt = List.filled(ac.len, 0)
        for s in 0..ac.len:
            for a in A:
                let t = ac.step(s, a)
                if not ac.hit(t): nxt[t] += ways[s]
        ways = nxt
    var brute = 0
    for code in 0..(1 << 6):
        let text = [A((code >> i) % 2) for i in 0..6]
        if ac.find_all(text).collect().is_empty: brute += 1
    assert ways.sum() == brute

@test
fn indexed_views_and_common_prefix_suffix():
    assert common_prefix_len("abacaba", "abax") == 3
    assert "abacaba".common_prefix_len("abax") == 3
    assert common_suffix_len("abacaba", "zzcaba") == 4
    assert "abacaba".common_suffix_len("zzcaba") == 4
    assert common_prefix_len("abacaba"[2..7], "acaz") == 3
    assert common_suffix_len("abacaba"[0..5], "xxaca") == 3
    assert common_prefix_len([1, 2, 3], [1, 2, 4, 5]) == 2
    assert common_suffix_len([1, 2, 3], [9, 2, 3]) == 2
    assert period("xxabababyy"[2..8]) == 2
    assert ("ababa".find_all("aba")).collect() == [0, 2]
    assert "xxababaxx"[2..7].find_all("aba").collect() == [2, 4]
    assert "ababa".prefix_matches("aba") == [1, 2, 3, 2, 3]
    assert overlap("abcab", "abxyz") == 2
    assert lyndon_factorization("bbab") == [0, 1, 2]
    assert min_rotation("cab") == 1
    let pal = manacher("abacaba")
    assert pal.is_palindrome(0..7)
    assert pal.is_palindrome(1..6)
    assert pal.is_palindrome(1..=5)
    assert not pal.is_palindrome(0..4)
    domain Pos = 1..=5
    let arr: [Pos]Char = [Pos] i => "ababa"[i.ord]
    assert manacher(arr).is_palindrome(Pos(1)..=Pos(5))
    assert not manacher(arr).is_palindrome(Pos(1)..=Pos(4))
    assert period(arr) == 2

@test
fn lyndon_runs_matches_brute():
    fn brute_runs(s: List<Int>) -> List<(l: Int, r: Int, period: Int)>:
        let n = s.len
        var out = List<(l: Int, r: Int, period: Int)>()
        for l in 0..n:
            for r in l + 2..=n:
                let min_p = first(p for p in 1..=(r - l) / 2 if (l..r - p).all(i => s[i] == s[i + p])) ?? 0
                if min_p == 0: continue
                if l > 0 and s[l - 1] == s[l - 1 + min_p]: continue
                if r < n and s[r] == s[r - min_p]: continue
                out.push((l:, r:, period: min_p))
        out

    assert lyndon_runs("").is_empty
    assert lyndon_runs("a").is_empty
    assert lyndon_runs("aa") == [(l: 0, r: 2, period: 1)]
    assert lyndon_runs("ababa") == [(l: 0, r: 5, period: 2)]
    var seed = 424242
    for n in 1..=14:
        for _ in 0..6:
            var s = List<Int>()
            for _ in 0..n:
                seed = (seed * 1103515245 + 12345) & 0x7fffffff
                s.push((seed >> 16) % 3)
            assert lyndon_runs(s) == brute_runs(s)

@test
fn sequence_generality_parity_across_list_array_and_views():
    let xs = [1, 2, 1, 3, 1, 2, 1]
    let pat = [1, 2, 1]
    domain P = 1..=7
    domain Q = 1..=3
    let arr: [P]Int = [P] i => xs[i.ord]
    let parr: [Q]Int = [Q] i => pat[i.ord]
    let padded = [9, 1, 2, 1, 3, 1, 2, 1, 9]
    let view = padded[1..8]
    let pview = padded[1..4]

    // KMP / Z / structural helpers over List<Int>, [D]Int, ListView<Int>, Str, StrView
    assert prefix_function(arr).collect() == prefix_function(xs)
    assert prefix_function(view) == prefix_function(xs)
    assert z_function(arr).collect() == z_function(xs)
    assert z_function(view) == z_function(xs)
    assert find_all(arr, parr).collect() == [P(1), P(5)]
    assert find_all(view, pview).collect() == [1, 5]
    assert prefix_matches(arr, parr).collect() == prefix_matches(xs, pat)
    assert prefix_matches(view, pview) == prefix_matches(xs, pat)
    assert period(arr) == period(xs)
    assert period(view) == period(xs)
    assert borders(arr) == borders(xs)
    assert borders(view) == borders(xs)
    assert overlap(arr, [2, 1, 7]) == overlap(xs, [2, 1, 7])
    assert common_prefix_len(arr, view) == 7
    assert common_suffix_len(arr, view) == 7

    // RollingHash over List<Int>, [D]Int, ListView<Int>, Str, StrView
    let bases = (131, 137)
    let hl = RollingHash(xs, bases:)
    let ha = RollingHash(arr, bases:)
    let hv = RollingHash(copy(view), bases:)
    assert ha.all == hl.all and hv.all == hl.all
    assert ha.get(P(2)..P(6)) == hl.get(1..5) and hv.get(1..5) == hl.get(1..5)
    let hs = RollingHash(copy("xabacabay"[1..8]), bases:)
    assert hs.all == RollingHash("abacaba", bases:).all

    // Suffix array / LCP / SuffixArray.matches over List<Int>, [D]Int, ListView<Int>, Str, StrView
    let sa_l = SuffixArray(xs).sa
    assert SuffixArray(arr).sa == sa_l
    assert SuffixArray(copy(view)).sa == sa_l
    assert SuffixArray(arr).lcp == SuffixArray(xs).lcp
    assert SuffixArray(copy(view)).lcp == SuffixArray(xs).lcp
    let sa_s = SuffixArray("abacaba").sa
    assert SuffixArray(copy("xabacabay"[1..8])).sa == sa_s
    assert SuffixArray(copy("xabacabay"[1..8])).lcp == SuffixArray("abacaba").lcp
    let sa_obj = SuffixArray.new(xs)
    assert sa_obj.rank_range(parr) == sa_obj.rank_range(pat)
    assert sa_obj.count(pview) == 2

    // Manacher / Lyndon / runs over List<Int>, [D]Int, ListView<Int>, StrView
    assert is_palindrome(arr) and is_palindrome(view)
    assert manacher(arr).is_palindrome(P(1)..=P(7))
    assert manacher(view).is_palindrome(0..7)
    assert [k.ord for k in lyndon_factorization(arr)] == lyndon_factorization(xs)
    assert [k - 1 for k in lyndon_factorization(view)] == lyndon_factorization(xs)
    assert min_rotation(arr)!.ord == min_rotation(xs)!
    assert min_rotation(view)! - 1 == min_rotation(xs)!
    assert lyndon_runs(arr) == lyndon_runs(xs)
    assert lyndon_runs(view) == lyndon_runs(xs)
    assert lyndon_runs("xabababy"[1..7]) == lyndon_runs("ababab")

    // Alphabet-domain automata over List<Sym>, [D]Sym, ListView<Sym>
    domain Sym = 0..4
    let syms = [Sym(x) for x in xs]
    let sarr: [P]Sym = [P] i => syms[i.ord]
    let spadded = [Sym(0), Sym(1), Sym(2), Sym(1), Sym(3), Sym(1), Sym(2), Sym(1), Sym(0)]
    let sview = spadded[1..8]
    let spat: [Q]Sym = [Q] i => Sym(pat[i.ord])
    let spview = spadded[1..4]

    let sam_l = SuffixAutomaton<Sym>.of(syms)
    let sam_a = SuffixAutomaton<Sym>.of(sarr)
    let sam_v = SuffixAutomaton<Sym>.of(sview)
    assert sam_a.distinct() == sam_l.distinct() and sam_v.distinct() == sam_l.distinct()
    assert sam_a.contains(spat) and sam_v.contains(spview)
    assert sam_a.lcs(spview) == sam_l.lcs(spview)

    let pt_l = PalindromicTree<Sym>.of(syms)
    let pt_a = PalindromicTree<Sym>.of(sarr)
    let pt_v = PalindromicTree<Sym>.of(sview)
    assert pt_a.distinct == pt_l.distinct and pt_v.distinct == pt_l.distinct

    var tr = Trie<Sym>.new()
    let _ = tr.add(sarr)
    let _ = tr.add(spview)
    assert tr.count(sview) == 1
    assert tr.count_prefix(spat) == 2
    assert tr.remove(spview)
    assert tr.count_prefix(spat) == 1

    let ac = AhoCorasick<Sym>.new([spat])
    assert ac.counts(sarr) == [2]
    assert ac.counts(sview) == [2]
    assert [m.at for m in ac.find_all(sarr)] == [0, 4]
    assert [m.at for m in ac.find_all(sview)] == [0, 4]

@test
fn dynamic_hash_matches_rolling_hash_after_updates():
    for _ in 0..20:
        let n = rand(0, 12)
        var s: List<Int> = [rand(0, 2) for _ in 0..n]
        var h = DynamicHash(s, bases: (131, 137))
        for _ in 0..20:
            if n > 0 and rand(0, 1) == 0:
                let i = rand(0, n - 1)
                let x = rand(0, 2)
                h.set(i, x)
                s[i] = x
            let rh = RollingHash(s, bases: (131, 137))
            assert h.len == n and h.is_empty == (n == 0) and h.all == rh.all
            let l = rand(0, n)
            let r = rand(l, n)
            let back: List<Int> = [s[r - 1 - k] for k in 0..r - l]
            assert h.get(l..r) == rh.get(l..r)
            assert h.rev_get(l..r) == rh.hash_of(back)
            assert h.is_palindrome(l..r) == (back == [s[k] for k in l..r])
    domain P = 0..6
    let g = DynamicHash<P>([P] p => p.ord % 2)
    assert g.len == 6 and g.get(P(0)..P(2)) == g.get(P(2)..P(4))
    assert g.is_palindrome(P(1)..P(4)) and not g.is_palindrome(P(0)..P(2))

@test
fn dynamic_aho_corasick_matches_brute_after_adds_and_removes():
    for _ in 0..10:
        var d = DynamicAhoCorasick<Char>()
        var live: List<List<Char>> = []
        for _ in 0..25:
            if not live.is_empty and rand(0, 2) == 0:
                let i = rand(0, live.len - 1)
                d.remove(live[i])
                live = [live[k] for k in 0..live.len if k != i]
            else:
                let w = random_word(rand(0, 3), 2)
                d.add(w)
                live.push(w)
            assert d.len == live.len and d.is_empty == live.is_empty
            let text = random_word(rand(0, 8), 2)
            var want = 0
            for w in live:
                for i in 0..=text.len - w.len:
                    if (0..w.len).all(j => text[i + j] == w[j]): want += 1
            assert d.count_matches(text) == want

/// `gen<Str of alphabet>(len: lo..=hi)`.
pub fn _gen_str(alphabet: Str, lo: Int, hi: Int) -> Str =
    Str([alphabet[rand(0, alphabet.len - 1)] for _ in 0..rand(lo, hi)])
