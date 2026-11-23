<!-- Ryan's product doc. Frozen after the 2026-11-22 edit pass; change only with Ryan's say-so (propose in plan/QUESTIONS.md). -->

# Milestone 1 — Friend-testable, contest-capable CPL

**I can share CPL with an interested competitive programmer, and they can immediately start learning it, solving real problems, and competing with it.**

This is more than a proof of concept, a toy interpreter or a demo website. The friend uses CPL on their own for real competitive programming, including a real or realistically timed contest. The whole vision need not be done, but the supported end-to-end experience must be complete, polished enough to judge fairly, and trustworthy.

## Constraints (Ryan, 2026-11-22)

- **Due:** about 12 hours, by 2026-11-23 09:30 PST. Plan in hours; checkpoints every 2 hours.
- **Platforms:** Linux, macOS and Windows, all supported and all tested.
- **Editors:** Neovim/Vim (the first friend's), VS Code and JetBrains, each with a working LSP.
- **Judge:** Codeforces. We check submissions against a local Codeforces profile; agents never submit to real judges.
- **Channel:** the edge release is enough. No stable release needed.
- Deferred work continues in parallel, but milestone blockers come first.

## The acceptance scenario

A Codeforces 1500+ friend who has never used CPL gets a link. Without help, they can:

1. **Understand:** open the website, see the value proposition, and read a concise introductory guide.
2. **Try:** run and modify CPL programs in the browser, completing a few introductory exercises.
3. **Install:** set up CPL and their editor integration on Linux, macOS or Windows.
4. **Solve:** implement several unfamiliar, real problems with the language and standard library.
5. **Test:** run samples, add custom tests, and diagnose ordinary mistakes from the CLI and the editor.
6. **Submit:** generate a Codeforces submission that gets accepted.
7. **Compete:** take part in a real or virtual Codeforces round with CPL as their main language, never blocked by missing ordinary functionality or a tool failure.

Afterwards they can give meaningful feedback on expressiveness, correctness mechanisms, tooling and usability, and they have felt at least one compelling advantage over their current workflow.

**Acceptance run:** an agent plays the friend on fresh Linux, macOS and Windows machines, using only the website and docs, every 2 hours. It records the steps passed (out of 7), the time per step, and every hesitation or workaround, and publishes a ranked blocker list. The idioms domain owns it.

## What must be ready

- **Language and compiler:** a coherent, documented subset that handles representative real problems, including nontrivial algorithms and data structures. It is correct and identical across interpreter and C++; gaps are listed, never disguised.
- **Standard library:** the foundational containers, algorithms, math and helpers. It need not be exhaustive, but ordinary solutions never require extensive workarounds.
- **CLI and local testing:** reliable check, run, sample tests, custom tests, checkers and judge-ready builds; sample import through Competitive Companion.
- **Editors:** Neovim/Vim, VS Code and JetBrains with highlighting, diagnostics, completion and formatting, so the user never switches between disconnected tools.
- **Website and learning:** a credible site, a concise intro guide, searchable docs, runnable examples that actually work, and enough practice to teach the core language model.
- **In-depth language chapters:** one per major part (ownership and mutation, domains and ranges, types and proofs, multitest, checked operations, the judge model), precise enough to predict behaviour, each explaining the design decisions behind it. The first friends are PL enthusiasts who will want to understand it fully.
- **Submission:** a reliable self-contained C++17 file for Codeforces. Single-test and multitest problems work; other formats work where feasible and are clearly scoped where not.
- **Debugging:** high-quality diagnostics and runtime errors, print debugging, assertions, and fast reproduction of failing cases. A basic integrated brute/stress workflow is the differentiator to show. Full time-travel debugging is not required.
- **Release quality:** reproducible builds, installers tested on all three platforms, automated regression checks, and no known serious correctness or submission bugs in supported workflows.

## What can wait

These don't block the milestone unless testing shows they are essential:

- Comprehensive advanced standard-library coverage.
- Automatic generation and shrinking for arbitrary input structures.
- Advanced static performance analysis and inline resource annotations.
- Direct submission to judges.
- A third-party package ecosystem.
- Long-range backward compatibility across language versions.
- Exhaustive beginner tutorials.
- Compiler optimization beyond competitive parity.

## Milestone 1.5

Debugger replay: time-travel through a run and view the trace, at most +20% over the interpreter. Work on it in parallel as soon as the milestone's stress loop works everywhere.

Scope is set by whether friends can realistically use CPL, not by whether every planned component is finished.
