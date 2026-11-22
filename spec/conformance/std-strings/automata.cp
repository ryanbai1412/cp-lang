// expect: ok, output "YES YES YES YES\nYES YES YES\nYES YES YES YES\nYES YES YES YES\nYES YES YES\nYES YES YES YES"
use std.strings.{SuffixAutomaton, PalindromicTree, Trie, AhoCorasick}
domain L = 'a'..='c'
fn w(s: Str) -> List<L> = [L(c) for c in s]
let m = SuffixAutomaton<L>.of(w("abb"))
print(yes(m.root == 0), yes(m.len == 5), yes(m.distinct() == 5), yes(m.occ_counts() == [3, 1, 1, 1, 2]))
print(yes([v for v in m.states()] == [0, 1, 4, 2, 3]), yes(m.endpos_tree() == [[1, 4], [], [], [], [2, 3]]), yes(m.contains(w("bb"))))
var g = SuffixAutomaton<L>.of(w("ab"))
g.reset()
g.push(L('c'))
var h = SuffixAutomaton<L>.new()
h.add_word(w("ab"))
h.add_word(w("b"))
let p = PalindromicTree<L>.of(w("aba"))
print(yes(g.contains(w("c")) and not g.contains(w("bc"))), yes(h.distinct() == 3), yes(p.len == 5 and p.distinct == 3 and p.last_state == 4), yes(p.occ_counts() == [0, 0, 2, 1, 1]))
print(yes(p.contains(w("aba")) and p.contains(w("b")) and not p.contains(w("ab"))), yes(PalindromicTree<L>.of(w("aab")).min_palindrome_partition() == [0, 1, 1, 2]), yes(PalindromicTree<L>.of(w("abba")).contains(w("bb"))), yes(PalindromicTree<L>.of(w("abacaba")).min_palindrome_partition()[7] == 1))
var t = Trie<L>()
print(yes(t.root == 0), yes(t.is_empty), yes(t.len == 1))
t.add(w("b"))
t.add(w("ab"))
t.add(w("a"))
let ok_t = t.contains(w("ab")) and not t.contains(w("")) and t.count_less(w("b")) == 2 and t.count_less(w("ab")) == 1 and t.count_less(w("c")) == 3
t.clear()
let ac = AhoCorasick<L>([w("a"), w("ab"), w("b")])
print(yes(ok_t and t.is_empty and t.len == 1), yes(ac.root == 0 and ac.len == 4), yes([x for x in ac.find_all(w("ab"))] == [(at: 0, pattern: 0), (at: 0, pattern: 1), (at: 1, pattern: 2)]), yes(ac.fail_order() == [0, 1, 3, 2] and ac.exit_link(2) == Some(3) and ac.exit_link(1) == None and ac.pattern_state(1) == 2))
