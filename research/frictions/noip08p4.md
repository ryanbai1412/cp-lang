# Friction Report: noip08p4

## 1. The v0.2 port's greedy was wrong
- **Where:** the simulation.
- **What felt wrong:** trying b, a, d, c in letter order disagrees with the accepted C++ (and the judge):
  a due pop on stack 2 must happen before the next push. On `2 1 5 8 4 3 6 7 9` the v0.2 port printed
  `... b a d b`, the AC `... b d a b`. The port now pops every due value before each push.
- **Goal it hurts:** correct.
- **Severity:** high (fixed here).

## 2. Clash graph built from a predicate
- **Where:** `let apart = [Pos] i => [j for j in Pos if i != j and clash(i, j)]`.
- **What felt wrong:** an undirected graph given by a symmetric predicate needs `clash` to symmetrise itself
  (`if j < i: return clash(j, i)`), since `Graph.undirected` wants an edge array.
- **Goal it hurts:** express.
- **Ideal code:** `Graph.undirected(pairs(Pos).filter(clash))`.
- **General fix:** `Graph.undirected` over a `Seq` of pairs, not only an edge array.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with AC 5185830 (the v0.2 port disagrees, section 1); max (n = 1000) 0.011 s vs C++ 0.004 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let side = two_color(Graph.undirected(pairs(Pos).filter(clash))) else: return print(0)
for i in Pos:
    pop_due()
    stacks[side[i]].push(a[i])
    script.push(if side[i]: 'c' else: 'a')
pop_due()
print(" ".join(script))
```
