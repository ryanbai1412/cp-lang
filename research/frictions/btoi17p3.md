# Friction report: btoi17p3

## A method value copies its receiver on every call

- **Severity:** high
- **Status:** open (routed to Core)
- **Goal it hurts:** the obvious way is the fast way
- **Area:** codegen
- **What happened:** `plan.sorted(by: tree.pos)` copied the whole `Rooted`
  once per key call, so the `--max` case took 7 s against a 1 s limit. The
  lambda `c => tree.pos(c)` borrows `tree` and runs in 0.02 s.
- **General fix:** a method value `x.f` lowers like `a => x.f(a)`,
  borrowing `x` instead of copying it.

## Dream

```cp
let tour = plan.sorted(by: tree.pos)
```
