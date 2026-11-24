# Friction report: btoi17p4

Rethought. The v0.2 port marked decisions as Int 0/1/-1 and used `found`, `cans` and `bad` flags. `solve()` printed from four places, and four lints fired on hand loops. In the v0.3 port:

- each side is a `[V]?Bool`;
- the search is `grow() -> ?List<V>`;
- `groups() -> ?[V]List<V>` bails with let-else;
- the pair loop is `V.after(i)`;
- the output has one `home`/`detention` site.

## Gap

The search still threads global `members` and `side` state through its recursion. `minus` and `valid` keep a stamp array because a `List` has no O(1) membership test.

## Measurements

gen.py --max ends in detention at once for both the port and the accepted C++, so it does not exercise the full 2^(p+q) search. Random cases up to n = 300 agree with the v0.2 port and the accepted C++.
