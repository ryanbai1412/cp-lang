# Repository Restructuring Migration Log

Records every deleted path, why it was deleted, and where any unique content went.

## Deleted `proto/` and legacy tooling paths

Migrated 2422 files from `proto/ports/` into `corpus/problems/<id>/`, `corpus/lib/{scout,real}`, and `research/`; deleted 763 superseded/duplicate files from `proto/` plus `bench/corpus_registry.py`.

### Legacy tooling deleted

- `bench/corpus_registry.py` — auto_strip folded into scripts/scoreboard.py; corpus discovery moved to corpus/problems/

### Superseded pre-v0.3 duplicate port (kept latest port in corpus/problems/<id>/port.cp) (392 files)

- `proto/ports/ccc03s4.cp`
- `proto/ports/ccc10s4.cp`
- `proto/ports/ccc17s4.cp`
- `proto/ports/ccc18s4.cp`
- `proto/ports/ccc21s2.cp`
- `proto/ports/ccc21s3.cp`
- `proto/ports/cco18p1.cp`
- `proto/ports/dpk.cp`
- `proto/ports/dpl.cp`
- `proto/ports/dpm.cp`
- `proto/ports/dpr.cp`
- `proto/ports/dpt.cp`
- `proto/ports/dpx.cp`
- `proto/ports/dpz.cp`
- `proto/ports/hillclimb_dp_graph/apio09p3.cp`
- `proto/ports/hillclimb_dp_graph/before/apio07p3.cp`
- `proto/ports/hillclimb_dp_graph/before/apio09p3.cp`
- `proto/ports/hillclimb_dp_graph/before/coi06p2.cp`
- `proto/ports/hillclimb_dp_graph/before/dmopc21c5p1.cp`
- `proto/ports/hillclimb_dp_graph/before/dpp.cp`
- `proto/ports/hillclimb_dp_graph/before/dps.cp`
- `proto/ports/hillclimb_dp_graph/before/dpu.cp`
- `proto/ports/hillclimb_dp_graph/before/dpv.cp`
- `proto/ports/hillclimb_dp_graph/before/ioi11p4.cp`
- `proto/ports/hillclimb_dp_graph/before/noi15p2.cp`
- `proto/ports/hillclimb_dp_graph/before/noip06p1.cp`
- `proto/ports/hillclimb_dp_graph/before/sac22cc3p4.cp`
- `proto/ports/hillclimb_dp_graph/dmopc21c5p1.cp`
- `proto/ports/hillclimb_dp_graph/dpo.cp`
- `proto/ports/hillclimb_dp_graph/dpp.cp`
- `proto/ports/hillclimb_dp_graph/dpu.cp`
- `proto/ports/hillclimb_dp_graph/dpv.cp`
- `proto/ports/hillclimb_dp_graph/ioi11p4.cp`
- `proto/ports/hillclimb_dp_graph/noi15p2.cp`
- `proto/ports/hillclimb_dp_graph/noip06p1.cp`
- `proto/ports/noip07p2.cp`
- `proto/ports/scout_batch_1/ccc19s5.cp`
- `proto/ports/scout_batch_10/dream/noi17p2.cp`
- `proto/ports/scout_batch_12/bugs/let_then_push.cp`
- `proto/ports/scout_batch_13/apio07p2.cp`
- `proto/ports/scout_batch_13/coci12c1p4.cp`
- `proto/ports/scout_batch_13/coci21c4p4.cp`
- `proto/ports/scout_batch_13/cpc21c1p5.cp`
- `proto/ports/scout_batch_13/ioi05p4.cp`
- `proto/ports/scout_batch_13/ioi11p3.cp`
- `proto/ports/scout_batch_13/tle17c5p6.cp`
- `proto/ports/scout_batch_14/csps21p2.cp`
- `proto/ports/scout_batch_14/dmopc18c1p6.cp`
- `proto/ports/scout_batch_14/ioi16p6io.cp`
- `proto/ports/scout_batch_14/noi15p3.cp`
- `proto/ports/scout_batch_14/noi19p1.cp`
- `proto/ports/scout_batch_14/noi23p2.cp`
- `proto/ports/scout_batch_14/noip00p4.cp`
- `proto/ports/scout_batch_14/noip21p2.cp`
- `proto/ports/scout_batch_15/btoi20p3.cp`
- `proto/ports/scout_batch_15/dream/noi09p4.cp`
- `proto/ports/scout_batch_15/noi09p4.cp`
- `proto/ports/scout_batch_17/apio07p2.cp`
- `proto/ports/scout_batch_17/dpw.cp`
- `proto/ports/scout_batch_17/ioi11p3.cp`
- `proto/ports/scout_batch_17/noi10p2.cp`
- `proto/ports/scout_batch_18/btoi17p2.cp`
- `proto/ports/scout_batch_18/btoi20p3.cp`
- `proto/ports/scout_batch_18/ccc22s5.cp`
- `proto/ports/scout_batch_18/cco21p1.cp`
- `proto/ports/scout_batch_18/cco21p4.cp`
- `proto/ports/scout_batch_18/dpw.cp`
- `proto/ports/scout_batch_18/ioi05p2.cp`
- `proto/ports/scout_batch_18/noi09p4.cp`
- `proto/ports/scout_batch_18/noi10p2.cp`
- `proto/ports/scout_batch_18/noi18p1.cp`
- `proto/ports/scout_batch_19/ccc04s5.cp`
- `proto/ports/scout_batch_19/ccc23s3.cp`
- `proto/ports/scout_batch_19/ceoi22p2.cp`
- `proto/ports/scout_batch_19/coci21c6p3.cp`
- `proto/ports/scout_batch_19/dmopc21c5p4.cp`
- `proto/ports/scout_batch_19/egoi23p3.cp`
- `proto/ports/scout_batch_19/ioi07p1.cp`
- `proto/ports/scout_batch_19/ioi14p3.cp`
- `proto/ports/scout_batch_19/ioi16p5.cp`
- `proto/ports/scout_batch_23/coci21c1p2.cp`
- `proto/ports/scout_batch_23/dpi.cp`
- `proto/ports/scout_batch_23/egoi22p2.cp`
- `proto/ports/scout_batch_23/ioi08p3.cp`
- `proto/ports/scout_batch_23/ioi17p3.cp`
- `proto/ports/scout_batch_23/noi07p1.cp`
- `proto/ports/scout_batch_23/noi09p5.cp`
- `proto/ports/scout_batch_23/noi18p2.cp`
- `proto/ports/scout_batch_23/noi19p5.cp`
- `proto/ports/scout_batch_23/noimps22p4.cp`
- `proto/ports/scout_batch_24/cco16p5.cp`
- `proto/ports/scout_batch_24/cco18p5.cp`
- `proto/ports/scout_batch_24/ceoi22p1.cp`
- `proto/ports/scout_batch_24/dmpg15g6.cp`
- `proto/ports/scout_batch_24/ioi07p3.cp`
- `proto/ports/scout_batch_24/ioi13p3io.cp`
- `proto/ports/scout_batch_24/noi05p2.cp`
- `proto/ports/scout_batch_24/noi07p4.cp`
- `proto/ports/scout_batch_24/noi20p2.cp`
- `proto/ports/scout_batch_24/noip22p4.cp`
- `proto/ports/scout_batch_25/btoi19p6.cp`
- `proto/ports/scout_batch_25/dmopc21c7p3.cp`
- `proto/ports/scout_batch_25/ioi09p3.cp`
- `proto/ports/scout_batch_25/ioi16p1.cp`
- `proto/ports/scout_batch_25/ioi18p4.cp`
- `proto/ports/scout_batch_25/ioi20p3.cp`
- `proto/ports/scout_batch_25/ioi20p4.cp`
- `proto/ports/scout_batch_25/ioi22p2.cp`
- `proto/ports/scout_batch_25/noi20p4.cp`
- `proto/ports/scout_batch_25/noip19p1.cp`
- `proto/ports/scout_batch_26/noi20p2.cp`
- `proto/ports/scout_batch_27/cco19p2.cp`
- `proto/ports/scout_batch_27/ioi18p3.cp`
- `proto/ports/scout_batch_27/noi07p1.cp`
- `proto/ports/scout_batch_27/noi10p3.cp`
- `proto/ports/scout_batch_28/ioi20p4.cp`
- `proto/ports/scout_batch_29/noi20p4.cp`
- `proto/ports/scout_batch_3/aac4p3.cp`
- `proto/ports/scout_batch_3/ccc18s4.cp`
- `proto/ports/scout_batch_3/cco04p4.cp`
- `proto/ports/scout_batch_3/cco09p4.cp`
- `proto/ports/scout_batch_3/coci06c1p6.cp`
- `proto/ports/scout_batch_3/dpr.cp`
- `proto/ports/scout_batch_3/modinv.cp`
- `proto/ports/scout_batch_3/noip99p2.cp`
- `proto/ports/scout_batch_3/today/ccc03s4.cp`
- `proto/ports/scout_batch_30/coci21c1p5.cp`
- `proto/ports/scout_batch_31/cco19p2.cp`
- `proto/ports/scout_batch_31/coci20c6p1.cp`
- `proto/ports/scout_batch_31/coci25c3p1.cp`
- `proto/ports/scout_batch_31/dmopc19c4p2.cp`
- `proto/ports/scout_batch_31/dmopc21c6p4.cp`
- `proto/ports/scout_batch_31/dmopc21c7p1.cp`
- `proto/ports/scout_batch_31/imo22p6checker.cp`
- `proto/ports/scout_batch_31/noi10p3.cp`
- `proto/ports/scout_batch_31/sac22cc1p3.cp`
- `proto/ports/scout_batch_31/ucrpc21d.cp`
- `proto/ports/scout_batch_31/ucrpc21f.cp`
- `proto/ports/scout_batch_31/ucrpc21g.cp`
- `proto/ports/scout_batch_32/996warmup8p3.cp`
- `proto/ports/scout_batch_32/aac5p3.cp`
- `proto/ports/scout_batch_32/ceoi22p5.cp`
- `proto/ports/scout_batch_32/dmopc20c4p4.cp`
- `proto/ports/scout_batch_32/iati21a22.cp`
- `proto/ports/scout_batch_32/ioi04p2hard.cp`
- `proto/ports/scout_batch_32/noimps23p1.cp`
- `proto/ports/scout_batch_32/noip10p1.cp`
- `proto/ports/scout_batch_32/rpeng3.cp`
- `proto/ports/scout_batch_32/ucrpc21e.cp`
- `proto/ports/scout_batch_33/ioi09p5.cp`
- `proto/ports/scout_batch_34/dmopc20c6p3.cp`
- `proto/ports/scout_batch_34/dmopc20c6p6.cp`
- `proto/ports/scout_batch_34/dmopc20c7p2.cp`
- `proto/ports/scout_batch_34/dmopc21c2p2.cp`
- `proto/ports/scout_batch_34/dmopc21c2p4.cp`
- `proto/ports/scout_batch_34/dmopc21c2p5.cp`
- `proto/ports/scout_batch_34/dmopc21c2p6.cp`
- `proto/ports/scout_batch_34/dmopc21c7p4.cp`
- `proto/ports/scout_batch_34/dmopc21c8p3.cp`
- `proto/ports/scout_batch_34/dmopc21c8p5.cp`
- `proto/ports/scout_batch_35/coci21c1p1.cp`
- `proto/ports/scout_batch_35/coci21c1p3.cp`
- `proto/ports/scout_batch_35/coci21c1p4.cp`
- `proto/ports/scout_batch_35/coci21c1p5.cp`
- `proto/ports/scout_batch_35/coci21c2p1.cp`
- `proto/ports/scout_batch_35/coci21c2p2.cp`
- `proto/ports/scout_batch_35/coci21c3p1.cp`
- `proto/ports/scout_batch_35/coci21c3p2.cp`
- `proto/ports/scout_batch_35/coci21c6p1.cp`
- `proto/ports/scout_batch_35/coci21c6p2.cp`
- `proto/ports/scout_batch_35/today/coi07p1.cp`
- `proto/ports/scout_batch_36/ccc02s4hard.cp`
- `proto/ports/scout_batch_36/ccc19s5hard.cp`
- `proto/ports/scout_batch_36/cco18p6.cp`
- `proto/ports/scout_batch_36/cco19p4.cp`
- `proto/ports/scout_batch_36/cco20p5.cp`
- `proto/ports/scout_batch_36/cco22p2.cp`
- `proto/ports/scout_batch_36/cco22p4.cp`
- `proto/ports/scout_batch_36/cco22p5.cp`
- `proto/ports/scout_batch_36/cco23p2.cp`
- `proto/ports/scout_batch_36/cco23p4.cp`
- `proto/ports/scout_batch_36/dream/cco20p5.cp`
- `proto/ports/scout_batch_37/ioi09p5.cp`
- `proto/ports/scout_batch_37/ioi12p5.cp`
- `proto/ports/scout_batch_37/ioi15p1.cp`
- `proto/ports/scout_batch_37/ioi20p2.cp`
- `proto/ports/scout_batch_37/noi10p1.cp`
- `proto/ports/scout_batch_37/noi14p5.cp`
- `proto/ports/scout_batch_37/noip04p4.cp`
- `proto/ports/scout_batch_37/noip06p3.cp`
- `proto/ports/scout_batch_37/noip10p4.cp`
- `proto/ports/scout_batch_37/noip18p4.cp`
- `proto/ports/scout_batch_38/apio09p1.cp`
- `proto/ports/scout_batch_38/apio09p2.cp`
- `proto/ports/scout_batch_38/btoi04p3.cp`
- `proto/ports/scout_batch_38/btoi08p6.cp`
- `proto/ports/scout_batch_38/btoi17p4.cp`
- `proto/ports/scout_batch_38/btoi18p4.cp`
- `proto/ports/scout_batch_38/dpf.cp`
- `proto/ports/scout_batch_38/egoi23p2.cp`
- `proto/ports/scout_batch_38/egoi23p4.cp`
- `proto/ports/scout_batch_38/egoi23p7.cp`
- `proto/ports/scout_batch_39/aac4p5.cp`
- `proto/ports/scout_batch_39/aac4p6.cp`
- `proto/ports/scout_batch_39/dmpg16g2.cp`
- `proto/ports/scout_batch_39/dmpg18s5.cp`
- `proto/ports/scout_batch_39/sac22cc4p4.cp`
- `proto/ports/scout_batch_39/sac22cc5p5.cp`
- `proto/ports/scout_batch_39/wac6p6.cp`
- `proto/ports/scout_batch_39/yac6p6.cp`
- `proto/ports/scout_batch_39/yac7p3.cp`
- `proto/ports/scout_batch_39/yac7p4.cp`
- `proto/ports/scout_batch_4/dpn.cp`
- `proto/ports/scout_batch_4/dps.cp`
- `proto/ports/scout_batch_40/cheerio1s1.cp`
- `proto/ports/scout_batch_40/cheerio1s2.cp`
- `proto/ports/scout_batch_40/cheerio1s5.cp`
- `proto/ports/scout_batch_40/tle17c5p1.cp`
- `proto/ports/scout_batch_40/tle18p5.cp`
- `proto/ports/scout_batch_41/nccc10s1.cp`
- `proto/ports/scout_batch_41/nccc8s1.cp`
- `proto/ports/scout_batch_41/nccc8s2.cp`
- `proto/ports/scout_batch_41/nccc8s4.cp`
- `proto/ports/scout_batch_41/nccc9s1.cp`
- `proto/ports/scout_batch_41/nccc9s2.cp`
- `proto/ports/scout_batch_41/nccc9s3.cp`
- `proto/ports/scout_batch_41/oly20practice139.cp`
- `proto/ports/scout_batch_41/oly23practice5.cp`
- `proto/ports/scout_batch_41/today/nccc9s4.cp`
- `proto/ports/scout_batch_42/mccc3j3.cp`
- `proto/ports/scout_batch_42/mccc3j4.cp`
- `proto/ports/scout_batch_42/mccc3j5.cp`
- `proto/ports/scout_batch_42/mccc4j3.cp`
- `proto/ports/scout_batch_42/mccc4j4.cp`
- `proto/ports/scout_batch_42/mccc5j3.cp`
- `proto/ports/scout_batch_42/mccc5j4.cp`
- `proto/ports/scout_batch_42/mccc5j5.cp`
- `proto/ports/scout_batch_42/mccc6s3.cp`
- `proto/ports/scout_batch_42/occ19g4.cp`
- `proto/ports/scout_batch_43/etsr23r1p3.cp`
- `proto/ports/scout_batch_43/etsr23r1p4.cp`
- `proto/ports/scout_batch_43/etsr23r2p2.cp`
- `proto/ports/scout_batch_43/etsr23r2p3.cp`
- `proto/ports/scout_batch_43/etsr23r2p4.cp`
- `proto/ports/scout_batch_43/etsr23r3p1.cp`
- `proto/ports/scout_batch_43/etsr23r3p3.cp`
- `proto/ports/scout_batch_43/etsr23r4p1.cp`
- `proto/ports/scout_batch_43/etsr23r4p3.cp`
- `proto/ports/scout_batch_43/today/etsr23r4p2.cp`
- `proto/ports/scout_batch_44/dmopc20c5p2.cp`
- `proto/ports/scout_batch_44/dmopc21c2p3.cp`
- `proto/ports/scout_batch_44/dmopc21c4p2.cp`
- `proto/ports/scout_batch_44/dmopc21c4p4.cp`
- `proto/ports/scout_batch_44/dmopc21c5p3.cp`
- `proto/ports/scout_batch_44/dmopc21c6p2.cp`
- `proto/ports/scout_batch_44/dmopc21c7p2.cp`
- `proto/ports/scout_batch_44/dmopc21c8p2.cp`
- `proto/ports/scout_batch_44/dmopc21c9p2.cp`
- `proto/ports/scout_batch_44/dmopc21c9p3.cp`
- `proto/ports/scout_batch_45/iati21a12.cp`
- `proto/ports/scout_batch_45/iati21a23.cp`
- `proto/ports/scout_batch_45/neerc10k.cp`
- `proto/ports/scout_batch_45/noimps22p5.cp`
- `proto/ports/scout_batch_45/rpeng2.cp`
- `proto/ports/scout_batch_45/shoi15p2.cp`
- `proto/ports/scout_batch_45/shoi16p1.cp`
- `proto/ports/scout_batch_45/shoi16p2.cp`
- `proto/ports/scout_batch_45/shoi16p5.cp`
- `proto/ports/scout_batch_45/ucrpc21h.cp`
- `proto/ports/scout_batch_46/aac4p4.cp`
- `proto/ports/scout_batch_46/aac5p4.cp`
- `proto/ports/scout_batch_46/sac22cc2p3.cp`
- `proto/ports/scout_batch_46/sac22cc2p5.cp`
- `proto/ports/scout_batch_46/sac22cc3p3.cp`
- `proto/ports/scout_batch_46/yac1p2.cp`
- `proto/ports/scout_batch_46/yac1p3.cp`
- `proto/ports/scout_batch_46/yac6p4.cp`
- `proto/ports/scout_batch_46/yac6p5.cp`
- `proto/ports/scout_batch_46/yac7p2.cp`
- `proto/ports/scout_batch_47/ccc18s2.cp`
- `proto/ports/scout_batch_47/ccc20s2hard.cp`
- `proto/ports/scout_batch_47/ccc22s2.cp`
- `proto/ports/scout_batch_47/cco01p4io.cp`
- `proto/ports/scout_batch_47/cco04p1.cp`
- `proto/ports/scout_batch_47/cco07p5.cp`
- `proto/ports/scout_batch_47/cco23p5.cp`
- `proto/ports/scout_batch_47/cco24p2.cp`
- `proto/ports/scout_batch_47/cco24p4.cp`
- `proto/ports/scout_batch_47/cco24p5.cp`
- `proto/ports/scout_batch_48/btoi06p5.cp`
- `proto/ports/scout_batch_48/ceoi19pp1.cp`
- `proto/ports/scout_batch_48/ceoi19pp2.cp`
- `proto/ports/scout_batch_48/egoi22p1.cp`
- `proto/ports/scout_batch_48/egoi22p5.cp`
- `proto/ports/scout_batch_48/egoi22p8.cp`
- `proto/ports/scout_batch_48/egoi23p1.cp`
- `proto/ports/scout_batch_48/egoi23p5.cp`
- `proto/ports/scout_batch_48/egoi23p6.cp`
- `proto/ports/scout_batch_48/egoi23p8.cp`
- `proto/ports/scout_batch_49/ioi00p2.cp`
- `proto/ports/scout_batch_49/ioi01p3.cp`
- `proto/ports/scout_batch_49/ioi12p2.cp`
- `proto/ports/scout_batch_49/ioi14p4.cp`
- `proto/ports/scout_batch_49/ioi15p5.cp`
- `proto/ports/scout_batch_49/ioi17p2.cp`
- `proto/ports/scout_batch_49/ioi18p2.cp`
- `proto/ports/scout_batch_49/ioi21p1.cp`
- `proto/ports/scout_batch_49/ioi22p4.cp`
- `proto/ports/scout_batch_49/ioi99p6.cp`
- `proto/ports/scout_batch_5/btoi17p1.cp`
- `proto/ports/scout_batch_5/cco18p4.cp`
- `proto/ports/scout_batch_5/cco22p1.cp`
- `proto/ports/scout_batch_5/cco23p6.cp`
- `proto/ports/scout_batch_5/ioi06p2.cp`
- `proto/ports/scout_batch_5/ioi13p5.cp`
- `proto/ports/scout_batch_5/noi06p4.cp`
- `proto/ports/scout_batch_5/noip10p3.cp`
- `proto/ports/scout_batch_5/today/noip10p3.cp`
- `proto/ports/scout_batch_5/yac6p3.cp`
- `proto/ports/scout_batch_50/noi03p4.cp`
- `proto/ports/scout_batch_50/noi04p5.cp`
- `proto/ports/scout_batch_50/noi17p1.cp`
- `proto/ports/scout_batch_50/noi20p5.cp`
- `proto/ports/scout_batch_50/noi22p1.cp`
- `proto/ports/scout_batch_50/noi22p4.cp`
- `proto/ports/scout_batch_50/noi23p1.cp`
- `proto/ports/scout_batch_50/noip02p4.cp`
- `proto/ports/scout_batch_50/noip08p4.cp`
- `proto/ports/scout_batch_50/noip22p2.cp`
- `proto/ports/scout_batch_51/apio23p2.cp`
- `proto/ports/scout_batch_51/ccc20s1.cp`
- `proto/ports/scout_batch_51/ccc23s1.cp`
- `proto/ports/scout_batch_51/ccc23s2.cp`
- `proto/ports/scout_batch_51/ccc96s3.cp`
- `proto/ports/scout_batch_51/cco03p4.cp`
- `proto/ports/scout_batch_51/cco18p2.cp`
- `proto/ports/scout_batch_51/egoi22p3.cp`
- `proto/ports/scout_batch_51/egoi22p4.cp`
- `proto/ports/scout_batch_51/egoi22p7.cp`
- `proto/ports/scout_batch_52/ctsc04p4.cp`
- `proto/ports/scout_batch_52/iati21a13.cp`
- `proto/ports/scout_batch_52/ioi13p3.cp`
- `proto/ports/scout_batch_52/ioi14p1.cp`
- `proto/ports/scout_batch_52/ioi14p6io.cp`
- `proto/ports/scout_batch_52/ioi17p4.cp`
- `proto/ports/scout_batch_52/ioi20p5.cp`
- `proto/ports/scout_batch_52/ioi22p5.cp`
- `proto/ports/scout_batch_52/noi02p2.cp`
- `proto/ports/scout_batch_52/noi02p6.cp`
- `proto/ports/scout_batch_52/noi08p5.cp`
- `proto/ports/scout_batch_52/noi19p3.cp`
- `proto/ports/scout_batch_52/noi21p5.cp`
- `proto/ports/scout_batch_52/noi23p4.cp`
- `proto/ports/scout_batch_52/noip20p3.cp`
- `proto/ports/scout_batch_6/ccc21s3.cp`
- `proto/ports/scout_batch_6/cco04p2.cp`
- `proto/ports/scout_batch_6/cco17p5.cp`
- `proto/ports/scout_batch_6/coci11c4p4.cp`
- `proto/ports/scout_batch_6/dmopc20c4p3.cp`
- `proto/ports/scout_batch_6/dmopc20c7p1.cp`
- `proto/ports/scout_batch_6/dmopc21c8p1.cp`
- `proto/ports/scout_batch_6/etsr23r1p2.cp`
- `proto/ports/scout_batch_6/joi16fp2.cp`
- `proto/ports/scout_batch_6/nccc8s3.cp`
- `proto/ports/scout_batch_8/dpy.cp`
- `proto/ports/scout_batch_9/btoi19p3.cp`
- `proto/ports/wave2/ccc18s4.cp`
- `proto/ports/wave2/ccc20s2.cp`
- `proto/ports/wave2/dpk.cp`
- `proto/ports/wave2/dpl.cp`
- `proto/ports/wave2/dpm.cp`
- `proto/ports/wave2/dpn.cp`
- `proto/ports/wave2/dpo.cp`
- `proto/ports/wave2/dpr.cp`
- `proto/ports/wave2/dpt.cp`
- `proto/ports/wave2/dpu.cp`
- `proto/ports/wave2/dpv.cp`
- `proto/ports/wave2/dpx.cp`
- `proto/ports/wave2/dpy.cp`
- `proto/ports/wave2/ioi00p1.cp`
- `proto/ports/wave2/ioi04p4.cp`
- `proto/ports/wave2/ioi94p1.cp`
- `proto/ports/wave2/noip00p1.cp`
- `proto/ports/wave2/noip01p2.cp`
- `proto/ports/wave2/noip04p1.cp`
- `proto/ports/wave2/noip04p3.cp`
- `proto/ports/wave2/noip06p2.cp`
- `proto/ports/wave2/noip14p2.cp`
- `proto/ports/wave2/noip14p5.cp`
- `proto/ports/wave2/noip18p2.cp`
- `proto/ports/wave2/yac1p1.cp`
- `proto/ports/yac6p3.cp`

### Duplicate C++ / .strip copy (original already in corpus/submissions/ or corpus/problems/<id>/orig.cpp) (289 files)

- `proto/ports/aac4p2.cpp`
- `proto/ports/aac4p2.strip`
- `proto/ports/aac4p2_orig.cpp`
- `proto/ports/aac5p2.cpp`
- `proto/ports/aac5p2.strip`
- `proto/ports/aac5p2_orig.cpp`
- `proto/ports/apio07p1.cpp`
- `proto/ports/apio07p1.strip`
- `proto/ports/apio07p1_orig.cpp`
- `proto/ports/btoi17p3.cpp`
- `proto/ports/btoi17p3.strip`
- `proto/ports/btoi17p3_orig.cpp`
- `proto/ports/btoi17p6.cpp`
- `proto/ports/btoi17p6.strip`
- `proto/ports/btoi17p6_orig.cpp`
- `proto/ports/btoi18p6.cpp`
- `proto/ports/btoi18p6.strip`
- `proto/ports/btoi18p6_orig.cpp`
- `proto/ports/ccc03s4.cpp`
- `proto/ports/ccc03s4.strip`
- `proto/ports/ccc03s4_orig.cpp`
- `proto/ports/ccc10s4.cpp`
- `proto/ports/ccc10s4.strip`
- `proto/ports/ccc10s4_orig.cpp`
- `proto/ports/ccc17s4.cpp`
- `proto/ports/ccc17s4.strip`
- `proto/ports/ccc17s4_orig.cpp`
- `proto/ports/ccc18s4.cpp`
- `proto/ports/ccc18s4.strip`
- `proto/ports/ccc18s4_orig.cpp`
- `proto/ports/ccc18s5.cpp`
- `proto/ports/ccc18s5.strip`
- `proto/ports/ccc18s5_orig.cpp`
- `proto/ports/ccc19s5.cpp`
- `proto/ports/ccc19s5.strip`
- `proto/ports/ccc19s5_orig.cpp`
- `proto/ports/ccc20s2.cpp`
- `proto/ports/ccc20s2.strip`
- `proto/ports/ccc20s2_orig.cpp`
- `proto/ports/ccc20s3.cpp`
- `proto/ports/ccc20s3.strip`
- `proto/ports/ccc20s3_orig.cpp`
- `proto/ports/ccc21s2.cpp`
- `proto/ports/ccc21s2.strip`
- `proto/ports/ccc21s2_orig.cpp`
- `proto/ports/ccc21s3.cpp`
- `proto/ports/ccc21s3.strip`
- `proto/ports/ccc21s3_orig.cpp`
- `proto/ports/ccc21s4.cpp`
- `proto/ports/ccc21s4.strip`
- `proto/ports/ccc21s4_orig.cpp`
- `proto/ports/ccc21s5.cpp`
- `proto/ports/ccc21s5.strip`
- `proto/ports/ccc21s5_orig.cpp`
- `proto/ports/ccc22s4.cpp`
- `proto/ports/ccc22s4.strip`
- `proto/ports/ccc22s4_orig.cpp`
- `proto/ports/ccc23s4.cpp`
- `proto/ports/ccc23s4.strip`
- `proto/ports/ccc23s4_orig.cpp`
- `proto/ports/ccc23s5.cpp`
- `proto/ports/ccc23s5.strip`
- `proto/ports/ccc23s5_orig.cpp`
- `proto/ports/cco16p2.cpp`
- `proto/ports/cco16p2.strip`
- `proto/ports/cco16p2_orig.cpp`
- `proto/ports/cco17p1.cpp`
- `proto/ports/cco17p1.strip`
- `proto/ports/cco17p1_orig.cpp`
- `proto/ports/cco18p1.cpp`
- `proto/ports/cco18p1.strip`
- `proto/ports/cco18p1_orig.cpp`
- `proto/ports/cco20p1.cpp`
- `proto/ports/cco20p1.strip`
- `proto/ports/cco20p1_orig.cpp`
- `proto/ports/cco24p1.cpp`
- `proto/ports/cco24p1.strip`
- `proto/ports/cco24p1_orig.cpp`
- `proto/ports/ceoi09p3.cpp`
- `proto/ports/ceoi09p3.strip`
- `proto/ports/ceoi09p3_orig.cpp`
- `proto/ports/ceoi18p1.cpp`
- `proto/ports/ceoi18p1.strip`
- `proto/ports/ceoi18p1_orig.cpp`
- `proto/ports/cheerio1s3.cpp`
- `proto/ports/cheerio1s3.strip`
- `proto/ports/cheerio1s3_orig.cpp`
- `proto/ports/coci13c1p3.cpp`
- `proto/ports/coci13c1p3.strip`
- `proto/ports/coci13c1p3_orig.cpp`
- `proto/ports/coci20c6p3.cpp`
- `proto/ports/coci20c6p3.strip`
- `proto/ports/coci20c6p3_orig.cpp`
- `proto/ports/dmopc21c2p1.cpp`
- `proto/ports/dmopc21c2p1.strip`
- `proto/ports/dmopc21c2p1_orig.cpp`
- `proto/ports/dmopc21c9p4.cpp`
- `proto/ports/dmopc21c9p4.strip`
- `proto/ports/dmopc21c9p4_orig.cpp`
- `proto/ports/dmpg16s5.cpp`
- `proto/ports/dmpg16s5.strip`
- `proto/ports/dmpg16s5_orig.cpp`
- `proto/ports/dpe.cpp`
- `proto/ports/dpe.strip`
- `proto/ports/dpe_orig.cpp`
- `proto/ports/dpg.cpp`
- `proto/ports/dpg.strip`
- `proto/ports/dpg_orig.cpp`
- `proto/ports/dph.cpp`
- `proto/ports/dph.strip`
- `proto/ports/dph_orig.cpp`
- `proto/ports/dpk.cpp`
- `proto/ports/dpk.strip`
- `proto/ports/dpk_orig.cpp`
- `proto/ports/dpl.cpp`
- `proto/ports/dpl.strip`
- `proto/ports/dpl_orig.cpp`
- `proto/ports/dpm.cpp`
- `proto/ports/dpm.strip`
- `proto/ports/dpm_orig.cpp`
- `proto/ports/dpn.cpp`
- `proto/ports/dpn.strip`
- `proto/ports/dpn_orig.cpp`
- `proto/ports/dpr.cpp`
- `proto/ports/dpr.strip`
- `proto/ports/dpr_orig.cpp`
- `proto/ports/dpt.cpp`
- `proto/ports/dpt.strip`
- `proto/ports/dpt_orig.cpp`
- `proto/ports/dpx.cpp`
- `proto/ports/dpx.strip`
- `proto/ports/dpx_orig.cpp`
- `proto/ports/dpy.cpp`
- `proto/ports/dpy.strip`
- `proto/ports/dpy_orig.cpp`
- `proto/ports/dpz.cpp`
- `proto/ports/dpz.strip`
- `proto/ports/dpz_orig.cpp`
- `proto/ports/hillclimb_dp_graph/apio07p3.cpp`
- `proto/ports/hillclimb_dp_graph/apio07p3.strip`
- `proto/ports/hillclimb_dp_graph/apio07p3_orig.cpp`
- `proto/ports/hillclimb_dp_graph/apio09p3.cpp`
- `proto/ports/hillclimb_dp_graph/apio09p3.strip`
- `proto/ports/hillclimb_dp_graph/apio09p3_orig.cpp`
- `proto/ports/hillclimb_dp_graph/ccc22s3.cpp`
- `proto/ports/hillclimb_dp_graph/ccc22s3.strip`
- `proto/ports/hillclimb_dp_graph/ccc22s3_orig.cpp`
- `proto/ports/hillclimb_dp_graph/coi06p2.cpp`
- `proto/ports/hillclimb_dp_graph/coi06p2.strip`
- `proto/ports/hillclimb_dp_graph/coi06p2_orig.cpp`
- `proto/ports/hillclimb_dp_graph/dmopc21c5p1.cpp`
- `proto/ports/hillclimb_dp_graph/dmopc21c5p1.strip`
- `proto/ports/hillclimb_dp_graph/dmopc21c5p1_orig.cpp`
- `proto/ports/hillclimb_dp_graph/dpo.cpp`
- `proto/ports/hillclimb_dp_graph/dpo.strip`
- `proto/ports/hillclimb_dp_graph/dpo_orig.cpp`
- `proto/ports/hillclimb_dp_graph/dpp.cpp`
- `proto/ports/hillclimb_dp_graph/dpp.strip`
- `proto/ports/hillclimb_dp_graph/dpp_orig.cpp`
- `proto/ports/hillclimb_dp_graph/dps.cpp`
- `proto/ports/hillclimb_dp_graph/dps.strip`
- `proto/ports/hillclimb_dp_graph/dps_orig.cpp`
- `proto/ports/hillclimb_dp_graph/dpu.cpp`
- `proto/ports/hillclimb_dp_graph/dpu.strip`
- `proto/ports/hillclimb_dp_graph/dpu_orig.cpp`
- `proto/ports/hillclimb_dp_graph/dpv.cpp`
- `proto/ports/hillclimb_dp_graph/dpv.strip`
- `proto/ports/hillclimb_dp_graph/dpv_orig.cpp`
- `proto/ports/hillclimb_dp_graph/ioi11p4.cpp`
- `proto/ports/hillclimb_dp_graph/ioi11p4.strip`
- `proto/ports/hillclimb_dp_graph/ioi11p4_orig.cpp`
- `proto/ports/hillclimb_dp_graph/noi15p2.cpp`
- `proto/ports/hillclimb_dp_graph/noi15p2.strip`
- `proto/ports/hillclimb_dp_graph/noi15p2_orig.cpp`
- `proto/ports/hillclimb_dp_graph/noip06p1.cpp`
- `proto/ports/hillclimb_dp_graph/noip06p1.strip`
- `proto/ports/hillclimb_dp_graph/noip06p1_orig.cpp`
- `proto/ports/hillclimb_dp_graph/sac22cc3p4.cpp`
- `proto/ports/hillclimb_dp_graph/sac22cc3p4.strip`
- `proto/ports/hillclimb_dp_graph/sac22cc3p4_orig.cpp`
- `proto/ports/hillclimb_dp_graph/yac6p2.cpp`
- `proto/ports/hillclimb_dp_graph/yac6p2.strip`
- `proto/ports/hillclimb_dp_graph/yac6p2_orig.cpp`
- `proto/ports/ioi00p1.cpp`
- `proto/ports/ioi00p1.strip`
- `proto/ports/ioi00p1_orig.cpp`
- `proto/ports/ioi04p4.cpp`
- `proto/ports/ioi04p4.strip`
- `proto/ports/ioi04p4_orig.cpp`
- `proto/ports/ioi05p6.cpp`
- `proto/ports/ioi05p6.strip`
- `proto/ports/ioi05p6_orig.cpp`
- `proto/ports/ioi10p3.cpp`
- `proto/ports/ioi10p3.strip`
- `proto/ports/ioi11p2.cpp`
- `proto/ports/ioi11p2.strip`
- `proto/ports/ioi94p1.cpp`
- `proto/ports/ioi94p1.strip`
- `proto/ports/ioi94p1_orig.cpp`
- `proto/ports/mccc4j5.cpp`
- `proto/ports/mccc4j5.strip`
- `proto/ports/mccc4j5_orig.cpp`
- `proto/ports/nccc10s2.cpp`
- `proto/ports/nccc10s2.strip`
- `proto/ports/nccc10s2_orig.cpp`
- `proto/ports/noi01p5.cpp`
- `proto/ports/noi01p5.strip`
- `proto/ports/noi01p5_orig.cpp`
- `proto/ports/noi02p1.cpp`
- `proto/ports/noi02p1.strip`
- `proto/ports/noi02p1_orig.cpp`
- `proto/ports/noi14p4.cpp`
- `proto/ports/noi14p4.strip`
- `proto/ports/noi14p4_orig.cpp`
- `proto/ports/noi16p4.cpp`
- `proto/ports/noi16p4.strip`
- `proto/ports/noi16p4_orig.cpp`
- `proto/ports/noip00p1.cpp`
- `proto/ports/noip00p1.strip`
- `proto/ports/noip00p1_orig.cpp`
- `proto/ports/noip01p2.cpp`
- `proto/ports/noip01p2.strip`
- `proto/ports/noip01p2_orig.cpp`
- `proto/ports/noip04p1.cpp`
- `proto/ports/noip04p1.strip`
- `proto/ports/noip04p1_orig.cpp`
- `proto/ports/noip04p2.cpp`
- `proto/ports/noip04p2.strip`
- `proto/ports/noip04p2_orig.cpp`
- `proto/ports/noip04p3.cpp`
- `proto/ports/noip04p3.strip`
- `proto/ports/noip04p3_orig.cpp`
- `proto/ports/noip06p2.cpp`
- `proto/ports/noip06p2.strip`
- `proto/ports/noip06p2_orig.cpp`
- `proto/ports/noip07p1.cpp`
- `proto/ports/noip07p1.strip`
- `proto/ports/noip07p1_orig.cpp`
- `proto/ports/noip07p2.cpp`
- `proto/ports/noip07p2.strip`
- `proto/ports/noip07p2_orig.cpp`
- `proto/ports/noip09p2.cpp`
- `proto/ports/noip09p2.strip`
- `proto/ports/noip09p2_orig.cpp`
- `proto/ports/noip09p3.cpp`
- `proto/ports/noip09p3.strip`
- `proto/ports/noip09p3_orig.cpp`
- `proto/ports/noip10p3.cpp`
- `proto/ports/noip10p3.strip`
- `proto/ports/noip10p3_orig.cpp`
- `proto/ports/noip14p2.cpp`
- `proto/ports/noip14p2.strip`
- `proto/ports/noip14p2_orig.cpp`
- `proto/ports/noip14p3.cpp`
- `proto/ports/noip14p3.strip`
- `proto/ports/noip14p3_orig.cpp`
- `proto/ports/noip14p5.cpp`
- `proto/ports/noip14p5.strip`
- `proto/ports/noip14p5_orig.cpp`
- `proto/ports/noip18p2.cpp`
- `proto/ports/noip18p2.strip`
- `proto/ports/noip18p2_orig.cpp`
- `proto/ports/scout_batch_17/harness/ioi12p4_main.cpp`
- `proto/ports/scout_batch_17/harness/ioi13p1_main.cpp`
- `proto/ports/scout_batch_35/coci21c1p1.strip`
- `proto/ports/scout_batch_35/coci21c1p3.strip`
- `proto/ports/scout_batch_35/coci21c1p4.strip`
- `proto/ports/scout_batch_35/coci21c1p5.strip`
- `proto/ports/scout_batch_35/coci21c2p1.strip`
- `proto/ports/scout_batch_35/coci21c2p2.strip`
- `proto/ports/scout_batch_35/coci21c3p1.strip`
- `proto/ports/scout_batch_35/coci21c3p2.strip`
- `proto/ports/scout_batch_35/coci21c6p1.strip`
- `proto/ports/scout_batch_35/coci21c6p2.strip`
- `proto/ports/scout_batch_35/coi07p1.strip`
- `proto/ports/scout_batch_9/harness/cmt21r2ztr_check.cpp`
- `proto/ports/scout_batch_9/harness/ioi10p6.cpp`
- `proto/ports/scout_batch_9/harness/ioi19p2.cpp`
- `proto/ports/scout_batch_9/harness/ioi19p2_check.cpp`
- `proto/ports/scout_batch_9/harness/ioi20p6.cpp`
- `proto/ports/yac1p1.cpp`
- `proto/ports/yac1p1.strip`
- `proto/ports/yac1p1_orig.cpp`
- `proto/ports/yac6p3.cpp`
- `proto/ports/yac6p3.strip`
- `proto/ports/yac6p3_orig.cpp`
- `proto/ports/yac7p5.cpp`
- `proto/ports/yac7p5.strip`
- `proto/ports/yac7p5_orig.cpp`

### Legacy proto/ports Python batch harness/helper replaced by corpus/problems/<id>/{gen,checker,interactor}.py (62 files)

- `proto/ports/aac5p2_judge.py`
- `proto/ports/scout_batch_10/check.py`
- `proto/ports/scout_batch_11/check.py`
- `proto/ports/scout_batch_12/check.py`
- `proto/ports/scout_batch_13/today/cpc21c1p5_judge.py`
- `proto/ports/scout_batch_14/check.py`
- `proto/ports/scout_batch_14/today/stress.py`
- `proto/ports/scout_batch_15/check.py`
- `proto/ports/scout_batch_16/check.py`
- `proto/ports/scout_batch_18/check.py`
- `proto/ports/scout_batch_19/verify.py`
- `proto/ports/scout_batch_2/check.py`
- `proto/ports/scout_batch_20/check.py`
- `proto/ports/scout_batch_21/check.py`
- `proto/ports/scout_batch_22/check.py`
- `proto/ports/scout_batch_23/check.py`
- `proto/ports/scout_batch_24/check.py`
- `proto/ports/scout_batch_25/verify.py`
- `proto/ports/scout_batch_26/check.py`
- `proto/ports/scout_batch_27/check.py`
- `proto/ports/scout_batch_28/check.py`
- `proto/ports/scout_batch_29/check.py`
- `proto/ports/scout_batch_3/check.py`
- `proto/ports/scout_batch_3/today/cco09p4_judge.py`
- `proto/ports/scout_batch_30/check.py`
- `proto/ports/scout_batch_31/check.py`
- `proto/ports/scout_batch_32/check.py`
- `proto/ports/scout_batch_33/check.py`
- `proto/ports/scout_batch_34/check.py`
- `proto/ports/scout_batch_35/check.py`
- `proto/ports/scout_batch_35/loc.py`
- `proto/ports/scout_batch_36/verify.py`
- `proto/ports/scout_batch_37/check.py`
- `proto/ports/scout_batch_37/loc.py`
- `proto/ports/scout_batch_38/check.py`
- `proto/ports/scout_batch_39/check.py`
- `proto/ports/scout_batch_40/check.py`
- `proto/ports/scout_batch_40/loc.py`
- `proto/ports/scout_batch_41/check.py`
- `proto/ports/scout_batch_41/loc.py`
- `proto/ports/scout_batch_42/check.py`
- `proto/ports/scout_batch_42/loc.py`
- `proto/ports/scout_batch_43/check.py`
- `proto/ports/scout_batch_44/check.py`
- `proto/ports/scout_batch_45/check.py`
- `proto/ports/scout_batch_45/loc.py`
- `proto/ports/scout_batch_46/check.py`
- `proto/ports/scout_batch_47/verify.py`
- `proto/ports/scout_batch_48/check.py`
- `proto/ports/scout_batch_49/check.py`
- `proto/ports/scout_batch_49/loc.py`
- `proto/ports/scout_batch_5/feed.py`
- `proto/ports/scout_batch_5/today/cco18p4_judge.py`
- `proto/ports/scout_batch_5/today/yac6p3_judge.py`
- `proto/ports/scout_batch_50/check.py`
- `proto/ports/scout_batch_50/loc.py`
- `proto/ports/scout_batch_51/check.py`
- `proto/ports/scout_batch_7/check.py`
- `proto/ports/scout_batch_8/check.py`
- `proto/ports/scout_batch_9/harness/judge.py`
- `proto/ports/yac6p3_judge.py`
- `proto/ports/yac7p5_judge.py`

### Legacy proto/ports batch runner/harness replaced by cpl and scripts/scoreboard.py (11 files)

- `proto/ports/ecnerwala/gate.sh`
- `proto/ports/scout_batch_13/check.sh`
- `proto/ports/scout_batch_17/stress.sh`
- `proto/ports/scout_batch_4/stress.sh`
- `proto/ports/scout_batch_5/check.sh`
- `proto/ports/scout_batch_52/test_batch_52.py`
- `proto/ports/scout_batch_6/check.sh`
- `proto/ports/scout_batch_6/today/stress.sh`
- `proto/ports/scout_batch_9/stress.sh`
- `proto/ports/wave1/gate.sh`
- `proto/ports/wave2/gate.sh`

### Legacy proto/ports batch metadata/notes superseded by corpus/problems/ and corpus/sets/ (8 files)

- `proto/ports/ecnerwala/lib.toml`
- `proto/ports/mccc4j5_max.txt`
- `proto/ports/scout_batch_10/lib.toml`
- `proto/ports/scout_batch_15/lib.toml`
- `proto/ports/scout_batch_21/lib.toml`
- `proto/ports/scout_batch_22/lib.toml`
- `proto/ports/scout_batch_29/lib.toml`
- `proto/ports/sim3/cp.toml`

### Legacy proto/ports artifact superseded by corpus/problems/ (1 files)

- `proto/ports/ecnerwala/math`

## Old documentation migration (`design/`, `plan/`, `archive/`)

Migrated/deleted 102 files from `design/`, `plan/`, and `archive/` per `plan/RESTRUCTURE.md`.

| Old path | Destination | Notes |
|---|---|---|
| `design/sketches/ioi2025/festival.cp` | `corpus/problems/ioi25_festival/dream.cp` | IOI 2025 gold sketch moved to per-problem dream.cp |
| `design/sketches/ioi2025/migrations.cp` | `corpus/problems/ioi25_migrations/dream.cp` | IOI 2025 gold sketch moved to per-problem dream.cp |
| `design/sketches/ioi2025/obstacles.cp` | `corpus/problems/ioi25_obstacles/dream.cp` | IOI 2025 gold sketch moved to per-problem dream.cp |
| `design/sketches/ioi2025/souvenirs.cp` | `corpus/problems/ioi25_souvenirs/dream.cp` | IOI 2025 gold sketch moved to per-problem dream.cp |
| `design/sketches/ioi2025/triples.cp` | `corpus/problems/ioi25_triples/dream.cp` | IOI 2025 gold sketch moved to per-problem dream.cp |
| `design/sketches/ioi2025/worldmap.cp` | `corpus/problems/ioi25_worldmap/dream.cp` | IOI 2025 gold sketch moved to per-problem dream.cp |
| `design/sketches/ioi2025/README.md` | `research/design-history/ioi2025-sketches-README.md` | IOI 2025 sketches overview preserved verbatim |
| `design/COST.md` | `spec/core/13-lowering.md (§13.8)` | Cost tables, debt list, and enforcement rules migrated verbatim |
| `plan/80-validation.md` | `corpus/VALIDATION.md` | Corpus validation design moved verbatim |
| `plan/STDLIB.md` | `spec/std/README.md` | Full stdlib surface design migrated verbatim; cpl-check test repointed |
| `design/00-grounding.md` | `research/design-history/00-grounding.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/ACCEPT.md` | `research/design-history/ACCEPT.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/ALIGN.md` | `research/design-history/ALIGN.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/CORE-ALIGN.md` | `research/design-history/CORE-ALIGN.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/DELTA.md` | `research/design-history/DELTA.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/EXEMPLARS-PORTS.md` | `research/design-history/EXEMPLARS-PORTS.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/EXEMPLARS.md` | `research/design-history/EXEMPLARS.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/GOALS.md` | `research/design-history/GOALS.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/IDIOMS.md` | `research/design-history/IDIOMS.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/README.md` | `research/design-history/README.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/SHIP-PLAN.md` | `research/design-history/SHIP-PLAN.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/decisions-design.md` | `research/design-history/decisions-design.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/keyword-parameter-labels.md` | `research/design-history/keyword-parameter-labels.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/sketches/exemplars-11-15.md` | `research/design-history/sketches/exemplars-11-15.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/sketches/exemplars-16-25.md` | `research/design-history/sketches/exemplars-16-25.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/sketches/split-minimal-chunked.md` | `research/design-history/sketches/split-minimal-chunked.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/sketches/split-minimal.md` | `research/design-history/sketches/split-minimal.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/archive/LOG-v1.md` | `research/design-history/archive/LOG-v1.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/archive/SPEC-v1.md` | `research/design-history/archive/SPEC-v1.md` | Preserved verbatim in research/design-history/ (superseded/historical design doc) |
| `design/spec/00-layers.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/10-syntax.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/11-types.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/12-scoped-types.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/13-arrays.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/14-functions-traits.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/15-control.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/16-state.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/17-contexts.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/18-checks-lowering.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/20-intrinsics.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/21-index-spaces.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/22-sequences.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/23-optionals.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/24-modular.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/25-io.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/26-containers.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/27-algebra-segtrees.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/28-graphs-trees.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/29-strings.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/30-stress.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/31-interaction.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/32-graders-bundling.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/33-debug-trace.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/40-signatures.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/90-open-questions.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/v2/01-index-spaces.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/v2/02-contexts-and-markers.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/v2/03-sequences-and-optionals.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/v2/04-stress-and-interaction.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/v2/05-segment-trees.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/v2/06-functions-state-control.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/v2/07-numbers.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `design/spec/v2/08-strings-io-containers.md` | `spec/ (core/, lib/, tools/)` | Superseded by normative v0.3 spec/; git history preserves exact prior draft |
| `archive/style-decisions.md` | `research/design-history/style-decisions.md` | Preserved verbatim in research/design-history/; archive/ removed |
| `plan/00-vision.md` | `research/design-history/plan-v0.2/00-vision.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/10-language-tour.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/11-values-and-types.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/12-domains-and-sequences.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/13-abstraction.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/14-state.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/15-program-structure.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/16-io.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/17-syntax.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/18-key-features.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/19-index-and-lends.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/20-stdlib.md` | `spec/ (core/, lib/)` | 0.2.2 implementation chapter superseded by normative spec/; git history preserves exact text |
| `plan/30-compiler.md` | `research/design-history/plan-v0.2/30-compiler.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/40-tooling.md` | `research/design-history/plan-v0.2/40-tooling.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/45-dev-tools.md` | `research/design-history/plan-v0.2/45-dev-tools.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/50-debugging.md` | `research/design-history/plan-v0.2/50-debugging.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/60-editor.md` | `research/design-history/plan-v0.2/60-editor.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/70-judges.md` | `research/design-history/plan-v0.2/70-judges.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/90-docs-and-playground.md` | `research/design-history/plan-v0.2/90-docs-and-playground.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/DREAM-BRIEF.md` | `research/design-history/plan-v0.2/DREAM-BRIEF.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/EXECUTION.md` | `research/design-history/plan-v0.2/EXECUTION.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/README.md` | `research/design-history/plan-v0.2/README.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/a0-migration.md` | `research/design-history/plan-v0.2/a0-migration.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/b0-distribution.md` | `research/design-history/plan-v0.2/b0-distribution.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/c0-llm.md` | `research/design-history/plan-v0.2/c0-llm.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/d0-roadmap.md` | `research/design-history/plan-v0.2/d0-roadmap.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/00-overview.md` | `research/design-history/plan-v0.2/site/00-overview.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/01-journey.md` | `research/design-history/plan-v0.2/site/01-journey.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/10-landing.md` | `research/design-history/plan-v0.2/site/10-landing.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/20-onboarding.md` | `research/design-history/plan-v0.2/site/20-onboarding.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/30-course.md` | `research/design-history/plan-v0.2/site/30-course.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/40-playground.md` | `research/design-history/plan-v0.2/site/40-playground.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/50-docs.md` | `research/design-history/plan-v0.2/site/50-docs.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/60-tools-setup.md` | `research/design-history/plan-v0.2/site/60-tools-setup.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/70-project.md` | `research/design-history/plan-v0.2/site/70-project.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/80-design-system.md` | `research/design-history/plan-v0.2/site/80-design-system.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/90-build.md` | `research/design-history/plan-v0.2/site/90-build.md` | Preserved verbatim in research/design-history/plan-v0.2/ |
| `plan/site/README.md` | `research/design-history/plan-v0.2/site/README.md` | Preserved verbatim in research/design-history/plan-v0.2/ |

## Replaced by `crates/cpl-conform`

- `scripts/conformance.py` — replaced by `crates/cpl-conform` (`conformance` suite, matching 227/575 met)
- `scripts/corpus-tests.py` (`run` subcommand) — replaced by `crates/cpl-conform` (`corpus` suite, matching 2/54 tested pass; `make` subcommand retained for test generation)
- `scripts/scoreboard.py` — replaced by `crates/cpl-conform` unified ratcheted scoreboard (`research/scoreboard.json`)
- `scripts/test_scoreboard.py` — unit tests for deleted `scripts/scoreboard.py`; ratchet unit tests now in `crates/cpl-conform/tests/ratchet.rs`
- `research/scoreboard.ports.jsonl` — legacy per-port JSONL sidecar for deleted `scripts/scoreboard.py`
- `scripts/fix-spelling-drift.py` — obsolete v0.2 codemod hardcoded to deleted `proto/ports/`; superseded by `cpl lint --fix`
