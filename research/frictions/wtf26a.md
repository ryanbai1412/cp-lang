# wtf26a

- **Source:** AtCoder WTF 2026 A (`proto/ports/ecnerwala/wtf26a_orig.cpp`)
- **Port:** `proto/ports/ecnerwala/wtf26a.cp`
- **Dream:** `proto/ports/ecnerwala/dream/wtf26a.cp`
- **Status:** Ported (`gate.sh wtf26a 150` passes)

## No Kruskal reconstruction tree

- **Goal it hurts:** express, hard to get wrong
- **Severity:** medium
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/wtf26a.cp:18-19, 23-24`
- **Actual:** `proto/ports/ecnerwala/wtf26a.cp:14, 19, 23-33, 46-50`
- **General fix:** a `KruskalTree<V>(edges)` (the DSU merge tree) with a `nodes` domain, `parent`, `size`, `weight` and `other_size` per node and the list of `merges` with each side's leaf count. The port builds it by hand with a `top` table, `n + e` node ids and a `?Cut` per node, and runs a second DSU by hand to count leaves per join.

## No top-down accumulation over a tree

- **Goal it hurts:** express
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/wtf26a.cp:20`
- **Actual:** `proto/ports/ecnerwala/wtf26a.cp:35-36`
- **General fix:** a `down(init, f)` on any rooted tree (parent links) that folds from the root to every node; the port relies on parents having larger ids and walks `Node.rev`.

## No saturating multiply

- **Goal it hurts:** hard to get wrong
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/wtf26a.cp:21`
- **Actual:** `proto/ports/ecnerwala/wtf26a.cp:40`
- **General fix:** a capped product (`min(a * b, cap, saturating)` or `a.mul_capped(b, cap)`) that never forms the overflowing product, instead of the hand `a > cap / b` test.
