# Doc notes: `std.strings`

Choices made where the spec was silent, and contradictions found, while bringing
`spec/std/strings.cp` to the `spec/std/README.md` standard. One bullet each:
entry, what the docs now say, why.

## Contradictions

- `starts_with`, `split_first`, `trim`: `decisions.md` (1.6, 16.2) spells `s.starts_with(x, at: i)`, `s.split_once`, `s.trim_start`; the stub and `spec/lib/strings.md` have `starts_with` without `at:`, `split_first`, and only `trim`. Documented the stub as written; the decisions entries need updating or the stub needs the extra forms.
- `digit`: the stub said the method "stops" on a non-digit, `spec/lib/strings.md` says it traps. Documented a trap, the only reading that gives a defined result.
- `period`: the stub defined it as the smallest `d >= 1`, undefined on empty input; `spec/lib/strings.md` says `0` for an empty sequence. Documented `0` on empty.
- `min_rotation`: `spec/lib/strings.md` returns `Int` with `0` for empty input; the stub returns `?K`. Kept the stub signature and documented `None` on empty.
- `SuffixArray.rank_range`, `SuffixArray.occ_counts`: `spec/lib/strings.md` names them `matches` and `occurrences`. Documented the stub names.
- `suffix_array`: `spec/lib/strings.md` gives `suffix_array(s) -> List<Int>` in O(n) and a separate `lcp_array`; the stub's `suffix_array` returns a `SuffixArray` and there is no free `lcp_array`. Documented the stub.
- `Palindromes.count`, `Palindromes.longest`: `@get` in `spec/lib/strings.md`, plain methods in the stub. Kept the stub.
- `SuffixAutomaton.topo_order`: `spec/lib/strings.md` describes decreasing `longest`; the stub says non-decreasing. Documented non-decreasing (iterate backwards for bottom-up DP).
- `SuffixAutomaton.reset`, `SuffixAutomaton.add_word`: the lib coverage table lists `reset_last` and `push_seq`. Documented the stub names.
- `PalindromicTree`: the lib coverage table lists `odd_root` and `even_root` accessors that the stub lacks; docs refer to the roots as states `0` and `1`.
- `AhoCorasick.find_all`: `spec/lib/strings.md` calls it `matches`, and its coverage table lists a `patterns_len` the stub lacks. Documented `find_all` with the lib's law (`at` is the start).
- `SuffixAutomaton.new`, `PalindromicTree.new`, `Trie.new`: the stub said O(1), `spec/lib/strings.md` says O(|A|) (a root with a domain-indexed child table). Documented O(|A|).
- `SuffixArray.count("")` is `n` (every suffix has the empty prefix) while the free `find_all("")` yields `n + 1` keys (the end key too). Both follow their own definitions; flagged so one convention can be picked.

## Choices

- `repeat`: traps on `n < 0`, like `seq.repeat`.
- `starts_with`, `ends_with`: cost written O(min(|self|, |pat|)), a tighter form of the stub's O(|pat|).
- `find`: an empty pattern gives `Some(first key)` (the end key when `self` is empty); leftmost occurrence wins (Rust/Python).
- `rfind`: an empty pattern gives `Some(end key)`; rightmost occurrence wins (Rust/Python).
- `lower`, `upper`: non-letters are returned unchanged.
- `split`: keeps empty pieces; `k` separators give `k + 1` pieces and `""` gives `[""]` (Python/Rust).
- `split_first`: `None` when the separator is absent.
- `join`: each element is written as `"{x}"` so numbers join directly.
- `replace`: greedy, left to right, non-overlapping; an empty `from` traps (inserting everywhere has no contest use and hides bugs).
- `parse_int`: optional leading `-` only; `+`, whitespace, empty, stray characters and out-of-range values give `None`.
- `parse`: a tuple type reads one whitespace-separated token per field.
- `prefix_matches`: after a full match the scan continues from the pattern's longest border, so overlapping matches are reported.
- `max_rotation`: `None` on empty input, matching the stub's `min_rotation`.
- `KmpAutomaton`: states are matched-prefix lengths `0..=|pat|`; a full-match state steps through its failure link.
- `find_all_wildcard`: the wildcard is special only in the pattern, never in the text.
- `Palindromes.longest_between`: cut positions range over `0..=len`.
- `SuffixArray.lcp_between`: accepts the empty suffix at position `n` (result `0`).
- `SuffixArray.rank_range`: an absent pattern gives the empty range at its insertion rank, so `count` and ordering queries still work.
- `SuffixArray.lcp_of_ranks`: a one-rank range gives that suffix's length.
- `SuffixArray.split`: returns the smallest rank achieving the minimum LCP.
- `SuffixArray.kth_distinct`: reports the occurrence in the smallest-rank suffix that has it; traps on `k < 0`.
- `suffix_tree_edges`: the meaning of `top_size` and `bottom_size` comes from its one user, the `ucup5s05f` port (total `edge_len` below the node, and that plus the edge itself).
- `HashVal.hash_token`: `(h1 << 31) | h2`, collision-free over the pair.
- `RollingHash`, `RollingHash61`, `DynamicHash`: polynomial hash with salted random bases unless one is passed; `combine` grows the power table as needed; cross-hash comparisons (`lcp_with`) trap on differing bases.
- `Hash2D`: independent row and column bases; the exact formula is left to the implementation.
- `SuffixAutomaton`: states are numbered in creation order, a clone right after the new state; the root's `occ_count` is `n`; `topo_order` and `states` break ties by smaller state number.
- `SuffixAutomaton.lcs`: ties go to the smallest `end`; `(0, 0)` when nothing is shared.
- `SuffixAutomaton.kth_substring`, `Trie.kth`: a proper prefix sorts before its extensions; `k < 0` traps.
- `lcs_all`: `0` for no words.
- `PalindromicTree`: both roots have `diff` `0` and `series_link` `0`; `last_state` is the even root `1` on an empty string; `contains("")` is `true`; `pop` traps when there is no push to undo.
- `Trie.find`: `Some(0)` for the empty word unless the trie stores nothing, matching `count_prefix(word) > 0`.
- `AhoCorasick`: an empty pattern ends at the root and matches at every `at` in `0..=|text|`, as the lib's law requires; `find_all` and `words_at` order equal-end matches longest first, then by index; `fail_order` breaks equal depths by state number; `exit_link` is `None` when no pattern ends strictly above.
- `DynamicAhoCorasick.remove`: traps when no copy is stored (it returns nothing, so a silent no-op would hide the bug; `Trie.remove` returns `Bool` instead).
