# P0 regressions

One case per silent-wrong-answer bug the scouts and blind critics found in the prototype. `x.cp` is built by
`proto/cpc`, compiled under ASan + UBSan with `-DLOCAL`, and must print `x.out`; the cases marked in
`../test_p0_regressions.py` also run in the reference interpreter `proto/cpi`, which must agree.

- `shift64`: `1 << 40` is a 64-bit shift, never 32-bit undefined behaviour.
- `value_copy`: `var t = s` and `var next = ways` copy; a later write to one never shows in the other.
- `array_literal`: `[D]T = [a, b, c]` holds one value per position.
- `brute_not_main`: a parameterless `@brute` is an oracle only and never runs in the judge build.
- `float_format`: `{x:.3f}` rounds to a fixed precision and never prints `-0.000`.
- stress honesty (in the test script): a bug that needs size 12 is found, the summary reports coverage,
  outputs that differ only in line breaks are a mismatch, and an all-rejected run is not "OK".
