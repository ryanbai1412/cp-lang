//! Aho-Corasick: one automaton for a whole dictionary, so a text is scanned for every
//! word at once in O(text + matches) after an O(total word length * |A|) build.

/// The automaton of a fixed list of words over the alphabet `A`. A state is the longest
/// suffix of the text read so far that is a prefix of some word; state 0 is the empty
/// prefix. Words are named by their position in the list passed to `new`, and equal
/// words keep separate numbers.
pub struct AhoCorasick<A>:
    /// `goto[s][a]`: the state after reading `a` in state `s`. Total: there is no failing.
    goto: List<[A]Int>
    /// The suffix link: the longest proper suffix of the state that is also a state.
    fail: List<Int>
    /// The words that end exactly at this state.
    ends: List<List<Int>>
    /// The nearest state along the suffix links where some word ends.
    exit: List<?Int>
    /// Every state, parents and suffix links before the state itself.
    order: List<Int>
    lens: List<Int>

    pub fn new(words: List<List<A>>) -> AhoCorasick<A>:
        let blank: [A]?Int = none
        var kid = [blank]
        var ends: List<List<Int>> = [[]]
        for i in 0..words.len:
            var at = 0
            for a in words[i]:
                if kid[at][a] == none:
                    let fresh: [A]?Int = none
                    kid.push(fresh)
                    ends.push([])
                    kid[at][a] = kid.len - 1
                at = kid[at][a]!
            ends[at].push(i)
        let n = kid.len
        var goto = [[A] a => kid[s][a] ?? 0 for s in 0..n]
        var fail = List.filled(n, 0)
        var exit: List<?Int> = [none for _ in 0..n]
        var order = [0]
        var head = 0
        while head < order.len:
            let s = order[head]
            head += 1
            for a in A:
                let past = if s == 0: 0 else: goto[fail[s]][a]
                if let t = kid[s][a]:
                    fail[t] = past
                    exit[t] = if ends[past].is_empty: exit[past] else: past
                    order.push(t)
                else:
                    goto[s][a] = past
        AhoCorasick(goto, fail, ends, exit, order, [w.len for w in words])

    /// The state before any letter is read.
    pub @get fn root(self) -> Int = 0

    /// The number of states.
    pub @get fn len(self) -> Int = goto.len

    /// The state after reading `a` in `state`. O(1).
    pub fn step(self, state: Int, a: A) -> Int = goto[state][a]

    /// The suffix link of `state` (the root links to itself).
    pub fn link(self, state: Int) -> Int = fail[state]

    /// Does some word end at the text position that put the automaton in `state`? O(1).
    /// This is the test a "texts that avoid every word" DP needs.
    pub fn hit(self, state: Int) -> Bool = not ends[state].is_empty or exit[state] != none

    /// Every word that ends where `state` was reached, longest first. O(answer).
    pub fn words_at(self, state: Int) -> List<Int>:
        var found: List<Int> = []
        var at: ?Int = state
        while at != none:
            for w in ends[at!]: found.push(w)
            at = exit[at!]
        found

    /// Every occurrence in `text` as `(at, word)`: word number `word` fills
    /// `text[at..at + its length]`. Ordered by where the occurrence ends.
    pub fn matches(self, text: List<A>) -> List<(at: Int, word: Int)>:
        var found: List<(at: Int, word: Int)> = []
        var state = 0
        for i in 0..text.len:
            state = goto[state][text[i]]
            for w in self.words_at(state): found.push((at: i + 1 - lens[w], word: w))
        found

    /// How many times each word occurs in `text`, overlaps included. O(text + states),
    /// however many matches there are.
    pub fn count_each(self, text: List<A>) -> List<Int>:
        var visits = List.filled(goto.len, 0)
        var state = 0
        for a in text:
            state = goto[state][a]
            visits[state] += 1
        // a visit to a state is also a visit to every suffix of it
        for i in (1..order.len).rev: visits[fail[order[i]]] += visits[order[i]]
        var total = List.filled(lens.len, 0)
        for s in 0..goto.len:
            for w in ends[s]: total[w] = visits[s]
        total

@test
fn matches_and_counts_agree_with_checking_every_position():
    domain A = 0..2
    for _ in 0..40:
        let words = [[A(rand(0, 1)) for _ in 0..rand(1, 4)] for _ in 0..rand(1, 5)]
        let text = [A(rand(0, 1)) for _ in 0..rand(0, 14)]
        let ac = AhoCorasick<A>(words)
        var want: List<(Int, Int)> = []
        for i in 0..text.len:
            for w in 0..words.len:
                let k = words[w].len
                if i + k <= text.len and all(
                    text[i + j] == words[w][j] for j in 0..k
                ): want.push((i, w))
        let got = [(m.at, m.word) for m in ac.matches(text)]
        assert got.sorted() == want.sorted()
        assert ac.count_each(text) == [count(p.1 == w for p in want) for w in 0..words.len]
        assert ac.len <= 1 + sum(w.len for w in words)

@test
fn longest_match_first_and_links():
    domain L = 'a'..='z'
    let word = w => [L(c) for c in w]
    let ac = AhoCorasick<L>([word("he"), word("she"), word("his"), word("hers"), word("e")])
    var state = ac.root
    for c in word("ushe"): state = ac.step(state, c)
    assert ac.words_at(state) == [1, 0, 4]  // "she", then "he", then "e"
    assert ac.link(state) == ac.step(ac.step(ac.root, L('h')), L('e'))
    assert [(m.at, m.word) for m in ac.matches(word("ushers"))] == [(1, 1), (2, 0), (3, 4), (2, 3)]
    assert ac.count_each(word("eehee")) == [1, 0, 0, 0, 4]

@test
fn count_texts_that_avoid_every_word():
    // dp over (length, state): never enter a state where a word ends
    domain A = 0..2
    for _ in 0..15:
        let words = [[A(rand(0, 1)) for _ in 0..rand(1, 3)] for _ in 0..rand(1, 3)]
        let ac = AhoCorasick<A>(words)
        let n = 8
        var ways = List.filled(ac.len, 0)
        ways[ac.root] = 1
        for _ in 0..n:
            var after = List.filled(ac.len, 0)
            for s in 0..ac.len:
                for a in A:
                    let t = ac.step(s, a)
                    if not ac.hit(t): after[t] += ways[s]
            ways = after
        var brute = 0
        for code in 0..(1 << n):
            let text = [A((code >> i) % 2) for i in 0..n]
            if ac.matches(text).is_empty: brute += 1
        assert ways.sum() == brute
