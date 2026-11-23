<!-- Ryan's text, 2026-11-22. Edit only with Ryan's say-so; propose changes in plan/QUESTIONS.md. -->

# Milestone 1 — Friend-testable, contest-capable CPL

### The milestone

**I can share CPL with an interested competitive programmer, and they can immediately start learning it, solving real problems, and competing with it.**

This is stronger than a proof of concept, toy interpreter, or demo website.

The friend should be able to use CPL independently for real competitive programming, including an actual contest or realistic timed contest session.

We don't need the entire long-term product vision completed. But the supported end-to-end experience must be complete, polished enough to evaluate fairly, and trustworthy.

### 5.1. The acceptance scenario

Imagine sharing a link with a Codeforces 1500+ friend who has never used CPL.

They should be able to:

1. **Understand:** Open the website, understand the value proposition, and read a concise introductory guide.
2. **Try:** Run and modify CPL programs directly in the browser, completing a few introductory competitive programming exercises.
3. **Install:** Set up CPL and its editor integration on a supported machine without needing your help.
4. **Solve:** Implement several unfamiliar, real competitive programming problems using the language and standard library.
5. **Test:** Run samples, add custom tests, and diagnose ordinary mistakes using the CLI and editor.
6. **Submit:** Generate judge-compatible code and receive accepted verdicts on supported online judges.
7. **Compete:** Participate in a real or simulated contest using CPL as their primary implementation language, without being blocked by missing ordinary functionality or unexpected tool failures.

At the end, they should have enough experience to offer meaningful feedback about the language's expressiveness, correctness mechanisms, tooling, and overall usability.

They should also have experienced at least one compelling advantage over their existing workflow.

### 5.2. What must be ready

**Language and compiler**

A coherent, documented, sufficiently complete subset of CPL that supports representative real contest problems, including nontrivial algorithms and data structures.

The supported subset must be correct and implemented consistently across execution modes. Gaps must be clearly identified rather than disguised.

**Standard library**

The foundational containers, algorithms, mathematical utilities, and helper functions needed for representative contests.

We do not need an exhaustive library yet, but users should not regularly encounter situations where ordinary contest solutions are impossible or require extensive workarounds.

**CLI and local testing**

Reliable checking, execution, sample testing, custom tests, and judge-ready builds.

Sample import through Competitive Companion or an equally smooth supported workflow should work.

**Editor support**

At least one polished, mainstream editor integration—VS Code is the natural initial target—with syntax highlighting, diagnostics, completion, and formatting.

The user should be able to write CPL comfortably without switching between disconnected tools.

**Website and learning**

A credible website, a concise introductory guide, searchable documentation, runnable examples, and enough introductory practice to teach the fundamental language model.

Documentation examples must actually work.

**Contest submission**

Reliable generation of self-contained C++17 submissions for the explicitly supported judges, including Codeforces and AtCoder as initial targets.

Standard single-test and multitest problems must work. Other problem formats should be supported where feasible and clearly scoped where not.

**Basic debugging**

High-quality diagnostics, runtime errors, print debugging, assertions, and fast reproduction of failing cases.

A basic integrated brute-force/stress-testing workflow would be particularly valuable because it demonstrates one of CPL's central differentiators.

The complete time-travel debugging vision is not necessarily required for this milestone.

**Release quality**

Reproducible builds, tested installation paths, automated regression checks, and no known serious correctness or submission blockers in supported workflows.

### 5.3. What can wait

The following are valuable but should not block the first milestone unless testing reveals that they're essential:

- Comprehensive advanced standard-library coverage.
- Fully polished time-travel debugging and trace visualization.
- Sophisticated automated test generation and shrinking for arbitrary input structures.
- Advanced static performance analysis and inline resource annotations.
- Direct submission integrations with every major judge.
- Comprehensive JetBrains support.
- A sophisticated third-party package ecosystem.
- Full backward compatibility across many future language versions.
- Exhaustive tutorials for beginners.
- World-class compiler optimizations beyond competitive parity.

The scope should be determined by whether friends can realistically use CPL, not by whether every planned component is finished.
