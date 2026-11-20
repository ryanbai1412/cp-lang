//! The suffix automaton of a string: the smallest machine that accepts exactly its
//! substrings. Built online in O(n * |A|) memory and O(n) amortized time.

/// The automaton of the letters pushed so far. State 0 is the empty string; every state
/// stands for a set of substrings that end at the same positions.
pub struct SuffixAutomaton<A>:
    next: List<[A]?Int>
    /// The suffix link: the state of the longest suffix that ends at more positions.
    link: List<?Int>
    /// The length of the longest substring of the state.
    longest: List<Int>
    /// Is the state a prefix of the string (not a clone)? Used to count occurrences.
    prefix: List<Bool>
    last: Int

    pub fn new() -> SuffixAutomaton<A>:
        let empty: [A]?Int = none
        SuffixAutomaton([empty], [none], [0], [false], 0)

    /// The automaton of `s`.
    pub fn of(s: List<A>) -> SuffixAutomaton<A>:
        var sam = SuffixAutomaton<A>.new()
        for a in s: sam.push(a)
        sam

    /// Append one letter to the string.
    pub fn push(mut self, a: A):
        let empty: [A]?Int = none
        let cur = self.state(empty, longest[last] + 1, true)
        var p: ?Int = last
        while p != none and next[p!][a] == none:
            next[p!][a] = cur
            p = link[p!]
        if p == none:
            link[cur] = 0
        else:
            let q = next[p!][a]!
            if longest[q] == longest[p!] + 1:
                link[cur] = q
            else:
                // split q: the clone keeps the short substrings, q keeps the long ones
                let clone = self.state([A] b => next[q][b], longest[p!] + 1, false)
                link[clone] = link[q]
                link[q] = clone
                link[cur] = clone
                while p != none and next[p!][a] == q:
                    next[p!][a] = clone
                    p = link[p!]
        last = cur

    /// The state of `word`, or `none` if it is not a substring.
    pub fn find(self, word: List<A>) -> ?Int:
        var at = 0
        for a in word:
            if next[at][a] == none: return none
            at = next[at][a]!
        at

    pub fn contains(self, word: List<A>) -> Bool = self.find(word) != none

    /// The number of distinct non-empty substrings.
    pub @get fn distinct(self) -> Int:
        var total = 0
        for v in 1..next.len: total += longest[v] - longest[link[v]!]
        total

    /// For every state, at how many positions its substrings end. O(n log n).
    pub fn occurrences(self) -> List<Int>:
        var occ = [Int(p) for p in prefix]
        var order = [v for v in 1..next.len]
        sort(mut order, by: v => -longest[v])
        for v in order: occ[link[v]!] += occ[v]
        occ

    /// The number of states.
    pub @get fn len(self) -> Int = next.len

    fn state(mut self, edges: [A]?Int, length: Int, is_prefix: Bool) -> Int:
        next.push(edges)
        link.push(none)
        longest.push(length)
        prefix.push(is_prefix)
        next.len - 1

@test
fn substrings_and_their_counts_match_brute_force():
    domain A = 0..2
    for _ in 0..40:
        let n = rand(0, 9)
        let s = [A(rand(0, 1)) for _ in 0..n]
        let sam = SuffixAutomaton<A>.of(s)
        let occ = sam.occurrences()
        var seen: List<List<A>> = []
        for len in 1..=4:
            for code in 0..(1 << len):
                let w = [A((code >> i) % 2) for i in 0..len]
                let times = count(all(s[i + j] == w[j] for j in 0..len) for i in 0..=n - len)
                assert sam.contains(w) == (times > 0)
                if let v = sam.find(w): assert occ[v] == times
        var distinct = 0
        for len in 1..=n:
            var words: List<List<A>> = []
            for i in 0..=n - len:
                let w = [s[i + j] for j in 0..len]
                if words.find(w) == none: words.push(w)
            distinct += words.len
        assert sam.distinct == distinct
        assert sam.len <= max(2 * n, 1)

@test
fn longest_common_substring():
    domain Letter = 'a'..='z'
    let text = w => [Letter(c) for c in w]
    let sam = SuffixAutomaton<Letter>.of(text("xabcabd"))
    var (at, run, best) = (0, 0, 0)
    for c in text("zzbcabz"):
        while at != 0 and sam.next[at][c] == none:
            at = sam.link[at]!
            run = sam.longest[at]
        if let to = sam.next[at][c]:
            at = to
            run += 1
        best max= run
    assert best == 4  // "bcab"
