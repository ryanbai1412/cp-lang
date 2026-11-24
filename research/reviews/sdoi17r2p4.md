# sdoi17r2p4

Score: 4/5
Dream: 4/5

Cold review: Dijkstra over roads with per-town arrival and departure chains over trie preorder (lines 27-60), the header's construction, close to dream.cp.

## Checklist failures

- R9: `pre`/`suf` (line 42) are the dream's `arrived`/`departing`, and the LCP is inline at line 49 where dream.cp names `fn lcp`.

## Owed rewrites

- Take dream.cp's names and `fn lcp`.

## Language gaps

- Road and chain nodes offset into one `Node` for want of sum-domain injections, and the `LcaTable` code-size cost: in research/frictions/sdoi17r2p4.md.

## Dream notes

- None.
