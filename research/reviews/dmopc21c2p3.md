# dmopc21c2p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

An interactive search for a hidden number by depth: each guess fixes the known high bits and randomises the low ones, and the reply says how many bits were missed.

## Checklist failures

- `known`, `low` and `guess` are bit-twiddled with `<<`, `>>`, `|` and `^` in one line (`known = (known | ((guess >> low) << low)) ^ (1 << low)`), with no name for the step.
- `missed / 2 - 1` decodes the reply by arithmetic without saying what it counts.
- The search is a `loop:` with one `break`.

## Gaps that need the language

- `@!multitest` drops the bound on the test count: research/frictions/dmopc21c2p3.md.
