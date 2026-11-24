# Friction Report: noip04p4

## 1. Backtracking state is hand-managed
- **Where:** `set(l, d)` undoes `digit`/`used` by hand on every failed guess.
- **What felt wrong:** an injective partial assignment with undo is a recurring backtracking structure.
- **Goal it hurts:** express; common mistakes hard to write (forgetting one undo).
- **Ideal code:** `var digit: Injection<Letter, Digit>` with `digit.try(l, d) { ... }` scoping the undo.
- **General fix:** a std partial bijection with scoped assignment.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5117252 on unique puzzles (n <= 6, uniqueness checked by brute force); max (base 26, a + a = c) 0.033 s vs C++ 0.41 s, same output.
