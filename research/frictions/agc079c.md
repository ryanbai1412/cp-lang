# agc079c — Reversible Queue Sort

## No O(1)-spliceable `LinkedList<T>` in stdlib (`containers`)
- **Where:** `proto/ports/ecnerwala/agc079c.cp` lines 27–28, 32–36, 50–51, 60, 70–71 vs `proto/ports/ecnerwala/dream/agc079c.cp` lines 18, 31–34, 39–71.
  ```cp
  var pred: [Val]?Val = none
  var succ: [Val]?Val = none
  fn swallow(l: Run) -> Run:
      let under = runs.pop()!
      succ[l.tail] = under.head
      pred[under.head] = l.tail
      Run(l.head, under.tail, under.opp_below)
  ```
- **What felt wrong:** Each run of unplaced values only needs O(1) `pop_front`, `pop_back`, and O(1) `splice_back` onto another run. Running `cpl check` on `dream/agc079c.cp` fails with:
  `error[E0204]: no type or domain named LinkedList, and not in the stdlib (yet)` (`dream/agc079c.cp:18:18`).
  Without a spliceable list, the port maintains parallel `pred`/`succ: [Val]?Val` arrays beside the `runs` stack and manually links `head`/`tail` pointers.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** (`dream/agc079c.cp` lines 18, 31–34)
  ```cp
  struct Run(vals: LinkedList<Val>, opp_below: Bool)

  fn swallow(mut l: Run) -> Run:
      let under = runs.pop()!
      l.vals.splice_back(under.vals)
      Run(l.vals, under.opp_below)
  ```
- **General fix:** Add an arena-backed or stdlib `LinkedList<T>` (or `SpliceList<T>`) supporting O(1) `push_front`/`push_back`/`pop_front`/`pop_back` and O(1) `splice_back`/`splice_front`. Helps every stack-of-runs, DSU-with-members, and Euler-tour/list-merging problem.
- **Severity:** medium.
