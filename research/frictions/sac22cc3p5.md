# Friction report: sac22cc3p5

There is no dream.cp; the ideal is below. The port lowercases and dedupes the banned words, builds `AhoCorasick<Letter>` over them, and counts each message's distinct matched patterns.

## Dream

```cp
let words = read<[n]Str>().map(.to_lower).distinct()
let ac = AhoCorasick<Letter>.new(words.map(Letter.list))
for msg in read<[m]Str>(): print(ac.find_all(Letter.list(msg.to_lower)).distinct(by: .pattern).len)
```

## Strings become `List<Letter>` through a hand helper
- **Where:** line 15: `fn letters(s: Str) -> List<Letter> = [Letter(c.lower) for c in s]`. Line 18 dedupes `by: w => w.to_lower` and then lowercases each word a second time in `letters`.
- **What felt wrong:** converting text to a letter domain is a per-port helper. The lowercasing is split between `distinct(by:)` and `letters`, so the automaton's alphabet and the dedupe key can drift apart.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** the dream's one `map(.to_lower)` followed by a std `Letter.list(s)` (a checked `Str` to `List<D>` conversion for a `Char` domain).
- **General fix:** a std conversion from `Str` to a list over a `Char` domain. noip20p2 and other string ports write the same helper. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 138, accepted C++ 533 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.12 s / 17 MB, accepted C++ 0.54 s / 6 MB, with the same output.
- **Emitted C++:** 28358 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
- **Memory gap:** the port's RSS at `--max` is over 2x the accepted C++'s (above). The repro (this package's `--max` case) is sent to perf via Factory 7bd146f2. Owner: Perf B (33c55557).
