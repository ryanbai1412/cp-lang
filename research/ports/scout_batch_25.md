# Scout Batch 25: Hard Greedy, Ad-hoc, Bitwise and Constructive Problems

Stage-1 scouting batch over 10 previously unported problems from `corpus/` (no overlap with the ports that existed when it was picked, `scout_batch_1`–`scout_batch_24`, or `blind_critic_*`; see the note on two later overlaps below): a 64-bit Gray code, three-key ranking, sorted-window greedy, a bit-reversal construction, top-bit recursion, exchange-argument greedy, fracturing search over a heap, bitset subset sum with reconstruction, a constructive whiteboard strategy, and an XOR step on a graph.

The friction analysis is in `research/friction-backlog-batch-25-greedy-bitwise.md`.

## Layout

- Top-level `*.cp` (10 files): ideal CP Lang, written from the statement before the C++ was opened. They do not build today; the constructs that are missing are listed in the backlog.
- `today/*.cp` (10 files): ports that build with `proto/cpc` today.
- `verify.py`: builds every `today/` port on a temporary copy, compiles it with `g++ -std=c++17 -O2 -fsanitize=address,undefined`, and runs it against the accepted C++ submission on random small and medium inputs. Multi-answer problems are judged by checkers in the script, and the five IOI grader functions get a small `main` appended.

```bash
python3 proto/ports/scout_batch_25/verify.py
```

## How the statements were obtained

`dmoj.ca/api/v2/problem/<code>` and the statement pages both answered with a Cloudflare challenge (HTTP 403) for the whole session. Nine problems are well-known olympiad tasks and were written from their published contest specs (NOI, NOIP, IOI, Baltic OI). The tenth, `dmopc21c7p3`, has no spec outside DMOJ: its task description was reconstructed from the accepted submission, and its file says so in the header.

Two things about the selection. The corpus has no unported XOR linear basis or meet-in-the-middle submission (a search of the unported accepted C++ for `basis`, `meet`, `mitm` and `half` found none; that is a text search, not a read of every file), so those two themes are not represented. And four problems first picked for this batch (`noi14p1`, `noi21p4`, `btoi19p2`, `btoi19p4`) were landed by scout batches 20 and 22 while this one was being written; they were removed and replaced by `noip19p1`, `ioi09p3`, `ioi16p1` and `ioi18p4`, which are easier than the ones they replace. The backlog still cites what the four dropped drafts ran into, marked as such.

Two overlaps remain. `ioi20p4` and `noi20p4` were also landed by scout batches 28 and 29 minutes before this batch was pushed. They are kept here because GB1 (the 32-bit shift) and GB5 (`Bitset`) in the backlog rest on them; the ports were written independently, so the two versions can be compared.

## Problems

1. **`noip19p1`** (NOIP '19, *Gray Code*, `corpus/submissions/5127723.cpp`): the k-th n-bit word is `k ^ (k >> 1)`, with `k` up to 2^64 - 1. 300 cases including `n = 64`, identical output.
2. **`ioi09p3`** (IOI '09, *POI*, `5106576.cpp`): rank by score, then tasks solved, then id. 250 cases, identical output.
3. **`ioi16p1`** (IOI '16, *Detecting Molecules*, `4686581.cpp`, grader function): a window over the weights in sorted order. Up to 300 cases; the subset is checked and existence equals the original's.
4. **`ioi18p4`** (IOI '18, *Mechanical Doll*, `4724581.cpp`, grader function): one root switch over a complete binary tree, leaves visited in bit-reversed order, unused subtrees pruned. 150 cases up to `n = 3000`; the circuit is simulated for both the port and the original, and the switch count is checked against `n + log2(n)`.
5. **`ioi20p4`** (IOI '20, *Packing Biscuits*, `4733047.cpp`, grader function): count of valid `y` below a cap, recursing on the top bit. 200 files of 5 calls, identical output.
6. **`ioi20p3`** (IOI '20, *Carnival Tickets*, `4724579.cpp`, grader function): exchange-argument greedy over upgrade gains, then dealing rounds. 200 cases; the allocation is re-scored by the checker and the prize equals the original's.
7. **`btoi19p6`** (Baltic OI '19, *Olympiads*, `4475247.cpp`): C-th best team by fracturing search over a heap. 200 cases, identical output.
8. **`noi20p4`** (NOI '20, *Dish Making*, `3890337.cpp`): constructive greedy for `m >= n - 1`, subset sum with reconstruction for `m = n - 2`. 250 cases; every plan is checked and feasibility equals the original's. The `today/` port stores the subset-sum table as one `Bool` per cell, so it is not sized for the full limits.
9. **`ioi22p2`** (IOI '22, *Prisoner Challenge*, `4783608.cpp`, grader function): a fixed tree over the coin counts with fan-out 3,3,3,3,3,2,2,1, 21 whiteboard states (`x = 20`). The strategy is simulated on every bag pair for 13 values of `N` up to 207, and on all adjacent pairs plus 40,000 random pairs at `N = 5000`, for both the port and the original.
10. **`dmopc21c7p3`** (DMOPC '21 Contest 7 P3, `4453037.cpp`, source-first): one XOR step over a wire graph, `-1` when a lit lamp reaches a cycle. 300 cases, identical output.

## Size

Non-blank, non-comment lines. The ten originals total 867 lines raw and 580 with the shared template header and `main` removed. The ideal ports total 245 lines (71.7% below raw, 57.8% below stripped), and that count includes their `@brute` and `@checker` blocks. The `today/` ports total 443 lines (48.9% below raw, 23.6% below stripped). Three `today/` ports are not shorter than their stripped C++: `noip19p1` (22 against 17, the 64-bit emulation), `ioi18p4` (49 against 47) and `dmopc21c7p3` (47 against 47, a hand-written adjacency list and iterative DFS).

## Not tested

Nothing was run at the full limits: no timing, no peak memory. The ideal top-level files were never compiled.
