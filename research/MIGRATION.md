# Migration log

What the restructure in `plan/RESTRUCTURE.md` deleted and where its content
went, so lossiness can be checked.

## Deleted branches

Remote branches fully merged into `main` when deleted, so every commit is
still on `main`. Restore one with `git push origin <sha>:refs/heads/<name>`.

| Branch | Tip |
|---|---|
| `devin/1795157722-docs-trace-offline` | `7b9aae5679bc866b24ae2bdb75078d973b5e8a83` |
| `devin/conf-04` | `23949db866e8811d9ddce20b8871a31ff37eb9bb` |
| `devin/conf-05` | `fe1a1a0682ed8d69e397efdda7a3f51960884f3a` |
| `devin/conf-08` | `76ad1330a85e58be9b1c6957eea673480cdbd052` |
| `devin/contra-a` | `d92fa8cd7c72cf0d6db891cc52ae7da915eb6a0b` |
| `devin/contra-b` | `f243720f459d0a34e86da4bf3e3bfdcd43df91bc` |
| `devin/docs-a` | `e91e49ec5676a069274d886fe5301743bcb220c1` |
| `devin/docs-b` | `3fe078bcb5027834d873468f2611d66b451adb6d` |
| `devin/docs-c` | `c03216fb212cabb98fc71341f2b7271be1902f82` |
| `devin/docs-d` | `12d1caa506196cae4848658ecccd2b5b5ba82acb` |
| `devin/docs-e` | `2101d5be04f49ca0b077326831369ececfc05a79` |
| `devin/docs-f` | `7495dbdb92438d6330f4e3d74bdda7ffb171fa5f` |
| `devin/spec-containers` | `3cd415453a80218fe71fc637b596c0291731ca61` |
| `devin/spec-flows-2` | `d9fd4f96c32de52fe6ca3a0aa25ef6685a3286bf` |
| `devin/spec-geometry` | `3968013467f23538eff8147eb30b950392eebcbf` |
| `devin/spec-graph` | `2754183c4e55146403a45e22e7a804c0a4d3da7b` |
| `devin/spec-ranges` | `de544505694e36683f0b35b4eb62bcce90ccca73` |
| `devin/spec-strings` | `6ddebcf8ed6b7637714ee0d5f5aac45ceeed1e43` |
| `devin/spec-trees` | `71b31e63d033e997bb737797edd9cb3ab19bbe71` |
| `devin/spec-window` | `53e31c34e50f3d9bdcad5b8214b379e61147bd09` |
| `rewrite-c` | `ac17a3434e9411f86564bc7bf84139017d567bdf` |

## Deleted paths

| Path | Where its content is |
|---|---|
| `subs/submissions/` | byte-identical to `corpus/submissions/` |
| `subs/body_*.cpp`, `subs/*.txt` | a 50-file sample of solution bodies, each contained in its `corpus/submissions/<id>.cpp` |
