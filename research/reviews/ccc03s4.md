# ccc03s4

Score: 5/5
Dream: 5/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header derives
n(n+1)/2 - sum(lcp), and the port is one line: `1 + SuffixArray.new(s).distinct()` under
`@!multitest`.

## Checklist failures
None.

## Owed rewrites
None.

## Language gaps
None.

## Dream notes
Sums each suffix's length minus its LCP, exactly as the header states. The port's
`.distinct()` is the same calculation named by std.
