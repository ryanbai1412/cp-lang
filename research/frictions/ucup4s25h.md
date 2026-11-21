# Friction: `ucup4s25h`

## String slicing `s[lo..hi]` lowers to an $O(n^2)$ character-concatenation loop (26x slowdown)

- **Severity:** high
- **Goal it hurts:** performance
- **Measured:** `221.75 ms` vs `8.51 ms` (`26.1x`) on `gen size=25000` (`research/scoreboard.json`).
- **Cause (codegen pattern):** `s[0..del]` and `s[del + 1..n]` emit `for (t5 = lo; t5 < hi; t5++) t4 = t4 + Capture(put_char(s[t5]))`, reallocating the entire string on every character instead of calling `s.substr(lo, hi - lo)`.
- **Fix:** lower `Str` slicing `s[lo..hi]` directly to `s.substr(lo, hi - lo)` (after a single bounds check).
