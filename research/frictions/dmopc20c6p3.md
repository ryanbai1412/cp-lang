# Friction report: dmopc20c6p3

No gap: the port is the dream (the attractor of the dead ends, walked back over `Graph.transpose` with `while let`).

## Dream

```cp
let police: [City]Bool = read()          // the 0/1 flags
...
            if police[p] or free[p] == 0:
...
print(caught)                            // one 0/1 per city
```

The rest is the port as written.

## A 0/1 flag line cannot be read or printed as `[City]Bool`
- **Where:** `let police: [City]0..=1 = read()` with `police[p] == 1`, and `print([Int(caught[u]) for u in City])`.
- **What felt wrong:** the statement's p_i and the answer are booleans written as 0/1; the port carries them as integers on the way in and converts each one on the way out.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `[City]Bool` read and `print(caught)`.
- **General fix:** `read()` of a `Bool` from a `0`/`1` token and `Show` for `[D]Bool` as 0/1 digits. Today `read()` of a `Bool` is E0315 ("it is not a token type") and `print` of a `[3]Bool` is E0606. Owner: Core (12cfdb46).
- **Severity:** low.

## Measurements

gen.py 1 500000 (a 5e5 chain): port 0.04 s, accepted C++ 0.14 s.
