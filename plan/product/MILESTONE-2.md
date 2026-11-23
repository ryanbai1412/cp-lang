<!-- Ryan's product doc. Change only with Ryan's say-so (propose in plan/QUESTIONS.md). -->

# Milestone 2 — Stable release

**CPL is ready to release to the world as stable, for anyone to try.** This is the maximally ambitious version: everything in [VISION.md](VISION.md) and [EXPERIENCE.md](EXPERIENCE.md), everything [MILESTONE-1.md](MILESTONE-1.md) deferred, and whatever else each domain judges the complete product needs.

A domain moves onto milestone 2 as soon as its milestone 1 exit criteria pass. Its README has a "Milestone 2" section with that domain's most ambitious complete version, its checkable exit criteria and its next steps.

## Done means

- **Every journey, J1–J10, is complete and polished** for strangers, not just friends: discovery, learning (beginner through in-depth), install, contests, writing and revising, testing, debugging, performance, submission, and managing libraries and old solutions.
- **Every domain hits its north-star targets** in `plan/domains/*/metrics.toml`. Examples:
  - interpreter within 5x of C++, trace overhead at most +20%;
  - runtime p50 under 1.1x, p95 under 1.2x and max under 1.5x of matched C++ for idiomatic code, and max under 1.1x once the user tunes in the language (widths, `unchecked:`, layout);
  - every solution under 32 KB;
  - std within 1.2x of optimized C++ on Library Checker;
  - the full lint catalog;
  - spec code blocks and doc examples all passing.
- **Everything deferred from milestone 1:**
  - time-travel debugging and trace viewing (milestone 1.5);
  - generators and shrinking for arbitrary input structures;
  - static performance analysis;
  - comprehensive standard library;
  - native editor UIs (VS Code, JetBrains) beyond the LSP;
  - personal libraries with stdlib powers;
  - old solutions that rebuild byte for byte, with pins and codemods.
- **Stable release:** two channels, `stable` and `edge`. `stable` is promoted by hand with release notes (v0.3.0 first); while the version is 0.x the site and README label it beta. Signed installers and package managers on Linux, macOS and Windows, a changelog, and a documented compatibility promise.
- **No target date yet.**
- **Public quality:**
  - no known correctness bugs in supported features;
  - interpreter and C++ agree on the whole corpus;
  - CI under 3 minutes and green;
  - a site and docs that stand on their own.
