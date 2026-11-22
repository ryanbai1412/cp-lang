//! String algorithms. They only compare elements, so they work on a `Str`
//! (ASCII bytes) and on any `List<T>` alike. Positions are 0-based.

use std.seq.{reverse}

/// `self` written `n` times in a row, as a new `Str`: `"0".repeat(4)` is `"0000"`.
///
/// cost: O(|self| * n)
/// empty: an empty `self`, or `n == 0`, gives `""`
/// traps: when `n < 0`
/// example: `"ab".repeat(3) == "ababab"`
/// example: `"x".repeat(0) == ""`
pub fn repeat(self: Str, n: Int) -> Str = todo

@allow(unproven_list_index)
/// True when `pat` occurs in `self` starting at position `at` (default 0, so a prefix test):
/// the `pat.len` elements of `self` from `at` equal those of `pat` in order, `false` when they
/// run past the end. Works on any `Indexed` pair (`Str`, `List<T>`, `[D]T`, views); `at`
/// counts positions from 0, and only the values are compared, never the keys.
///
/// cost: O(min(|self| - at, |pat|))
/// empty: an empty `pat` matches at every `at` in `0..=self.len`; an empty `self` starts only with an empty `pat`
/// traps: when `at` is outside `0..=self.len`
/// example: `"banana".starts_with("ban") == true`
/// example: `"banana".starts_with("nan", at: 2) == true`
/// example: `[1, 2, 3].starts_with([2]) == false`
pub fn starts_with<S: Indexed, P: Indexed>(self: S, pat: P, at: Int = 0) -> Bool = todo

@allow(unproven_list_index)
/// True when `self` closes with `pat`: the last `pat.len` elements of `self` equal those of
/// `pat` in order. Works on any `Indexed` pair (`Str`, `List<T>`, `[D]T`, views); only the
/// values are compared, never the keys.
///
/// cost: O(min(|self|, |pat|))
/// empty: an empty `pat` is a suffix of everything; an empty `self` ends only with an empty `pat`
/// traps: never
/// example: `[1, 2, 3].ends_with([2, 3]) == true`
/// example: `"banana".ends_with("nab") == false`
pub fn ends_with<S: Indexed, P: Indexed>(self: S, pat: P) -> Bool = todo

/// The key where `p` first stands in `self`, or `None`. `p` is a `Str` or one `Char`.
///
/// The result is a key of `self`, so on a slice it is valid on the source: `s[l..].search(c)`
/// needs no `+ l`. An empty `p` stands at the first key of `self` (the end key when `self`
/// is empty).
///
/// cost: O(|self| + |p|)
/// empty: an empty `p` gives `Some(first key)`; an empty `self` and non-empty `p` give `None`
/// ties: the leftmost occurrence
/// traps: never
/// example: `"banana".search("nan") == Some(2)`
/// example: `"banana".search('n') == Some(2)`
/// example: `"banana"[2..].search('a') == Some(3)`
/// example: `"banana".search("x") == None`
pub fn search<P>(self: Str, p: P) -> ?Int = todo

/// How many times the character `c` stands in `self`: `"banana".count('a')` is `3`.
///
/// cost: O(|self|)
/// empty: `0`
/// traps: never
/// example: `"banana".count('a') == 3`
/// example: `"".count('a') == 0`
pub fn count(self: Str, c: Char) -> Int = todo

/// The key where `p` last starts in `self`, or `None`. `p` is a `Str` or one `Char`.
///
/// Like `find`, the result is a key of `self`. An empty `p` stands at the end key, the cut
/// after the last element (`self.len` on a whole `Str`).
///
/// cost: O(|self| + |p|)
/// empty: an empty `p` gives `Some(end key)`; an empty `self` and non-empty `p` give `None`
/// ties: the rightmost occurrence
/// traps: never
/// example: `"banana".rsearch("an") == Some(3)`
/// example: `"banana".rsearch('a') == Some(5)`
/// example: `"ab".rsearch("") == Some(2)`
pub fn rsearch<P>(self: Str, p: P) -> ?Int = todo

/// The lowercase letter of an ASCII capital; any other character is returned as it is.
///
/// cost: O(1)
/// ties: none; one character in, one out
/// traps: never
/// example: `'A'.lower == 'a'`
/// example: `'7'.lower == '7'`
@get pub fn lower(self: Char) -> Char = todo

/// The capital of an ASCII lowercase letter; any other character is returned as it is.
///
/// cost: O(1)
/// ties: none; one character in, one out
/// traps: never
/// example: `'a'.upper == 'A'`
/// example: `'?'.upper == '?'`
@get pub fn upper(self: Char) -> Char = todo

/// The value `0..=9` of an ASCII decimal digit: `'7'.digit` is `7`.
///
/// cost: O(1)
/// traps: when `self` is not in `'0'..='9'` (test with `is_digit` first)
/// example: `'7'.digit == 7`
/// example: `'0'.digit == 0`
@get pub fn digit(self: Char) -> Int = todo

/// True when `self` is an ASCII decimal digit, `'0'..='9'`.
///
/// cost: O(1)
/// traps: never
/// example: `'5'.is_digit == true`
/// example: `'x'.is_digit == false`
@get pub fn is_digit(self: Char) -> Bool = todo

/// True when `self` is an ASCII lowercase letter, `'a'..='z'`.
///
/// cost: O(1)
/// traps: never
/// example: `'q'.is_lower == true`
/// example: `'Q'.is_lower == false`
@get pub fn is_lower(self: Char) -> Bool = todo

/// True when `self` is an ASCII capital letter, `'A'..='Z'`.
///
/// cost: O(1)
/// traps: never
/// example: `'Q'.is_upper == true`
/// example: `'q'.is_upper == false`
@get pub fn is_upper(self: Char) -> Bool = todo

/// True when `self` is an ASCII letter, `'a'..='z'` or `'A'..='Z'`.
///
/// cost: O(1)
/// traps: never
/// example: `'Z'.is_alpha == true`
/// example: `'_'.is_alpha == false`
@get pub fn is_alpha(self: Char) -> Bool = todo

/// True when `self` is an ASCII letter or decimal digit.
///
/// cost: O(1)
/// traps: never
/// example: `'9'.is_alnum == true`
/// example: `' '.is_alnum == false`
@get pub fn is_alnum(self: Char) -> Bool = todo

/// True when `self` is one of the four ASCII whitespace characters `' '`, `'\t'`, `'\n'`, `'\r'`.
///
/// cost: O(1)
/// traps: never
/// example: `'\t'.is_space == true`
/// example: `'x'.is_space == false`
@get pub fn is_space(self: Char) -> Bool = todo

/// A copy of `self` with every ASCII capital replaced by its lowercase letter (`Char.lower`).
///
/// cost: O(|self|)
/// empty: `""`
/// traps: never
/// example: `"AbC1".to_lower() == "abc1"`
pub fn to_lower(self: Str) -> Str = todo

/// A copy of `self` with every ASCII lowercase letter replaced by its capital (`Char.upper`).
///
/// cost: O(|self|)
/// empty: `""`
/// traps: never
/// example: `"AbC1".to_upper() == "ABC1"`
pub fn to_upper(self: Str) -> Str = todo

/// A copy of `self` with leading and trailing whitespace (`Char.is_space`) removed; inner
/// whitespace is kept.
///
/// cost: O(|self|)
/// empty: `""`; so is a string of only whitespace
/// traps: never
/// example: `"  a b \n".trim() == "a b"`
/// example: `" \t ".trim() == ""`
pub fn trim(self: Str) -> Str = todo

/// A copy of `self` with leading whitespace (`Char.is_space`) removed.
///
/// cost: O(|self|)
/// empty: `""`; so is a string of only whitespace
/// traps: never
/// example: `"  a b \n".trim_start() == "a b \n"`
pub fn trim_start(self: Str) -> Str = todo

/// A copy of `self` with trailing whitespace (`Char.is_space`) removed.
///
/// cost: O(|self|)
/// empty: `""`; so is a string of only whitespace
/// traps: never
/// example: `"  a b \n".trim_end() == "  a b"`
pub fn trim_end(self: Str) -> Str = todo

/// The pieces of `self` between occurrences of `sep`, as new strings.
///
/// Every `sep` is a cut, so `k` separators give `k + 1` pieces; adjacent, leading and
/// trailing separators give empty pieces. Use `split_whitespace` to drop empty pieces.
///
/// cost: O(|self|)
/// empty: `[""]`, one empty piece
/// order: left to right; a new `List`, keyed from 0
/// traps: never
/// example: `"a,b,,c".split(',') == ["a", "b", "", "c"]`
/// example: `"a,".split(',') == ["a", ""]`
/// example: `"".split(',') == [""]`
pub fn split(self: Str, sep: Char) -> List<Str> = todo

/// `self` cut at the first `sep` into `(before, after)`, neither holding that `sep`; `None`
/// when `sep` does not occur. Later separators stay in `after`.
///
/// cost: O(|self|)
/// empty: `None`
/// traps: never
/// example: `"key=val=x".split_once('=') == Some(("key", "val=x"))`
/// example: `"abc".split_once('=') == None`
pub fn split_once(self: Str, sep: Char) -> ?(Str, Str) = todo

/// The maximal runs of non-whitespace characters of `self` (`Char.is_space` separates), as
/// new strings. No piece is empty.
///
/// cost: O(|self|)
/// empty: `[]`; so is a string of only whitespace
/// order: left to right; a new `List`, keyed from 0
/// traps: never
/// example: `" a  b\tc ".split_whitespace() == ["a", "b", "c"]`
/// example: `"   ".split_whitespace() == []`
pub fn split_whitespace(self: Str) -> List<Str> = todo

/// The elements of `parts` in order with `self` between each adjacent pair: `", ".join(xs)`.
///
/// Each element is written as string interpolation `"{x}"` writes it, so `Str`, `Char` and
/// numbers all join without a `map`.
///
/// cost: O(total output length)
/// empty: `""`
/// traps: never
/// example: `", ".join(["a", "b", "c"]) == "a, b, c"`
/// example: `" ".join([1, 2, 3]) == "1 2 3"`
/// example: `"-".join(["x"]) == "x"`
pub fn join<S: Seq>(self: Str, parts: S) -> Str = todo

/// A copy of `self` with every occurrence of `from` replaced by `to`.
///
/// Occurrences are taken greedily left to right and never overlap; the text that `to`
/// inserts is not scanned again.
///
/// cost: O(|self| + |output|)
/// empty: an empty `self` gives `""`
/// traps: when `from` is empty
/// example: `"aaaa".replace("aa", "b") == "bb"`
/// example: `"aaa".replace("aa", "b") == "ba"`
/// example: `"banana".replace("an", "") == "ba"`
pub fn replace(self: Str, from: Str, to: Str) -> Str = todo

/// `self` read as a signed integer in `base`, or `None` if malformed or out of `Int` range.
///
/// The whole string must be an optional `-` followed by one or more digits of `base`; digits
/// past 9 are letters in either case (`'a'` and `'A'` are 10). Whitespace, a `+` sign or any
/// other character makes it malformed.
///
/// cost: O(|self|)
/// empty: `None`
/// traps: when `base` is outside `2..=36`
/// example: `"ff".parse_int(base: 16) == Some(255)`
/// example: `"-12".parse_int() == Some(-12)`
/// example: `"12a".parse_int() == None`
pub fn parse_int(self: Str, base: Int = 10) -> ?Int = todo

/// `self` read as a `T` by the same rules as `read<T>()` (core 10.2), or `None` if a token is
/// malformed, out of `T`'s range, missing, or left over.
///
/// Tokens are separated by whitespace exactly as on standard input, so a tuple or list type
/// reads several tokens.
///
/// cost: O(|self|)
/// empty: `None` unless `T` reads zero tokens
/// traps: never
/// example: `"-12".parse<Mod<7>>() == Some(2)`
/// example: `"3 4".parse<(Int, Int)>() == Some((3, 4))`
/// example: `"3 4".parse<Int>() == None`
pub fn parse<T: Read>(self: Str) -> ?T = todo

@allow(unproven_list_index)
/// The Z-array of `s`: `z[i]` is the length of the longest common prefix of `s` and the
/// suffix starting at `i`; `z` at the first key is `s.len`.
///
/// Keyed by `s.keys`, so `z_function(s[l..])[i]` takes `i` from the slice. Works on any
/// `Indexed` sequence (`Str`, `List<T>`, `[D]T`, `Range<I>`, views) without conversion.
///
/// cost: O(n)
/// empty: an empty array
/// order: keyed by the keys of `s`
/// traps: never
/// example: `z_function("aabxaab") == [7, 1, 0, 0, 3, 1, 0]`
pub fn z_function<S: Indexed, K>(s: S) -> [K]Int = todo

@allow(unproven_list_index)
/// The prefix function (failure function) of `s`: `p[i]` is the length of the longest
/// proper prefix of `s` up to and including `i` that is also a suffix of it.
///
/// Keyed by `s.keys`. Works on any `Indexed` sequence without conversion.
///
/// cost: O(n)
/// empty: an empty array
/// order: keyed by the keys of `s`
/// traps: never
/// example: `prefix_function("aabaaab") == [0, 1, 0, 1, 2, 2, 3]`
pub fn prefix_function<S: Indexed, K>(s: S) -> [K]Int = todo

@allow(unproven_list_index)
/// The start key of every occurrence of `pat` in `self`, overlapping ones included
/// (Knuth-Morris-Pratt).
///
/// Works on any `Indexed` pair (`Str`, `List<T>`, `[D]T`, views) without conversion; keys
/// are those of `self`. An empty pattern occurs at every key and at the end key.
///
/// cost: O(|self| + |pat|)
/// empty: an empty `pat` yields every key of `self` then its end key; an empty `self` yields nothing for a non-empty `pat`
/// ties: every occurrence is yielded; none is preferred
/// order: increasing start key
/// traps: never
/// example: `"aaaa".find_all("aa").collect() == [0, 1, 2]`
/// example: `"ab".find_all("").collect() == [0, 1, 2]`
pub fn find_all<S: Indexed, P: Indexed, K>(self: S, pat: P) -> yields K = todo

@allow(unproven_list_index)
/// The length of the longest common prefix of `self` and `other`, comparing values in order
/// from the first key of each. Works on any `Indexed` pair.
///
/// cost: O(min(|self|, |other|))
/// empty: `0` when either is empty
/// traps: never
/// example: `"abcd".common_prefix_len("abxd") == 2`
pub fn common_prefix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int = todo

@allow(unproven_list_index)
/// The length of the longest common suffix of `self` and `other`, comparing values in order
/// from the last key of each. Works on any `Indexed` pair.
///
/// cost: O(min(|self|, |other|))
/// empty: `0` when either is empty
/// traps: never
/// example: `"xbcd".common_suffix_len("abcd") == 3`
pub fn common_suffix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int = todo

/// The shortest period of `s`: the smallest `d >= 1` with `s[i] == s[i + d]` wherever both
/// sides exist. It is `s.len` when nothing shorter works.
///
/// cost: O(n)
/// empty: `0`
/// traps: never
/// example: `period("abcabca") == 3`
/// example: `period("aaaa") == 1`
/// example: `period("abc") == 3`
pub fn period<S: Indexed>(s: S) -> Int = todo

@allow(unproven_list_index)
/// True when `s` reads the same forwards and backwards. Works on any `Indexed` sequence.
///
/// cost: O(n)
/// empty: `true`
/// traps: never
/// example: `is_palindrome("abba") == true`
/// example: `is_palindrome([1, 2]) == false`
pub fn is_palindrome<S: Indexed>(s: S) -> Bool = todo

@allow(unproven_list_index)
/// The length of every non-empty proper border of `s` (a prefix that is also a suffix,
/// shorter than `s`). Works on any `Indexed` sequence.
///
/// cost: O(n)
/// empty: yields nothing; so does a single element
/// order: longest first
/// traps: never
/// example: `borders("abacaba").collect() == [3, 1]`
/// example: `borders("aaa").collect() == [2, 1]`
pub fn borders<S: Indexed>(s: S) -> yields Int = todo

@allow(unproven_list_index)
/// For each key `i` of `self`, the length of the longest prefix of `pat` that ends at `i`
/// (the KMP matcher's state after reading `self[i]`).
///
/// A full match counts as `pat.len`; matching then continues from the longest border of
/// `pat`, so `res[i] == pat.len` exactly where an occurrence ends. Keyed by `self.keys`.
///
/// cost: O(|self| + |pat|)
/// empty: an empty `self` gives an empty array; an empty `pat` gives all zeros
/// order: keyed by the keys of `self`
/// traps: never
/// example: `"abab".prefix_matches("ab") == [1, 2, 1, 2]`
pub fn prefix_matches<S: Indexed, P: Indexed, K>(self: S, pat: P) -> [K]Int = todo

@allow(unproven_list_index)
/// The length of the longest suffix of `a` that is also a prefix of `b`, at most
/// `min(|a|, |b|)`; how far `b` can slide onto the end of `a`.
///
/// cost: O(|a| + |b|)
/// empty: `0` when either is empty
/// traps: never
/// example: `overlap("abcab", "abd") == 2`
/// example: `overlap("abc", "abc") == 3`
pub fn overlap<A: Indexed, B: Indexed>(a: A, b: B) -> Int = todo

@allow(unproven_list_index)
/// The start key of each factor of the Lyndon factorization of `s` (Duval): the unique way
/// to cut `s` into Lyndon words `w1 >= w2 >= ...` in lexicographic order.
///
/// cost: O(n)
/// empty: `[]`
/// order: increasing keys; the first is the first key of `s`
/// traps: never
/// example: `lyndon_factorization("banana") == [0, 1, 3, 5]`
pub fn lyndon_factorization<S: Indexed, K>(s: S) -> List<K> = todo

@allow(unproven_list_index)
/// The key at which the lexicographically smallest cyclic rotation of `s` starts (Booth).
///
/// cost: O(n)
/// empty: `None`
/// ties: the smallest key, when `s` is periodic
/// traps: never
/// example: `min_rotation("baca") == Some(3)`
/// example: `min_rotation("abab") == Some(0)`
pub fn min_rotation<S: Indexed, K>(s: S) -> ?K = todo

/// The key at which the lexicographically greatest cyclic rotation of `s` starts.
///
/// cost: O(n)
/// empty: `None`
/// ties: the smallest key, when `s` is periodic
/// traps: never
/// example: `max_rotation("baca") == Some(2)`
/// example: `max_rotation("abab") == Some(1)`
pub fn max_rotation<S: Indexed, K>(s: S) -> ?K = todo

/// O(|pat| * |A|). Prefix (KMP) automaton over alphabet domain `A`, for O(1) transitions in DP.
pub struct KmpAutomaton<A>:
    alpha: 0..=2_000_000_000
    goto: List<0..=2_000_000_000>
    pi: List<0..=2_000_000_000>

    /// The KMP automaton of `pat` over the alphabet domain `A`: one state per matched prefix
    /// length `0..=pat.len`, with a full transition table. Call it as `KmpAutomaton<A>(pat)`.
    ///
    /// cost: O(|pat| * |A|)
    /// empty: an empty `pat` gives one state, `0`, which matches everywhere
    /// traps: when an element of `pat` is not in `A`
    /// example: `KmpAutomaton<Char>("aba").len == 4`
    pub fn new<S: Seq>(pat: S) -> KmpAutomaton<A> = todo

    /// The number of states, `pat.len + 1`; the states are `0..=pat.len` and `pat.len` means a
    /// full match.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `KmpAutomaton<Char>("ab").len == 3`
    @get pub fn len(self) -> Int = todo

    /// The matched prefix length after reading `a` in `state`. From the full-match state the
    /// automaton falls back along `link` first, so matching continues and overlaps are found.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let k = KmpAutomaton<Char>("aba"); k.step(3, 'b') == 2 and k.step(0, 'b') == 0`
    pub fn step(self, state: Int, a: A) -> Int = todo

    /// The failure link of `state`: the length of the longest proper border of `pat[..state]`,
    /// and `0` at state `0`. It equals `prefix_function(pat)[state - 1]` for `state >= 1`.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let k = KmpAutomaton<Char>("aba"); k.link(3) == 1 and k.link(2) == 0`
    pub fn link(self, state: Int) -> Int = todo

    /// The start position (0-based, counted in `text`'s iteration order) of every occurrence of
    /// `pat` in `text`, overlapping ones included.
    ///
    /// cost: O(|text|)
    /// empty: an empty `text` yields nothing unless `pat` is empty; an empty `pat` yields `0..=|text|`
    /// ties: every occurrence is yielded; none is preferred
    /// order: increasing start
    /// traps: when an element of `text` is not in `A`
    /// example: `KmpAutomaton<Char>("aba").find_all("ababa").collect() == [0, 2]`
    pub fn find_all<S: Seq>(self, text: S) -> yields Int = todo

/// The start position of every occurrence of `pat` in `self`, overlapping ones included,
/// by bit-parallel Shift-And. Faster than `find_all` for short patterns on small alphabets.
///
/// Positions count from 0 in `self`'s order; on a whole `Str` or `List` they are its keys.
///
/// cost: O(|self| * ceil(|pat| / 64))
/// empty: an empty `pat` yields `0..=|self|`; an empty `self` yields nothing for a non-empty `pat`
/// order: increasing start
/// traps: never
/// example: `"abab".shift_and("ab").collect() == [0, 2]`
pub fn shift_and<S: Indexed, P: Indexed>(self: S, pat: P) -> yields Int = todo

/// The start position of every place where `pat` matches `self` when each `wildcard` in
/// `pat` matches any one element. A `wildcard` in `self` is an ordinary element.
///
/// Positions count from 0 in `self`'s order, as for `shift_and`.
///
/// cost: O(|self| * ceil(|pat| / 64))
/// empty: an empty `pat` yields `0..=|self|`; an empty `self` yields nothing for a non-empty `pat`
/// ties: every match is yielded; none is preferred
/// order: increasing start
/// traps: never
/// example: `"abcab".find_all_wildcard("a?").collect() == [0, 3]`
/// example: `"a?c".find_all_wildcard("ab").collect() == []`
pub fn find_all_wildcard<S: Indexed, P: Indexed>(
    self: S,
    pat: P,
    wildcard: Char = '?',
) -> yields Int = todo

/// O(n) space, O(1) queries. Manacher palindrome radii. `odd[i]` is the number of palindromes centred on `s[i]`;
/// `even[i]` is the number centred between `s[i - 1]` and `s[i]`.
pub struct Palindromes(odd: List<Int>, even: List<Int>):
    /// The length of the sequence the radii were computed for.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `manacher("aba").len == 3`
    @get pub fn len(self) -> Int = todo

    /// True when the sequence the radii were computed for is empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `manacher("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo

    @allow(unproven_list_index)
    /// The length of the longest odd-length palindrome centred on position `i`, `2 * odd[i] - 1`.
    ///
    /// cost: O(1)
    /// traps: when `i` is outside `0..self.len`
    /// example: `manacher("abacaba").longest_at(3) == 7`
    pub fn longest_at(self, i: Int) -> Int = todo

    @allow(unproven_list_index)
    /// The length of the longest even-length palindrome centred on the cut `i` (between
    /// positions `i - 1` and `i`), `2 * even[i]`; `0` at the cuts `0` and `self.len`.
    ///
    /// cost: O(1)
    /// traps: when `i` is outside `0..=self.len`
    /// example: `manacher("abba").longest_between(2) == 4`
    pub fn longest_between(self, i: Int) -> Int = todo

    /// True when the positions `r` (`l..r`, `l..=r`, or a `Range<D>`) of the original sequence
    /// form a palindrome: `p.is_palindrome(l..r) == is_palindrome(s[l..r])`.
    ///
    /// cost: O(1)
    /// traps: when `r` reaches outside `0..=self.len`
    /// example: `let p = manacher("abacaba"); p.is_palindrome(0..3) and not p.is_palindrome(1..4)`
    pub fn is_palindrome<R>(self, r: R) -> Bool = todo

    /// The number of non-empty palindromic substrings, each occurrence counted:
    /// `sum(odd) + sum(even)`.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `manacher("aaa").count() == 6`
    pub fn count(self) -> Int = todo

    /// The half-open range `(l, r)` of a longest palindromic substring.
    ///
    /// cost: O(n)
    /// ties: the leftmost (smallest `l`)
    /// traps: never
    /// example: `manacher("abacdc").longest() == (0, 3)`
    /// example: `manacher("").longest() == (0, 0)`
    pub fn longest(self) -> (l: Int, r: Int) = todo

@allow(unproven_list_index)
/// The palindrome radii of `s` by Manacher's algorithm: `odd[i]` is the largest `k` with
/// `s[i - k + 1..i + k]` a palindrome, and `even[i]` the largest `k` with `s[i - k..i + k]`
/// one. Works on any `Indexed` sequence; positions count from 0.
///
/// cost: O(n)
/// empty: both lists empty
/// traps: never
/// example: `manacher("abba").count() == 6`
/// example: `manacher("aba").odd == [1, 2, 1]`
pub fn manacher<S: Indexed>(s: S) -> Palindromes = todo

fn induced_sort(
    vec: List<Int>,
    counts: List<Int>,
    is_l: List<Bool>,
    lms: List<Int>,
    mut sa: List<Int>,
) = todo

@allow(unproven_list_index)
fn sa_is(vec: List<Int>, alpha: Int) -> List<Int> = todo

/// O(n) SA-IS + Kasai LCP (O(n log n) with RMQ). Suffix array, inverse rank, adjacent LCP, and O(1) LCP/comparison queries.
pub struct SuffixArray:
    sa: List<Int>
    rank: List<Int>
    lcp: List<Int>

    /// The suffix array of `s` with its inverse and LCP array: `sa[r]` is the start of the
    /// suffix of rank `r`, `rank[sa[r]] == r`, and `lcp[r]` is the LCP of the suffixes of ranks
    /// `r` and `r + 1`. Call it as `SuffixArray(s)`; positions count from 0.
    ///
    /// Elements are compared by `Ord`, so a shorter suffix that is a prefix of a longer one
    /// ranks first.
    ///
    /// cost: O(n) for `Str` and small integer alphabets, O(n log n) otherwise
    /// empty: `sa`, `rank` and `lcp` all empty
    /// traps: never
    /// example: `SuffixArray("banana").sa == [5, 3, 1, 0, 4, 2]`
    /// example: `SuffixArray("banana").lcp == [1, 3, 0, 0, 2]`
    pub fn new<S: Indexed>(s: S) -> SuffixArray = todo

    /// The length `n` of the sequence the suffix array was built over.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SuffixArray("banana").len == 6`
    @get pub fn len(self) -> Int = todo

    /// True when the suffix array was built over an empty sequence.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SuffixArray("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo

    /// The length of the longest common prefix of the suffixes starting at `i` and `j`. The
    /// empty suffix at `n` is allowed, and `lcp_between(i, i) == n - i`.
    ///
    /// cost: O(1)
    /// traps: when `i` or `j` is outside `0..=self.len`
    /// example: `SuffixArray("banana").lcp_between(1, 3) == 3`
    pub fn lcp_between(self, i: Int, j: Int) -> Int = todo

    /// The ranks `lo..hi` of the suffixes that start with `pat`. When there are none it is the
    /// empty range at the rank where `pat` would sort.
    ///
    /// cost: O(|pat| log n)
    /// empty: an empty `pat` gives `0..n`
    /// order: a range of ranks, increasing
    /// traps: never
    /// example: `SuffixArray("banana").rank_range("ana") == 1..3`
    /// example: `SuffixArray("banana").rank_range("c") == 4..4`
    pub fn rank_range<P: Indexed>(self, pat: P) -> Range<Int> = todo

    /// The start of every occurrence of `pat`, read off `sa` over `rank_range(pat)`.
    ///
    /// cost: O(|pat| log n + occurrences)
    /// empty: an empty `pat` yields every start `0..n` (not the end position `n`)
    /// ties: every occurrence is yielded; none is preferred
    /// order: suffix-array order (by rank), not by position
    /// traps: never
    /// example: `SuffixArray("banana").find_all("ana").collect() == [3, 1]`
    pub fn find_all<P: Indexed>(self, pat: P) -> yields Int = todo

    /// The number of occurrences of `pat`, overlapping ones included: `rank_range(pat).len`.
    ///
    /// cost: O(|pat| log n)
    /// empty: an empty `pat` gives `n`
    /// traps: never
    /// example: `SuffixArray("banana").count("an") == 2`
    pub fn count<P: Indexed>(self, pat: P) -> Int = todo

    /// The ranks `lo..hi` of the suffixes whose LCP with the suffix at `i` is at least `len`;
    /// always contains `rank[i]`.
    ///
    /// cost: O(log n)
    /// order: a range of ranks, increasing; it holds `rank[i]`
    /// traps: when `i` is outside `0..n` or `len` outside `0..=n - i`
    /// example: `SuffixArray("banana").lcp_range(1, 3) == 1..3`
    pub fn lcp_range(self, i: Int, len: Int) -> Range<Int> = todo

    /// The lexicographic comparison of `s[l1..r1]` and `s[l2..r2]`: `-1`, `0` or `1`. A proper
    /// prefix compares less.
    ///
    /// cost: O(1)
    /// traps: when a range is not within `0..=n` or has its end before its start
    /// example: `SuffixArray("banana").compare(1, 3, 3, 5) == 0`
    /// example: `SuffixArray("banana").compare(1, 2, 1, 3) == -1`
    pub fn compare(self, l1: Int, r1: Int, l2: Int, r2: Int) -> Int = todo

    /// The number of distinct non-empty substrings: `n * (n + 1) / 2 - sum(lcp)`.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `SuffixArray("banana").distinct() == 15`
    pub fn distinct(self) -> Int = todo

    /// The longest prefix common to every suffix with rank in `r`: the minimum of `lcp[j]` for
    /// `j in r.start..r.end - 1`, or the suffix's own length when `r` holds one rank.
    ///
    /// cost: O(1)
    /// empty: an empty `r` traps
    /// traps: when `r` is empty or reaches outside `0..n`
    /// example: `SuffixArray("banana").lcp_of_ranks(1..3) == 3`
    /// example: `SuffixArray("banana").lcp_of_ranks(3..4) == 6`
    pub fn lcp_of_ranks(self, r: Range<Int>) -> Int = todo

    /// Where the rank range `r` splits in the suffix tree: the `k` in `r.start + 1..r.end` whose
    /// adjacent LCP `lcp[k - 1]` equals `lcp_of_ranks(r)`, so `r.start..k` and `k..r.end` are
    /// the first child and the rest.
    ///
    /// cost: O(1)
    /// empty: an `r` with fewer than two ranks traps
    /// ties: the smallest such `k`
    /// traps: when `r.len < 2` or `r` reaches outside `0..n`
    /// example: `SuffixArray("banana").split(0..3) == 1`
    /// example: `SuffixArray("banana").split(0..6) == 3`
    pub fn split(self, r: Range<Int>) -> Int = todo

    /// The `k`-th (0-based) lexicographically smallest distinct non-empty substring, as the
    /// positions `l..r` of one occurrence, or `None` when `k >= distinct()`.
    ///
    /// cost: O(log n)
    /// ties: the occurrence in the suffix of smallest rank that holds it
    /// order: `k` counts in increasing lexicographic order; the range is increasing positions
    /// traps: when `k < 0`
    /// example: `SuffixArray("banana").kth_distinct(1) == Some(3..5)`
    /// example: `SuffixArray("banana").kth_distinct(15) == None`
    pub fn kth_distinct(self, k: Int) -> ?Range<Int> = todo

/// The suffix array of `s`; the same as `SuffixArray(s)`.
///
/// cost: O(n) for `Str` and small integer alphabets, O(n log n) otherwise
/// empty: an empty suffix array
/// traps: never
/// example: `suffix_array([3, 1, 2]).sa == [1, 2, 0]`
pub fn suffix_array<S: Indexed>(s: S) -> SuffixArray = todo

/// O(1) value. One edge of the compressed suffix tree of a sequence, built in O(n) from SA + LCP.
pub struct SuffixTreeEdge(
    top_size: Int,
    bottom_size: Int,
    parent_len: Int,
    edge_len: Int,
    rank_range: Range<Int>,
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
/// cost: O(n)
/// empty: `[]`
/// order: bottom-up: each edge comes after every edge below it, subtrees in rank order
/// traps: never
/// example: `suffix_tree_edges("aa") == [SuffixTreeEdge(0, 1, 1, 1, 1..2), SuffixTreeEdge(1, 2, 0, 1, 0..2)]`
pub fn suffix_tree_edges<S: Indexed>(s: S) -> List<SuffixTreeEdge> = todo

@allow(unproven_list_index)
/// Every run (maximal repetition) of `s`, by the Lyndon-runs theorem: each `(l, r, period)`
/// is a half-open range `s[l..r]` with smallest period `period`, `r - l >= 2 * period`, that
/// cannot be extended left or right with the same period. Positions count from 0.
///
/// cost: O(n log n) time and memory
/// empty: `[]`
/// order: by `(l, r)`; a new `List`, keyed from 0
/// traps: never
/// example: `lyndon_runs("aabaab") == [(0, 2, 1), (0, 6, 3), (3, 5, 1)]`
/// example: `lyndon_runs("abc") == []`
pub fn lyndon_runs<S: Indexed>(s: S) -> List<(l: Int, r: Int, period: Int)> = todo

// ---- generalized rolling hash

const RH_M1 = 1_000_000_007
const RH_M2 = 1_000_000_009

/// O(1) space and comparison. A 64-bit double-mod polynomial hash value (`Copy + Eq + Ord + Hash`).
/// Can itself be fed into another `RollingHash` (e.g. hashing rows into a column hash for 2D grids).
pub struct HashVal(h1: Int, h2: Int):
    /// One integer that identifies the pair, `(h1 << 31) | h2`, so a `RollingHash` can hash a
    /// sequence of `HashVal` (rows of a grid, say).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `HashVal(1, 2).hash_token == 2147483650`
    @get pub fn hash_token(self) -> Int = todo

/// O(1) per element. Elements accepted by `RollingHash`: `Char`, `Int`, `Bool`, `HashVal`, or any user type
/// implementing `hash_token`.
pub trait Hashable:
    @get fn hash_token(self) -> Int = todo

@get fn ord(self: Int) -> Int = todo

/// O(n) build, O(1) slice hash. Generalized double-mod polynomial rolling hash over `Str`, `List<T>`, or any `[D]T`
/// (`Char`, `Int`, `Bool`, `HashVal`).
///
/// Built in O(n); substring queries `.get(l..r)` / `.slice(l, r)` and `.combine(a, b, b_len)`
/// are O(1). Each constructor draws independent random salted bases unless `bases:` is passed
/// explicitly, so 2D row and column hashes never collide on transposes.
pub struct RollingHash:
    bases: (Int, Int)
    p1: List<Int>
    p2: List<Int>
    pow1: List<Int>
    pow2: List<Int>

    @allow(unproven_list_index)
    /// A double-modulus polynomial rolling hash of `seq`; call it as `RollingHash(seq)`.
    ///
    /// Modulo `M1 = 1_000_000_007` with base `b1` and `M2 = 1_000_000_009` with base `b2`, the
    /// hash of `x_0 .. x_{k-1}` is the sum of `(t(x_i) % M + 1) * b^(k - 1 - i)` where `t` is
    /// `hash_token` (a `Char`'s code, a `Bool`'s `0`/`1`). The bases are drawn at random unless
    /// `bases:` gives them; hashes only compare under the same bases.
    ///
    /// cost: O(n)
    /// empty: an empty hash; `push` can grow it
    /// traps: when a given base is outside `2..M`
    /// example: `let h = RollingHash("abab"); h.get(0..2) == h.get(2..4)`
    pub fn new<S: Seq>(seq: S, bases: ?(Int, Int) = None) -> RollingHash = todo

    /// Appends `x` to the end of the hashed sequence.
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `var h = RollingHash("ab"); h.push('a'); h.all == h.hash_of("aba")`
    pub fn push<T: Hashable>(mut self, x: T) = todo

    /// Removes the last element of the hashed sequence.
    ///
    /// cost: O(1)
    /// traps: when the hash is empty
    /// example: `var h = RollingHash("abc"); h.pop(); h.all == h.hash_of("ab")`
    pub fn pop(mut self) = todo

    /// The hash `seq` would have under `self`'s bases, without building a `RollingHash` of it;
    /// compare it with any slice of `self`.
    ///
    /// cost: O(|seq|)
    /// empty: `HashVal(0, 0)`
    /// traps: never
    /// example: `let h = RollingHash("xab"); h.hash_of("ab") == h.get(1..3)`
    pub fn hash_of<S: Seq>(self, seq: S) -> HashVal = todo

    /// The hash of the concatenation of `parts`, each given as its hash and length, under
    /// `self`'s bases; `combine` folded left to right.
    ///
    /// cost: O(|parts|) amortized
    /// empty: `HashVal(0, 0)`
    /// traps: when a `len` is negative
    /// example: `let h = RollingHash("abcd"); h.concat([(h.get(0..2), 2), (h.get(2..4), 2)]) == h.all`
    pub fn concat(self, parts: List<(hash: HashVal, len: Int)>) -> HashVal = todo

    /// The number of elements hashed.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `RollingHash("abc").len == 3`
    @get pub fn len(self) -> Int = todo

    /// True when no elements are hashed.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `RollingHash("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo

    /// The hash of the whole sequence, `self.get(0..self.len)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let h = RollingHash("abc"); h.all == h.get(0..3)`
    @get pub fn all(self) -> HashVal = todo

    @allow(unproven_list_index)
    fn raw_slice(self, l: Int, r: Int) -> HashVal = todo

    @allow(unproven_list_index)
    fn eq_sub(self, a: Int, b: Int, len: Int) -> Bool = todo

    /// The hash of the half-open slice `l..r` (0-based); `HashVal(0, 0)` when `l == r`.
    ///
    /// cost: O(1)
    /// traps: when `l > r`, `l < 0` or `r > self.len`
    /// example: `let h = RollingHash("abab"); h.slice(0, 2) == h.slice(2, 4) and h.slice(1, 1) == HashVal(0, 0)`
    pub fn slice(self, l: Int, r: Int) -> HashVal = todo

    /// The hash of a range of positions: `l..r`, `l..=r`, `..k`, `k..`, or a `Range<D>` when the
    /// hash was built over a `[D]T`. The same value as `slice`.
    ///
    /// cost: O(1)
    /// traps: when the range reaches outside the hashed sequence
    /// example: `let h = RollingHash("abab"); h.get(0..2) == h.get(2..=3)`
    pub fn get<R>(self, r: R) -> HashVal = todo

    /// The hash of `a`'s sequence followed by `b`'s, where `b` has length `b_len`:
    /// `a * base^b_len + b` per modulus.
    ///
    /// cost: O(1) amortized (the power table grows past `len` on demand)
    /// traps: when `b_len < 0`
    /// example: `let h = RollingHash("abc"); h.combine(h.get(0..1), h.get(1..3), 2) == h.all`
    pub fn combine(self, a: HashVal, b: HashVal, b_len: Int) -> HashVal = todo

    /// The length of the longest common prefix of the suffixes starting at `a` and `b`, by
    /// binary search on hashes (so it can be wrong on a collision).
    ///
    /// cost: O(log n)
    /// traps: when `a` or `b` is outside `0..=self.len`
    /// example: `RollingHash("abcabd").lcp(0, 3) == 2`
    pub fn lcp(self, a: Int, b: Int) -> Int = todo

    /// The length of the longest common suffix of the prefixes `s[..a]` and `s[..b]`, by binary
    /// search on hashes.
    ///
    /// cost: O(log n)
    /// traps: when `a` or `b` is outside `0..=self.len`
    /// example: `RollingHash("xabyab").lcs(3, 6) == 2`
    pub fn lcs(self, a: Int, b: Int) -> Int = todo

    /// The length of the longest common prefix of `self[a..]` and `other[b..]`, by binary search
    /// on hashes. Both hashes must use the same bases.
    ///
    /// cost: O(log min(|self| - a, |other| - b))
    /// traps: when the bases differ, `a` is outside `0..=self.len` or `b` outside `0..=other.len`
    /// example: `RollingHash("abcd", bases: Some((131, 137))).lcp_with(1, RollingHash("xbcx", bases: Some((131, 137))), 1) == 2`
    pub fn lcp_with(self, a: Int, other: RollingHash, b: Int) -> Int = todo

/// O(R * C) build, O(1) rectangle hash. Double-mod 2D polynomial prefix hash over a grid with row domain `R` and column domain `C`.
/// Built in O(R * C); `rect(rows, cols)` is O(1) and returns a `HashVal`.
pub struct Hash2D<R, C>(bases: (Int, Int)):
    stride: Int
    pre1: List<Int>
    pre2: List<Int>
    pow_r1: List<Int>
    pow_c1: List<Int>
    pow_r2: List<Int>
    pow_c2: List<Int>

    @allow(unproven_list_index)
    /// A 2D polynomial prefix hash of the grid whose cell `(r, c)` has the token `code(r, c)`,
    /// for every `r` in `R` and `c` in `C`; call it as `Hash2D<R, C>(code)`.
    ///
    /// Each modulus uses one base along rows and one along columns; they are drawn at random
    /// unless `bases:` gives them. A token enters as `token % M + 1`, as in `RollingHash`.
    ///
    /// cost: O(|R| * |C|)
    /// empty: an empty `R` or `C` gives a hash whose only rectangle is empty
    /// traps: when a given base is outside `2..M`
    /// example: `domain R = 0..2; domain C = 0..4; let h = Hash2D<R, C>((r, c) => c.to_idx % 2); h.rect(R(0)..=R(1), C(0)..=C(1)) == h.rect(R(0)..=R(1), C(2)..=C(3))`
    pub fn new(code: (R, C) -> Int, bases: ?(Int, Int) = None) -> Hash2D<R, C> = todo

    @allow(unproven_list_index)
    /// The hash of the rectangle `rows` x `cols`; equal rectangles of equal shape hash equal.
    /// A rectangle with no rows or no columns hashes to `HashVal(0, 0)`.
    ///
    /// cost: O(1)
    /// empty: an empty `rows` or `cols` gives `HashVal(0, 0)`
    /// traps: never (the ranges are typed by `R` and `C`)
    /// example: `domain R = 0..2; domain C = 0..4; let h = Hash2D<R, C>((r, c) => r.to_idx); h.rect(R(0)..=R(0), C(0)..=C(1)) != h.rect(R(1)..=R(1), C(0)..=C(1))`
    pub fn rect(self, rows: Range<R>, cols: Range<C>) -> HashVal = todo

/// O(1) value. Single-modulus `2^61 - 1` (Mersenne prime) polynomial hash value (`Copy + Eq + Ord + Hash`).
pub struct Hash61(v: U64)

/// O(n) build, O(1) query. Single-modulus `2^61 - 1` polynomial rolling hash with randomized base.
pub struct RollingHash61:
    base: U64
    pre: List<U64>
    pow: List<U64>

    /// A single-modulus polynomial rolling hash of `seq` modulo the Mersenne prime `2^61 - 1`;
    /// call it as `RollingHash61(seq)`. Same scheme as `RollingHash` with one modulus: each
    /// token enters as `token % M + 1`; the base is drawn at random unless `base:` gives it.
    ///
    /// cost: O(n)
    /// empty: an empty hash; `push` can grow it
    /// traps: when a given base is outside `2..2^61 - 1`
    /// example: `let h = RollingHash61("abab"); h.get(0..2) == h.get(2..4)`
    pub fn new<S: Seq>(seq: S, base: ?U64 = None) -> RollingHash61 = todo

    /// The number of elements hashed.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `RollingHash61("abc").len == 3`
    @get pub fn len(self) -> Int = todo

    /// True when no elements are hashed.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `RollingHash61("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo

    /// The hash of the whole sequence, `self.get(0..self.len)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let h = RollingHash61("abc"); h.all == h.get(0..3)`
    @get pub fn all(self) -> Hash61 = todo

    /// The hash of a range of positions (`l..r`, `l..=r`, `..k`, `k..`, or a `Range<D>` when
    /// built over a `[D]T`); an empty range hashes to `Hash61(0)`.
    ///
    /// cost: O(1)
    /// traps: when the range reaches outside the hashed sequence
    /// example: `let h = RollingHash61("abab"); h.get(0..2) == h.get(2..=3)`
    pub fn get<R>(self, r: R) -> Hash61 = todo

    /// The hash of `a`'s sequence followed by `b`'s, where `b` has length `b_len`:
    /// `a * base^b_len + b` modulo `2^61 - 1`.
    ///
    /// cost: O(1) amortized (the power table grows past `len` on demand)
    /// traps: when `b_len < 0`
    /// example: `let h = RollingHash61("abc"); h.combine(h.get(0..1), h.get(1..3), 2) == h.all`
    pub fn combine(self, a: Hash61, b: Hash61, b_len: Int) -> Hash61 = todo

    /// Appends `x` to the end of the hashed sequence.
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `var h = RollingHash61("ab"); h.push('a'); h.push('b'); h.get(0..2) == h.get(2..4)`
    pub fn push<T: Hashable>(mut self, x: T) = todo

    /// Removes the last element of the hashed sequence.
    ///
    /// cost: O(1)
    /// traps: when the hash is empty
    /// example: `var h = RollingHash61("abab"); h.pop(); h.len == 3`
    pub fn pop(mut self) = todo

    /// The length of the longest common prefix of the suffixes starting at `a` and `b`, by
    /// binary search on hashes.
    ///
    /// cost: O(log n)
    /// traps: when `a` or `b` is outside `0..=self.len`
    /// example: `RollingHash61("abcabd").lcp(0, 3) == 2`
    pub fn lcp(self, a: Int, b: Int) -> Int = todo

    /// The length of the longest common suffix of the prefixes `s[..a]` and `s[..b]`, by binary
    /// search on hashes.
    ///
    /// cost: O(log n)
    /// traps: when `a` or `b` is outside `0..=self.len`
    /// example: `RollingHash61("xabyab").lcs(3, 6) == 2`
    pub fn lcs(self, a: Int, b: Int) -> Int = todo

    /// The length of the longest common prefix of `self[a..]` and `other[b..]`, by binary search
    /// on hashes. Both hashes must use the same base.
    ///
    /// cost: O(log min(|self| - a, |other| - b))
    /// traps: when the bases differ, `a` is outside `0..=self.len` or `b` outside `0..=other.len`
    /// example: `RollingHash61("abcd", base: Some(U64(131))).lcp_with(1, RollingHash61("xbcx", base: Some(U64(131))), 1) == 2`
    pub fn lcp_with(self, a: Int, other: RollingHash61, b: Int) -> Int = todo

/// O(n) build, O(log n) point update and range hash. Mutable double-mod rolling hash over domain `D`.
/// Supports `h[i] = x` via W8 `IndexSet`.
pub struct DynamicHash<D>:
    bases: (Int, Int)

    /// A double-modulus rolling hash of `seq` that supports point updates, kept forwards and
    /// backwards in a segment tree; call it as `DynamicHash(seq)`. Positions are `seq`'s keys
    /// (domain `D`); hashes follow the `RollingHash` scheme and bases.
    ///
    /// cost: O(n)
    /// empty: an empty hash
    /// traps: when a given base is outside `2..M`
    /// example: `let h = DynamicHash("abab"); h.get(0..2) == h.get(2..4)`
    pub fn new<S: Indexed>(seq: S, bases: ?(Int, Int) = None) -> DynamicHash<D> = todo

    /// The number of hashed positions, `D.K`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `DynamicHash("abc").len == 3`
    @get pub fn len(self) -> Int = todo

    /// True when there are no positions.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `DynamicHash("").is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo

    /// The hash of the whole sequence, `self.get(..)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let h = DynamicHash("abab"); h.all == h.get(0..4)`
    @get pub fn all(self) -> HashVal = todo

    /// Replaces the element at position `i` with `x`; `h[i] = x` means the same.
    ///
    /// cost: O(log n)
    /// traps: never (`i` is a `D`)
    /// example: `var h = DynamicHash("abxb"); h.set(2, 'a'); h.get(0..2) == h.get(2..4)`
    pub fn set<T: Hashable>(mut self, i: D, x: T) = todo

    /// The hash of the elements over `r` in order (a cut range `j..i` works too, decision 124).
    ///
    /// cost: O(log n)
    /// empty: an empty `r` gives `HashVal(0, 0)`
    /// traps: never (`r` is a `Range<D>`)
    /// example: `let h = DynamicHash("xyxy"); h.get(0..2) == h.get(2..4)`
    pub fn get(self, r: Range<D>) -> HashVal = todo

    /// The hash of the elements over `r` read backwards, comparable with `get`.
    ///
    /// cost: O(log n)
    /// empty: an empty `r` gives `HashVal(0, 0)`
    /// traps: never (`r` is a `Range<D>`)
    /// example: `let h = DynamicHash("abba"); h.rev_get(0..2) == h.get(2..4)`
    pub fn rev_get(self, r: Range<D>) -> HashVal = todo

    /// True when the elements over `r` read the same both ways: `get(r) == rev_get(r)`.
    ///
    /// cost: O(log n)
    /// empty: `true` for an empty `r`
    /// traps: never (`r` is a `Range<D>`)
    /// example: `let h = DynamicHash("abca"); h.is_palindrome(1..2) and not h.is_palindrome(0..4)`
    pub fn is_palindrome(self, r: Range<D>) -> Bool = todo

// ---- alphabet-domain automata

/// O(n * |A|) space, O(1) amortized push. Online suffix automaton over alphabet domain `A`.
/// O(n * |A|) flat memory, O(n) amortized build, O(states) `topo_order` and `occurrences`.
pub struct SuffixAutomaton<A>:
    alpha: 0..=2_000_000_000
    next: List<-1..=2_000_000_000>
    links: List<-1..=2_000_000_000>
    max_len: List<0..=2_000_000_000>
    first_end: List<-1..=2_000_000_000>
    prefix: List<Bool>
    last: 0..=2_000_000_000

    /// An empty suffix automaton over the alphabet domain `A`: only the root state `0`. Call it
    /// as `SuffixAutomaton<A>()`, then `push` letters.
    ///
    /// cost: O(|A|)
    /// traps: never
    /// example: `SuffixAutomaton<Char>().len == 1`
    pub fn new() -> SuffixAutomaton<A> = todo

    /// The suffix automaton of `s`: `SuffixAutomaton<A>()` followed by `push` of each element.
    ///
    /// States are numbered in creation order: each `push` creates its new state, then the
    /// clone if one is needed. For `"abb"` the states are `0` (empty), `1` (`a`), `2` (`ab`),
    /// `3` (`abb`, `bb`) and `4` (`b`, the clone).
    ///
    /// cost: O(|s| * |A|) memory, O(|s|) amortized time
    /// empty: only the root
    /// traps: when an element of `s` is not in `A`
    /// example: `SuffixAutomaton<Char>.of("abb").len == 5`
    pub fn of<S: Seq>(s: S) -> SuffixAutomaton<A> = todo

    @allow(unproven_list_index)
    fn add_empty_state(mut self, length: Int, is_prefix: Bool) -> Int = todo

    @allow(unproven_list_index)
    fn clone_state(mut self, src: Int, length: Int) -> Int = todo

    @allow(unproven_list_index)
    /// Appends the letter `a` to the string the automaton recognises the substrings of.
    ///
    /// cost: O(1) amortized (O(|A|) for a cloned state)
    /// traps: never (`a` is an `A`)
    /// example: `var m = SuffixAutomaton<Char>(); m.push('a'); m.push('b'); m.distinct() == 3`
    pub fn push(mut self, a: A) = todo

    /// The root state, for the empty string: always `0`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("ab").root == 0`
    @get pub fn root(self) -> Int = todo

    /// The number of states, the root included; at most `max(1, 2n - 1)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").len == 5`
    @get pub fn len(self) -> Int = todo

    @allow(unproven_list_index)
    /// The state reached from `state` by the letter `a`, or `None` when that extension is not a
    /// substring.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.step(0, 'b') == Some(4) and m.step(1, 'a') == None`
    pub fn step(self, state: Int, a: A) -> ?Int = todo

    @allow(unproven_list_index)
    /// The suffix link of `state`: the state of its longest suffix that lies in another state;
    /// `None` at the root.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.link(3) == Some(4) and m.link(0) == None`
    pub fn link(self, state: Int) -> ?Int = todo

    @allow(unproven_list_index)
    /// The length of the longest substring in `state`; `0` at the root.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `SuffixAutomaton<Char>.of("abb").longest(3) == 3`
    pub fn longest(self, state: Int) -> Int = todo

    @allow(unproven_list_index)
    /// The length of the shortest substring in `state`, `longest(link) + 1`; `0` at the root.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `SuffixAutomaton<Char>.of("abb").shortest(3) == 2`
    pub fn shortest(
        self,
        state: Int,
    ) -> Int = todo

    @allow(unproven_list_index)
    /// The 0-based position where the first occurrence of `state`'s substrings ends; `-1` at the
    /// root. A clone keeps the position of the state it was cloned from.
    ///
    /// cost: O(1)
    /// ties: the earliest occurrence; later ones are not tracked
    /// traps: when `state` is outside `0..self.len`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.first_pos(4) == 1 and m.first_pos(3) == 2`
    pub fn first_pos(self, state: Int) -> Int = todo

    @allow(unproven_list_index)
    /// The state holding `word`, or `None` when `word` is not a substring; the root for an empty
    /// `word`.
    ///
    /// cost: O(|word|)
    /// empty: an empty `word` gives `Some(0)`
    /// ties: none; a substring lies in exactly one state
    /// traps: when an element of `word` is not in `A`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.find("bb") == Some(3) and m.find("ba") == None`
    pub fn find<S: Seq>(self, word: S) -> ?Int = todo

    /// True when `word` is a substring of the automaton's string: `find(word) != None`.
    ///
    /// cost: O(|word|)
    /// empty: `true` for an empty `word`
    /// traps: when an element of `word` is not in `A`
    /// example: `let m = SuffixAutomaton<Char>.of("abb"); m.contains("ab") and not m.contains("ba")`
    pub fn contains<S: Seq>(self, word: S) -> Bool = todo

    /// The number of distinct non-empty substrings: the sum of `longest(v) - shortest(v) + 1`
    /// over the states other than the root.
    ///
    /// cost: O(states)
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").distinct() == 5`
    pub fn distinct(self) -> Int = todo

    @allow(unproven_list_index)
    /// Every state, by non-decreasing `longest` (counting sort), so each state comes after its
    /// suffix link; iterate it backwards to push values up the links.
    ///
    /// cost: O(states)
    /// empty: `[0]` for an empty automaton
    /// ties: smaller state number first
    /// order: non-decreasing `longest`; a new `List`, keyed from 0
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").topo_order() == [0, 1, 4, 2, 3]`
    pub fn topo_order(self) -> List<Int> = todo

    @allow(unproven_list_index)
    /// For each state, the number of end positions of its substrings in the automaton's
    /// string; the root counts every position, `n`.
    ///
    /// cost: O(states)
    /// empty: `[0]` for an empty automaton
    /// order: keyed by state
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").occ_counts() == [3, 1, 1, 1, 2]`
    pub fn occ_counts(self) -> List<Int> = todo

    @allow(unproven_list_index)
    /// A longest common substring of the automaton's string and `other`, as `(len, end)` with
    /// `other[end - len..end]` the substring (positions in `other`, from 0).
    ///
    /// cost: O(|other|)
    /// empty: `(0, 0)` for an empty `other`; also when nothing is shared
/// ties: the smallest `end`
    /// traps: when an element of `other` is not in `A`
    /// example: `SuffixAutomaton<Char>.of("abb").lcs("xbbx") == (2, 3)`
    pub fn lcs<S: Seq>(self, other: S) -> (len: Int, end: Int) = todo

    /// Moves the append point back to the root, so the next `push` starts a new word of a
    /// generalized automaton (the substrings of all words added so far).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m = SuffixAutomaton<Char>.of("ab"); m.reset(); m.push('c'); m.contains("c") and not m.contains("bc")`
    pub fn reset(mut self) = todo

    /// Adds `word` as a separate word of a generalized automaton: `reset()`, then `push` each
    /// letter. Afterwards the automaton holds the substrings of every word added.
    ///
    /// cost: O(|word|) amortized
    /// empty: an empty `word` only resets
    /// traps: when an element of `word` is not in `A`
    /// example: `var m = SuffixAutomaton<Char>(); m.add_word("ab"); m.add_word("b"); m.distinct() == 3`
    pub fn add_word<S: Seq>(mut self, word: S) = todo

    /// Every state in the order of `topo_order`, lazily.
    ///
    /// cost: O(states)
    /// order: non-decreasing `longest`; ties by smaller state number
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").states().collect() == [0, 1, 4, 2, 3]`
    pub fn states(self) -> yields Int = todo

    /// The suffix-link tree as child lists: entry `v` lists the states whose link is `v`.
    /// Rooted at `0`.
    ///
    /// cost: O(states)
    /// order: keyed by state; each child list in increasing state number
    /// traps: never
    /// example: `SuffixAutomaton<Char>.of("abb").endpos_tree() == [[1, 4], [], [], [], [2, 3]]`
    pub fn endpos_tree(self) -> List<List<Int>> = todo

    /// The `k`-th (0-based) non-empty substring in lexicographic order, as its letters, or
    /// `None` when there are not that many. With `distinct: false` each substring counts once
    /// per occurrence. A proper prefix comes before its extensions.
    ///
    /// cost: O(states * |A|)
    /// ties: with `distinct: false`, copies of one substring are adjacent and interchangeable
    /// order: the letters of the substring, first to last
    /// traps: when `k < 0`
    /// example: `SuffixAutomaton<Char>.of("abb").kth_substring(3) == Some(['b'])`
    /// example: `SuffixAutomaton<Char>.of("abb").kth_substring(5, distinct: false) == Some(['b', 'b'])`
    pub fn kth_substring(self, k: Int, distinct: Bool = true) -> ?List<A> = todo

/// The length of the longest sequence that is a substring of every word in `words`.
///
/// cost: O(sum |w| * |A|)
/// empty: `0` for no words; one word gives its length
/// traps: never
/// example: `lcs_all(["abcd", "xbcy", "bcz"]) == 2`
pub fn lcs_all<A, S: Seq>(words: List<S>) -> Int = todo

/// O(n * |A|) space, O(1) amortized push. Online palindromic tree (eertree) over alphabet domain `A`, with `diff` and `series_link`
/// for O(n log n) palindromic partition DP.
/// State `0` is the odd root (`pal_len == -1`); state `1` is the even root (`pal_len == 0`).
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

    /// An empty palindromic tree over the alphabet domain `A`: just the odd root `0` and the
    /// even root `1`. Call it as `PalindromicTree<A>()`.
    ///
    /// cost: O(|A|)
    /// traps: never
    /// example: `PalindromicTree<Char>().len == 2`
    pub fn new() -> PalindromicTree<A> = todo

    /// The palindromic tree of `s`: `PalindromicTree<A>()` followed by `push` of each element.
    /// New states are numbered from `2` in creation order.
    ///
    /// cost: O(|s| * |A|) memory, O(|s|) amortized time
    /// empty: just the two roots
    /// traps: when an element of `s` is not in `A`
    /// example: `PalindromicTree<Char>.of("aba").distinct == 3`
    pub fn of<S: Seq>(s: S) -> PalindromicTree<A> = todo

    @allow(unproven_list_index)
    /// Appends the letter `a` and returns the state of the longest palindromic suffix of the
    /// new string, creating it if it is new.
    ///
    /// cost: O(1) amortized
    /// traps: never (`a` is an `A`)
    /// example: `var t = PalindromicTree<Char>(); t.push('a'); t.push('a') == 3`
    pub fn push(mut self, a: A) -> Int = todo

    /// The number of states, both roots included: `distinct + 2`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aba").len == 5`
    @get pub fn len(self) -> Int = todo

    /// The number of distinct non-empty palindromic substrings, `len - 2`; at most `n`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aba").distinct == 3`
    @get pub fn distinct(self) -> Int = todo

    @allow(unproven_list_index)
    /// The length of the palindrome of `state`: `-1` at the odd root, `0` at the even root.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let t = PalindromicTree<Char>.of("aba"); t.pal_len(4) == 3 and t.pal_len(0) == -1`
    pub fn pal_len(self, state: Int) -> Int = todo

    @allow(unproven_list_index)
    /// The state of the longest proper palindromic suffix of `state`'s palindrome. A one-letter
    /// palindrome links to the even root `1`; both roots link to the odd root `0`.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let t = PalindromicTree<Char>.of("aba"); t.link(4) == 2 and t.link(2) == 1 and t.link(1) == 0`
    pub fn link(self, state: Int) -> Int = todo

    @allow(unproven_list_index)
    /// The state of `a + P + a` where `P` is `state`'s palindrome, or `None` when that is not in
    /// the tree. From the odd root it gives the one-letter palindrome `a`.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let t = PalindromicTree<Char>.of("aba"); t.step(3, 'a') == Some(4) and t.step(1, 'a') == None`
    pub fn step(self, state: Int, a: A) -> ?Int = todo

    @allow(unproven_list_index)
    /// The difference `pal_len(state) - pal_len(link(state))`, the period that groups
    /// palindromic suffixes into series; `0` at both roots.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `PalindromicTree<Char>.of("aba").diff(4) == 2`
    pub fn diff(self, state: Int) -> Int = todo

    @allow(unproven_list_index)
    /// The nearest state up the suffix links whose `diff` differs from `state`'s: `link(state)`
    /// when the diffs differ, else `series_link(link(state))`. Following it from any state
    /// reaches a root in O(log n) steps. Both roots give `0`.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `PalindromicTree<Char>.of("aba").series_link(4) == 2`
    /// example: `PalindromicTree<Char>.of("aaaa").series_link(5) == 1`
    pub fn series_link(self, state: Int) -> Int = todo

    @allow(unproven_list_index)
    /// For each state, how many times its palindrome occurs in the pushed string, overlaps
    /// included; `0` at both roots. The entries from `2` sum to `manacher(s).count()`.
    ///
    /// cost: O(states)
    /// order: keyed by state
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aba").occ_counts() == [0, 0, 2, 1, 1]`
    pub fn occ_counts(self) -> List<Int> = todo

    /// Prepends the letter `a` (a double-ended tree) and returns the state of the longest
    /// palindromic prefix of the new string, creating it if it is new.
    ///
    /// cost: O(1) amortized
    /// traps: never (`a` is an `A`)
    /// example: `var t = PalindromicTree<Char>.of("ba"); t.push_front('a') == 4`
    pub fn push_front(mut self, a: A) -> Int = todo

    /// Undoes the most recent `push` not yet undone, removing the state it created if any, so a
    /// DFS over a tree of strings can back out.
    ///
    /// cost: O(1) amortized
    /// traps: when there is no `push` to undo
    /// example: `var t = PalindromicTree<Char>.of("ab"); t.pop(); t.distinct == 1 and t.last_state == 2`
    pub fn pop(mut self) = todo

    /// The state of the longest palindromic suffix of the current string; the even root `1`
    /// when the string is empty.
    ///
    /// cost: O(1)
    /// ties: none; the longest suffix is unique
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aba").last_state == 4`
    @get pub fn last_state(self) -> Int = todo

    /// True when `word` is a palindrome occurring in the pushed string, that is, a state of the
    /// tree.
    ///
    /// cost: O(|word|)
    /// empty: `true` (the even root)
    /// traps: never
    /// example: `let t = PalindromicTree<Char>.of("aba"); t.contains("aba") and not t.contains("ab")`
    pub fn contains<S: Seq>(self, word: S) -> Bool = todo

    /// `dp[i]` is the least number of palindromes the first `i` pushed letters split into, for
    /// `i in 0..=n`, by the series-link DP.
    ///
    /// cost: O(n log n)
    /// ties: none; only the count is returned
    /// order: keyed by prefix length `0..=n`
    /// traps: never
    /// example: `PalindromicTree<Char>.of("aab").min_palindrome_partition() == [0, 1, 1, 2]`
    pub fn min_palindrome_partition(self) -> List<Int> = todo

/// Dynamic trie over alphabet domain `A`. O(|word|) per operation; flat `|A|`-stride table.
pub struct Trie<A>:
    alpha: 0..=2_000_000_000
    next: List<-1..=2_000_000_000>
    ends: List<0..=2_000_000_000>
    pass: List<0..=2_000_000_000>

    /// An empty trie over the alphabet domain `A`: one root node `0`, no words. Call it as
    /// `Trie<A>()`. Nodes are numbered in creation order.
    ///
    /// cost: O(|A|)
    /// traps: never
    /// example: `Trie<Char>().len == 1`
    pub fn new() -> Trie<A> = todo

    /// The root node, for the empty word: always `0`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Trie<Char>().root == 0`
    @get pub fn root(self) -> Int = todo

    /// The number of nodes, the root included. `remove` keeps nodes, so this never shrinks
    /// except by `clear`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.add("ac"); t.len == 4`
    @get pub fn len(self) -> Int = todo

    /// True when no words are stored (every count is zero), whatever nodes remain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("a"); t.remove("a"); t.is_empty`
    @get pub fn is_empty(self) -> Bool = todo

    @allow(unproven_list_index)
    /// The child of `node` along `a`, or `None` when it was never created.
    ///
    /// cost: O(1)
    /// traps: when `node` is outside `0..self.len`
    /// example: `var t = Trie<Char>(); t.add("ab"); t.step(0, 'a') == Some(1) and t.step(1, 'z') == None`
    pub fn step(self, node: Int, a: A) -> ?Int = todo

    @allow(unproven_list_index)
    /// The node of `word`, or `None` when no stored word starts with `word`
    /// (`count_prefix(word) == 0`).
    ///
    /// cost: O(|word|)
    /// empty: `Some(0)` unless the trie is empty
    /// ties: none; a word has one node
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.find("a") == Some(1) and t.find("b") == None`
    pub fn find<S: Seq>(self, word: S) -> ?Int = todo

    @allow(unproven_list_index)
    /// Inserts one copy of `word` and returns its terminal node; `count(word)` and
    /// `count_prefix(p)` for every prefix `p` of `word` grow by one.
    ///
    /// cost: O(|word|)
    /// empty: an empty `word` is stored at the root
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab") == 2`
    pub fn add<S: Seq>(mut self, word: S) -> Int = todo

    @allow(unproven_list_index)
    /// Removes one copy of `word`, reversing one `add`, and returns `true`; returns `false` and
    /// changes nothing when `count(word) == 0`.
    ///
    /// cost: O(|word|)
    /// empty: removes a copy of the empty word, if stored
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.remove("ab") and not t.remove("ab")`
    pub fn remove<S: Seq>(mut self, word: S) -> Bool = todo

    @allow(unproven_list_index)
    /// The number of copies of exactly `word` stored.
    ///
    /// cost: O(|word|)
    /// empty: copies of the empty word
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.add("ab"); t.count("ab") == 2 and t.count("a") == 0`
    pub fn count<S: Seq>(self, word: S) -> Int = todo

    @allow(unproven_list_index)
    /// The number of stored words (with multiplicity) that start with `prefix`.
    ///
    /// cost: O(|prefix|)
    /// empty: every stored word
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.add("ac"); t.count_prefix("a") == 2`
    pub fn count_prefix<S: Seq>(self, prefix: S) -> Int = todo

    @allow(unproven_list_index)
    /// The number of stored words that end at `node`.
    ///
    /// cost: O(1)
    /// traps: when `node` is outside `0..self.len`
    /// example: `var t = Trie<Char>(); let v = t.add("ab"); t.ends_at(v) == 1`
    pub fn ends_at(self, node: Int) -> Int = todo

    @allow(unproven_list_index)
    /// The number of stored words that pass through or end at `node`; at the root, all of them.
    ///
    /// cost: O(1)
    /// traps: when `node` is outside `0..self.len`
    /// example: `var t = Trie<Char>(); t.add("ab"); t.add("ac"); t.pass_at(1) == 2`
    pub fn pass_at(self, node: Int) -> Int = todo

    /// True when at least one copy of exactly `word` is stored: `count(word) > 0`.
    ///
    /// cost: O(|word|)
    /// empty: whether the empty word is stored
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.contains("ab") and not t.contains("a")`
    pub fn contains<S: Seq>(self, word: S) -> Bool = todo

    /// Removes every word and node, leaving just the root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("ab"); t.clear(); t.is_empty and t.len == 1`
    pub fn clear(mut self) = todo

    /// The `k`-th (0-based) stored word in lexicographic order, copies counted, as its letters;
    /// `None` when fewer than `k + 1` words are stored. A word comes before its extensions.
    ///
    /// cost: O(depth * |A|)
    /// ties: copies of one word are adjacent and interchangeable
    /// order: the letters of the word, first to last
    /// traps: when `k < 0`
    /// example: `var t = Trie<Char>(); t.add("b"); t.add("ab"); t.add("a"); t.kth(1) == Some(['a', 'b']) and t.kth(3) == None`
    pub fn kth(self, k: Int) -> ?List<A> = todo

    /// The number of stored words (with multiplicity) lexicographically smaller than `word`.
    ///
    /// cost: O(|word| * |A|)
    /// empty: `0` for an empty `word`
    /// traps: never
    /// example: `var t = Trie<Char>(); t.add("b"); t.add("ab"); t.add("a"); t.count_less("b") == 2`
    pub fn count_less<S: Seq>(self, word: S) -> Int = todo

/// O(total * |A|) build, O(1) step. Aho-Corasick multi-pattern automaton over alphabet domain `A`.
/// Built in O(sum |p| * |A|) flat memory and time; `step`, `link`, and `hit` are O(1).
pub struct AhoCorasick<A>:
    alpha: 0..=2_000_000_000
    goto: List<-1..=2_000_000_000>
    fail: List<0..=2_000_000_000>
    ends: List<List<0..=2_000_000_000>>
    exit: List<-1..=2_000_000_000>
    order: List<0..=2_000_000_000>
    lens: List<0..=2_000_000_000>

    @allow(unproven_list_index)
    /// The Aho-Corasick automaton of `patterns`; call it as `AhoCorasick<A>(patterns)`. Pattern
    /// `p` is `patterns[p]`. States are the trie nodes of the patterns, numbered in creation
    /// order with the root `0`; equal patterns share a state.
    ///
    /// cost: O(sum |p| * |A|)
    /// empty: no patterns gives just the root; an empty pattern ends at the root and matches at every `at` in `0..=|text|`
    /// traps: when an element of a pattern is not in `A`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).len == 4`
    pub fn new<S: Seq>(patterns: List<S>) -> AhoCorasick<A> = todo

    /// The root state, before any letter is read: always `0`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `AhoCorasick<Char>(["a"]).root == 0`
    @get pub fn root(self) -> Int = todo

    /// The number of states, the root included.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).len == 4`
    @get pub fn len(self) -> Int = todo

    @allow(unproven_list_index)
    /// The state after reading `a` in `state`: the longest pattern prefix that is a suffix of
    /// what has been read. Total, so it never fails.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let ac = AhoCorasick<Char>(["a", "ab", "b"]); ac.step(2, 'b') == 3 and ac.step(1, 'a') == 1`
    pub fn step(self, state: Int, a: A) -> Int = todo

    @allow(unproven_list_index)
    /// The failure link of `state`: the state of its longest proper suffix that is a pattern
    /// prefix; `0` at the root.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let ac = AhoCorasick<Char>(["a", "ab", "b"]); ac.link(2) == 3 and ac.link(0) == 0`
    pub fn link(self, state: Int) -> Int = todo

    @allow(unproven_list_index)
    /// True when some pattern ends at `state` itself or at a state up its failure links:
    /// `words_at(state)` is non-empty.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let ac = AhoCorasick<Char>(["ab", "b"]); ac.hit(2) and not ac.hit(1)`
    pub fn hit(self, state: Int) -> Bool = todo

    @allow(unproven_list_index)
    /// Every pattern that ends at `state` or up its failure links, by index.
    ///
    /// cost: O(output)
    /// order: longest pattern first; equal patterns by increasing index; a new `List`, keyed from 0
    /// traps: when `state` is outside `0..self.len`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).words_at(2) == [1, 2]`
    pub fn words_at(self, state: Int) -> List<Int> = todo

    @allow(unproven_list_index)
    /// Every occurrence of every pattern in `text`, as `(at, pattern)` with `at` the 0-based
    /// start: exactly the pairs with `text[at..at + patterns[pattern].len] == patterns[pattern]`.
    ///
    /// cost: O(|text| + output)
    /// empty: an empty `text` yields only matches of an empty pattern
    /// ties: at one end position, longest pattern first, then increasing index (`words_at`)
    /// order: by end position
    /// traps: when an element of `text` is not in `A`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).find_all("ab").collect() == [(0, 0), (0, 1), (1, 2)]`
    pub fn find_all<S: Seq>(self, text: S) -> yields (at: Int, pattern: Int) = todo

    @allow(unproven_list_index)
    /// For each pattern, how many times it occurs in `text`, overlaps included: the number of
    /// `find_all` pairs with that index.
    ///
    /// cost: O(|text| + states)
    /// empty: all zeros for an empty `text` (an empty pattern still counts once)
    /// order: keyed by pattern index
    /// traps: when an element of `text` is not in `A`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).counts("abb") == [1, 1, 2]`
    pub fn counts<S: Seq>(self, text: S) -> List<Int> = todo

    /// Every state in BFS order of the trie (root first, then by depth, equal depths by state
    /// number), so each state comes after its failure link; iterate backwards for bottom-up DP.
    ///
    /// cost: O(states)
    /// order: non-decreasing depth; ties by smaller state number
    /// traps: never
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).fail_order() == [0, 1, 3, 2]`
    pub fn fail_order(self) -> List<Int> = todo

    /// The nearest state strictly up the failure links of `state` at which a pattern ends, or
    /// `None` when there is none.
    ///
    /// cost: O(1)
    /// traps: when `state` is outside `0..self.len`
    /// example: `let ac = AhoCorasick<Char>(["a", "ab", "b"]); ac.exit_link(2) == Some(3) and ac.exit_link(1) == None`
    pub fn exit_link(self, state: Int) -> ?Int = todo

    /// The state where `patterns[pattern]` ends.
    ///
    /// cost: O(1)
    /// traps: when `pattern` is outside `0..patterns.len`
    /// example: `AhoCorasick<Char>(["a", "ab", "b"]).pattern_state(1) == 2`
    pub fn pattern_state(self, pattern: Int) -> Int = todo

/// O(|pat| * |A| * log n) amortized insert/delete, O(|text| * log n) query.
/// Online multi-pattern Aho-Corasick via log-decomposed static buckets.
pub struct DynamicAhoCorasick<A>:
    alpha: 0..=2_000_000_000

    /// An empty online pattern set over the alphabet domain `A`; call it as
    /// `DynamicAhoCorasick<A>()`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `DynamicAhoCorasick<Char>().is_empty`
    pub fn new() -> DynamicAhoCorasick<A> = todo

    /// The number of patterns stored, copies counted.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("a"); d.add("a"); d.len == 2`
    @get pub fn len(self) -> Int = todo

    /// True when no patterns are stored.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("a"); d.remove("a"); d.is_empty`
    @get pub fn is_empty(self) -> Bool = todo

    /// Stores one more copy of `pattern`.
    ///
    /// cost: O(|pattern| * |A| * log n) amortized
    /// empty: an empty pattern matches at every `at` in `0..=|text|`
    /// traps: when an element of `pattern` is not in `A`
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("ab"); d.len == 1`
    pub fn add<S: Seq>(mut self, pattern: S) = todo

    /// Removes one stored copy of `pattern`.
    ///
    /// cost: O(|pattern| * |A| * log n) amortized
    /// empty: removes a copy of the empty pattern
    /// traps: when no copy of `pattern` is stored
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("ab"); d.remove("ab"); d.count_matches("ab") == 0`
    pub fn remove<S: Seq>(mut self, pattern: S) = todo

    /// The total number of occurrences of all stored patterns in `text`, overlaps and copies
    /// counted.
    ///
    /// cost: O(|text| * log n)
    /// empty: `0` for an empty `text` unless an empty pattern is stored
    /// traps: when an element of `text` is not in `A`
    /// example: `var d = DynamicAhoCorasick<Char>(); d.add("ab"); d.add("b"); d.count_matches("abb") == 3`
    pub fn count_matches<S: Seq>(self, text: S) -> Int = todo

// ---- tests

fn random_word(n: Int, letters: Int) -> List<Char> = todo

fn common_prefix<T>(s: List<T>, a: Int, b: Int) -> Int = todo

fn suffix_less<T>(s: List<T>, a: Int, b: Int) -> Bool = todo

@test
fn z_and_prefix_function_match_their_definitions() = todo

@test
fn find_all_matches_a_scan() = todo

@test
fn suffix_array_and_lcp_match_sorting_the_suffixes() = todo

@test
fn rolling_hash_over_str_list_and_domain_arrays() = todo

@test
fn repeat_writes_a_string_over() = todo

@test
fn starts_ends_and_find_on_text() = todo

@test
fn lower_and_upper_change_only_letters() = todo

@test
fn digit_is_the_value_of_a_decimal_character() = todo

@test
fn count_counts_one_character() = todo

@test
fn rfind_gives_the_last_place() = todo

@test
fn borders_prefix_matches_and_overlap_match_brute() = todo

@test
fn lyndon_factorization_and_min_rotation_match_definitions() = todo

@test
fn manacher_and_is_palindrome_match_brute() = todo

@test
fn suffix_array_struct_queries_match_brute() = todo

@test
fn rolling_hash_lcp_lcs_and_hash2d_match_brute() = todo

@test
fn suffix_automaton_matches_brute_and_linear_occurrences() = todo

@test
fn palindromic_tree_distinct_occurrences_and_series_dp() = todo

@test
fn trie_add_remove_and_prefix_counts() = todo

@test
fn aho_corasick_matches_counts_and_avoidance_dp() = todo

@test
fn indexed_views_and_common_prefix_suffix() = todo

@test
fn lyndon_runs_matches_brute() = todo

@test
fn sequence_generality_parity_across_list_array_and_views() = todo
