# 90. Open questions

**Layer:** appendix. Collected from every chapter. Status: [proposed].

<!-- from 01-index-spaces.md §1.17 -->
## 1.17 Open questions

1. `A.Cut` (a type, capitalized like one) vs `A.cuts`. Reviewer: keep `A.Cut`.
2. `D.at(i)` vs `D(i)`. Reviewer: keep `at`; `D(x)` is for meaning-preserving conversions like `I32(x)`.
3. Do product/sum/shape domains survive the ports? (Backlog.)
4. ~~Nonempty as a flag or a type?~~ A type, `NonEmpty<…>` (§1.10) [r10].

<!-- from 02-contexts-and-markers.md §2.9 -->
## 2.9 Open questions

1. ~~Validation in contexts?~~ Only through types (literal and range types) [r10].
2. ~~Bare `with` mid-block?~~ Top of block only [r10].
3. ~~`@grader`/`@multitest` as markers on the program?~~ No: `--- with multitest() ---` wraps the rest of the file as the per-test closure (16.3) [p4].

<!-- from 03-sequences-and-optionals.md §3.7 -->
## 3.7 Open questions

1. Reviewer wants parentheses dropped as an `x?` boundary (parens would silently change meaning). You
   accepted parens; kept for now.
2. ~~Two scan defaults?~~ One principle: cuts iff the monoid has an identity in `T` [r10, r11].
3. Reviewer: drop `with_prev`, keep `pairs`. Kept, because you endorsed `prev ?? fallback`.

<!-- from 04-stress-and-interaction.md §4.7 -->
## 4.7 Open questions

1. ~~Caps checked against generators?~~ Yes, when static (§4.1) [r10].
2. ~~Tests per input?~~ Several, with `t` from `gen` and shrunk toward 1, because state leaking between tests
   is the main multitest bug [r10].

<!-- from 05-segment-trees.md §5.5 -->
## 5.5 Open questions

1. ~~`restrict` as its own kind?~~ No: an optional method under one law [r10].
2. Shorter form `SegTree<Tick>(init: values)`, inferring `D` and `M` from `init` (reviewer). The exemplars
   still write all three; decide with the ports.

<!-- from 06-functions-state-control.md §6.6 -->
## 6.6 Open questions

1. ~~Reset statics per test?~~ Never; a non-static already gives per-test state [r11].
2. ~~Negative `repeat n` a compile error?~~ No; runtime trap, ranges on the read rule it out [r11].

<!-- from 07-numbers.md §7.6 -->
## 7.6 Open questions

1. Should `Int` overflow trap in release too, given the ≤ 5% check budget? Ryan decided: wrap in release.
2. `I128` availability depends on the judge compiler; the bundler must reject it on profiles without it.

<!-- from 08-strings-io-containers.md §8.5 -->
## 8.5 Open questions

1. ~~Per-string domain?~~ Keep `0..s.len`; revisit if ports show cross-string index bugs [r11].
2. ~~Default `Heap` order?~~ None; `order:` required [r11].
