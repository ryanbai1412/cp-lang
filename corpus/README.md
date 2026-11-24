# Corpus

`submissions/` holds the raw judge submissions (Ryan's and Andrew's) and
`submissions/info.json` their judge metadata. Nothing there is edited by hand.

`problems/<problem>/meta.json` is one record per problem. `scripts/corpus.py
build` regenerates the generated fields and `corpus/index.json`;
`scripts/corpus.py check` fails when a record is stale or names a missing
file.

Generated fields:

- `problem`, `source` (the contest prefix of the problem code);
- `submissions`: every submission with language, verdict, time, memory and
  file, oldest first; `cpp_ac` counts accepted C++ ones;
- `artifacts`: files elsewhere in the repo for this problem, by kind
  (`original` C++, `port`, `draft`, `dream`).

Hand-written fields, kept across rebuilds and filled in by the spec
hill-climb:

- `tags`: domain tags (`dp`, `graphs`, `trees`, `ranges`, `strings`, `math`,
  `geometry`, `flows`, `sequences`, `search`, `containers`, `io`);
- `difficulty`: `easy`, `medium`, `hard` or `null`;
- `dream`, `spec`: status of the dream and of the spec-compliant solution,
  one of `none`, `draft`, `reviewed`, `gold`;
- `notes`: what the problem taught the spec, with decision numbers.

The dream and the spec-compliant solution for a problem live beside its
record as `problems/<problem>/dream.cp` and `problems/<problem>/spec.cp`.

- `dream.cp` is the ideal program, written from the accepted C++ and the
  idea alone, without consulting the spec or running the checker. It may use
  syntax and library calls that do not exist; that is the point.
- `spec.cp` is written after comparing the dream with the spec. It must pass
  `cpl check` against `std` (failures only on forms with a `// gap:` conformance case).
- Every difference between the two is logged in the record's `meta.json` `notes`
  (older ones in `research/design-history/spec-delta/hillclimb.md`) as a spec change, a rejected wish (with the reason), or a dream bug.

`hillclimb.json` is the reproducible 100-problem set for the spec
hill-climb; progress is the `dream` and `spec` fields of those records.
