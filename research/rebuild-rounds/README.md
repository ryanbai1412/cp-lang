# Rebuild rounds

Each file is one cold-reader rebuild's `HOLES.md`: an agent reads only `spec/` and `std/`, solves ten unseen problems, and records every place the spec was silent, contradictory or missing a std item. The triage of each round is in `decisions.md` and its cases in `spec/conformance/`; the solutions themselves are in git history under `plan/domains/spec/delta/rebuild-N/`.
