# Scout batch 1

Corpus submissions re-solved the way we would like to write them. This is
a Stage-1 critique pass: the files are design sketches, not regression
ports. Every line that the prototype rejects today is marked `// WISH Fn`,
and `Fn` is an item in [`research/archive/frictions/friction-backlog.md`](../../../research/archive/frictions/friction-backlog.md).

The batch had ten sketches. The first Stage-2 sprint landed what eight of
them needed, and those moved up to `proto/ports/`, where
`proto/run_all.sh` stresses them against their originals: ccc18s5,
ccc21s4, ccc22s4, cco18p1, cco20p1, ioi04p4, ioi10p3 and ioi11p2.

Two are left here and do not build yet. `proto/run_all.sh` only looks at
`proto/ports/*.cp`, so this folder does not affect it. The originals are in
`corpus/submissions/`:

- `ccc19s5.cp` from `5119937.cpp`: maximum over every size-k sub-triangle.
  Needs a triangular index (F4).
- `ccc05s5.cp` from `4173126.cpp`: online rank queries, float output.
  Needs a sorted bag with ranks (F13) and floats (F15, decision 1.5).
