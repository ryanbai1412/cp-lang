# Friction Report: ioi08p1

## Dream

`dream.cp` (verified): a `Trie<Letter>` walk that defers the child on the
longest word's path.

## Gaps

- **Trie nodes are plain `Int`s.** `Trie.root` and `Trie.step` hand out node
  numbers, so "the nodes on the longest word" is a `Set<Int>`; a trie node
  domain would make it a `[Node]Bool`. Severity: low.
- **Words are not bounded in their read type.** "at most 20 lowercase letters"
  is not something a `Str` read can state. Severity: low.
