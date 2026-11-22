//! A trie over an alphabet given as a domain: insert words, count words by prefix, and
//! walk it node by node (for xor-maximization, Aho-Corasick style scans, and the like).

/// A trie whose edges are labelled by `A`. Memory is one `[A]?Int` per node; `add` and
/// the queries are O(word length).
pub struct Trie<A>:
    next: List<[A]?Int>
    /// How many added words end at this node.
    ends: List<Int>
    /// How many added words pass through or end at this node.
    pass: List<Int>

    pub fn new() -> Trie<A>:
        let none_yet: [A]?Int = none
        Trie([none_yet], [0], [0])

    /// The root: the node of the empty word.
    pub @get fn root(self) -> Int = 0

    /// The node reached from `node` along the edge `a`, if a word goes that way.
    pub fn step(self, node: Int, a: A) -> ?Int = next[node][a]

    /// The node of `word`, or `none` if no added word starts with it.
    pub fn find(self, word: List<A>) -> ?Int:
        var at = 0
        for a in word:
            if next[at][a] == none: return none
            at = next[at][a]!
        at

    /// Add one copy of `word`; returns its node.
    pub fn add(mut self, word: List<A>) -> Int:
        var at = 0
        pass[0] += 1
        for a in word:
            if next[at][a] == none:
                let fresh: [A]?Int = none
                next.push(fresh)
                ends.push(0)
                pass.push(0)
                next[at][a] = next.len - 1
            at = next[at][a]!
            pass[at] += 1
        ends[at] += 1
        at

    /// Remove one copy of `word`. `false` (and no change) if it is not there.
    pub fn remove(mut self, word: List<A>) -> Bool:
        let node = self.find(word)
        if node == none or ends[node!] == 0: return false
        var at = 0
        pass[0] -= 1
        for a in word:
            at = next[at][a]!
            pass[at] -= 1
        ends[at] -= 1
        true

    /// How many copies of `word` are in the trie.
    pub fn count(self, word: List<A>) -> Int:
        let node = self.find(word)
        if node == none: 0 else: ends[node!]

    /// How many added words start with `prefix`.
    pub fn count_prefix(self, prefix: List<A>) -> Int:
        let node = self.find(prefix)
        if node == none: 0 else: pass[node!]

    /// The number of nodes, root included.
    pub @get fn len(self) -> Int = next.len

@test
fn counts_match_a_list_of_words():
    domain A = 0..3
    for _ in 0..25:
        var t = Trie<A>()
        var words: List<List<A>> = []
        for _ in 0..30:
            let w = [A(rand(0, 2)) for _ in 0..rand(0, 4)]
            if rand(0, 3) > 0:
                t.add(w)
                words.push(w)
            else:
                let i = words.find(w)
                assert t.remove(w) == (i != none)
                if i != none: words = [words[j] for j in 0..words.len if j != i!]
            let q = [A(rand(0, 2)) for _ in 0..rand(0, 3)]
            assert t.count(q) == words.count(w => w == q)
            assert t.count_prefix(q) == words.count(
                w => w.len >= q.len and (0..q.len).all(i => w[i] == q[i])
            )

@test
fn largest_xor_with_a_set():
    // numbers as 6 bits, high bit first; walk towards the opposite bit when possible
    domain Bit = 0..2
    let bits = x => [Bit((x >> (5 - i)) % 2) for i in 0..6]
    let xs = [3, 10, 5, 25, 2, 8]
    var t = Trie<Bit>()
    for x in xs: t.add(bits(x))
    for q in 0..64:
        var (at, got) = (t.root, 0)
        for b in bits(q):
            let want = Bit(1 - b.to_idx)
            if let n = t.step(at, want):
                at = n
                got = 2 * got + 1
            else:
                at = t.step(at, b)!
                got = 2 * got
        assert got == xs.map(x => q ^ x).max()
