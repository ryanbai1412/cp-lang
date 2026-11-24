// n: 512
// fixed: A
use std.strings.{Trie, AhoCorasick, DynamicAhoCorasick}
domain L = 0..4
let n: Int = read()
let m = n / 8
let words = [[L((i * 7919 + i / 3 + j * 5) % 3) for i in 0..m] for j in 0..32]
let text = [L((i * 7919 + i / 3) % 3) for i in 0..n]
let runs = [[L(0) for _ in 0..k] for k in 1..=m]
let shorts = [[L(j / 3 ** i % 3) for i in 0..8] for j in 0..n / 8]
var acc = 0
for r in 0..128:
    var d = DynamicAhoCorasick<L>()
    for w in shorts:
        d.add(w) // timed: strings.DynamicAhoCorasick.add [fixed: pattern]
    for _ in 0..32:
        acc += d.count_matches(text) // timed: strings.DynamicAhoCorasick.count_matches
    for w in shorts:
        d.remove(w) // timed: strings.DynamicAhoCorasick.remove [fixed: pattern]
for r in 0..512:
    var t = Trie<L>()
    for w in words:
        acc += t.add(w) // timed: strings.Trie.add
    for w in words:
        if t.contains(w): acc += 1 // timed: strings.Trie.contains
    for w in words:
        acc += t.count(w) // timed: strings.Trie.count
    for w in words:
        acc += t.count_prefix(w) // timed: strings.Trie.count_prefix
    for w in words:
        if let v = t.find(w): acc += v // timed: strings.Trie.find
    for w in words:
        acc += t.count_less(w) // timed: strings.Trie.count_less
    for k in 0..32:
        if let w = t.kth(k): acc += w.len // timed: strings.Trie.kth
    for w in words:
        if t.remove(w): acc += 1 // timed: strings.Trie.remove
    t.clear()
    let ac = AhoCorasick<L>(words) // timed: strings.AhoCorasick.new
    for _ in 0..4:
        acc += ac.counts(text).len // timed: strings.AhoCorasick.counts
    for h in ac.find_all(text): acc += h.at // timed: strings.AhoCorasick.find_all
    for _ in 0..16:
        acc += ac.fail_order().len // timed: strings.AhoCorasick.fail_order
let deep = AhoCorasick<L>(runs)
let top = deep.pattern_state(m - 1)
for r in 0..8192:
    acc += deep.words_at(top).len // timed: strings.AhoCorasick.words_at
var t = Trie<L>()
for w in words: let _ = t.add(w)
let ac = AhoCorasick<L>(words)
let nodes = t.len
var t2 = Trie<L>()
var d0 = DynamicAhoCorasick<L>()
d0.add(words[0])
let states = ac.len
for i in 0..1 << 18:
    let v = i * 7 % nodes
    let u = i * 13 % states
    let c = L(i % 3)
    acc += Trie<L>().len // timed: strings.Trie.new
    t2.clear() // timed: strings.Trie.clear
    acc += DynamicAhoCorasick<L>().len // timed: strings.DynamicAhoCorasick.new
    acc += d0.len // timed: strings.DynamicAhoCorasick.len
    if d0.is_empty: acc += 1 // timed: strings.DynamicAhoCorasick.is_empty
    acc += t.root // timed: strings.Trie.root
    acc += t.len // timed: strings.Trie.len
    if t.is_empty: acc += 1 // timed: strings.Trie.is_empty
    if let x = t.step(v, c): acc += x // timed: strings.Trie.step
    acc += t.ends_at(v) // timed: strings.Trie.ends_at
    acc += t.pass_at(v) // timed: strings.Trie.pass_at
    acc += ac.root // timed: strings.AhoCorasick.root
    acc += ac.len // timed: strings.AhoCorasick.len
    acc += ac.step(u, c) // timed: strings.AhoCorasick.step
    acc += ac.link(u) // timed: strings.AhoCorasick.link
    if ac.hit(u): acc += 1 // timed: strings.AhoCorasick.hit
    if let x = ac.exit_link(u): acc += x // timed: strings.AhoCorasick.exit_link
    acc += ac.pattern_state(i % 32) // timed: strings.AhoCorasick.pattern_state
print(acc)
