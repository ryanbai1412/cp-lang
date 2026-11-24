# Friction report: egoi22p7

Rethought. The v0.2 port numbered segment-tree nodes `2o, 2o + 1` in `4n + 5` arrays and cleared generation-stamped marks. The v0.3 port keys designs and representatives by the span `(lo, hi)` in a `Map`, so the recursion has no heap numbering. The left-half components seen and the right components matched are short pin lists tested with `Dsu.same`, as the accepted C++ does. The embedded grader is a `List<Dsu<Pin>>` of designs. A `@check fn` accepts any design with design 0 connectivity in at most 1350 queries, since the described wires depend on the Dsu root choice.

No language gap. The accepted C++ is grader-only, so orig.cpp builds it with its LOCAL grader and prints what the port prints (its debug `assert(queries <= 20)` would fire).

## Measurements

gen.py --max (n = 200): 838 queries for both the port and the accepted C++, 0.00 s.
