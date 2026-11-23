<!-- Ryan's product doc. Frozen after the 2026-11-22 edit pass; change only with Ryan's say-so (propose in plan/QUESTIONS.md). -->

# CPL — Product experience

The user-facing experience we want to deliver, not the implementation architecture. Who it is for and why is in [VISION.md](VISION.md).

## 1. Overview

CPL handles the whole workflow of people who use it as their primary language for contests, upsolving and training, from opening a problem to an accepted submission. The language, standard library, editor, docs, testing tools, debugger and contest management work together, so attention goes to algorithms rather than syntax, personal templates, implementation bugs or tooling.

## 2. Major user journeys

### J1. Discovering and evaluating CPL

**User goal:** Understand what CPL offers and experience its value before committing to learning or installing it.

Someone encounters CPL through word of mouth, a Codeforces blog, or a submission from a strong competitor.

They visit our website and immediately understand what the language is, who it's for, and why it might be better than their current workflow.

They can see compelling examples of elegant solutions, powerful abstractions, safety features, and debugging tools.

Ideally, they can try CPL directly in their browser without installing anything.

**The desired experience:** Within a few minutes, an experienced competitor can understand CPL's value proposition and experience something meaningfully better than their existing workflow.

*Detailed discovery and marketing design can come later.*

### J2. Learning CPL

**User goal:** Become productive quickly, then progressively master the language and its idioms.

**Initial learning**

An experienced programmer can read a concise guide in approximately 30 minutes and learn the essential syntax, semantics, and programming model.

They can then work through a carefully selected collection of simple competitive programming problems, roughly Div. 2 A/B difficulty, directly in the browser.

The problems are selected to teach CPL's distinctive patterns and idioms, not just generic programming.

While solving:

- They can easily search for any operation or language feature they're unsure about.
- Contextual hints point to relevant standard-library documentation or language guides.
- Linting suggests cleaner, more idiomatic ways to express their code.
- Documentation contains runnable, modifiable examples.
- Feedback helps them understand CPL's intended abstractions rather than merely giving them accepted solutions.

**Progressive learning**

After becoming productive, users can explore advanced guides covering deeper language features, standard-library facilities, performance, and idiomatic programming.

Guides can be interactive, with embedded code and optional exercises.

The documentation is comprehensive, exceptionally well organized, and easy to search. It should be possible to find the right abstraction even when you don't know its name.

Learning also happens passively during normal programming: editor suggestions, diagnostics, and lints continually teach better ways of expressing ideas.

We can offer optional AI assistance for questions such as "How would I express this more idiomatically in CPL?" while avoiding inappropriate assistance in contexts where contest rules prohibit it.

**The desired experience:** Get productive in under an hour, and keep getting better naturally through everyday use.

### J3. Installing and configuring CPL

**User goal:** Start using CPL in their preferred environment with essentially no setup friction.

Installation should be a simple script or equivalent native installer.

Setting up the editor should require as few clicks as possible. Supported editors should offer syntax highlighting, completion, diagnostics, formatting, navigation, and relevant CPL tooling immediately.

Everything should work with excellent defaults.

Advanced configuration is available for users who want it, but shouldn't be required.

Installation failures should be rare, clearly explained, and easy to fix.

**The desired experience:** Install CPL, open your favorite editor, create a file, and start programming. No complicated setup or manually assembled toolchain.

### J4. Starting and organizing problems and contests

**User goal:** Open a problem or contest and immediately start working.

CPL integrates with Competitive Companion and similar sources to automatically identify problems and retrieve relevant metadata, sample tests, and constraints where available.

It can create problem files and organize contest workspaces using sensible defaults.

Users can work with their own preferred directory structures, naming conventions, templates, and configurations.

Starting another problem should be almost effortless. Switching between contest problems should preserve the relevant context, tests, and files.

The same system should comfortably support live contests, independent upsolving, practice sessions, and locally created problems.

**The desired experience:** Contest organization happens automatically when possible, without imposing a rigid workflow or directory layout.

### J5. Writing, exploring, and revising solutions

**User goal:** Translate algorithmic thinking into working code as naturally as possible.

This is one of CPL's central experiences.

The language should be exceptionally expressive, coherent, and ergonomic. Code should closely resemble how a programmer naturally thinks about the algorithm.

The standard library should provide powerful, composable abstractions for common competitive programming operations.

Users should rarely need to write repetitive machinery, manually manage irrelevant implementation details, or remember obscure language-specific tricks.

**Discoverability is central.**

Whenever a user knows what they want to accomplish but doesn't know how CPL expresses it:

- Documentation and examples are easy to find.
- Editor completion makes APIs discoverable.
- Lints identify manual patterns that could be expressed more cleanly.
- Relevant stdlib alternatives are suggested automatically.
- Optional AI assistance helps explain or rewrite code idiomatically.

**Writing incomplete and evolving programs should also feel natural.**

The editor provides useful feedback while code is unfinished. Compiler errors are accurate, intuitive, and actionable, rather than producing confusing cascades.

Revising an algorithm should require local, understandable changes. The programmer shouldn't repeatedly synchronize duplicated information or restructure unrelated code.

Users can quickly experiment with parts of their implementation and test hypotheses without constructing an entire separate program.

**The desired experience:** The distance between thinking of an algorithm and expressing it in code is extremely small. The programmer can focus almost entirely on the problem rather than the language.

### J6. Running, testing, and experimenting

**User goal:** Quickly determine what their program does and whether it behaves as expected.

The default workflow is extremely simple.

When samples have been imported through Competitive Companion, running `cpl run` can execute the relevant tests automatically and display useful results.

Users can easily:

- Run an individual test case or all samples.
- Provide arbitrary custom input.
- Add and organize local test cases.
- Compare against expected outputs.
- Define custom checkers for problems with multiple valid answers.
- Test individual functions or intermediate stages.
- Run incomplete programs or selected fragments where supported.
- Experiment with small examples and hypotheses.

The CLI should be fast, predictable, and pleasant even without editor integration.

**Interactive problems should be first-class.**

Users should be able to play the interactor themselves or write a local interactor with very little ceremony. Input/output behavior and flushing should work correctly by default.

**The desired experience:** Testing a program or exploring an idea takes a single command or a few clicks, without setup overhead.

### J7. Debugging, stress testing, and understanding execution

**User goal:** Understand why something is wrong and fix it as quickly as possible.

CPL supports multiple debugging styles. No particular style is mandatory.

**Lightweight debugging**

Users can print values, inspect runtime errors, add assertions, run specific inputs, and use ordinary CLI commands.

These basic workflows should be excellent. Sophisticated tools should enhance them, not replace them.

**Stress testing**

Users can write an optimized solver and a brute-force alternative naturally, ideally in the same source file.

CPL makes it exceptionally easy to produce random tests and compare the implementations.

Where input schemas and constraints provide sufficient information, it can automatically generate valid cases and appropriate shrinkers. Users can add custom generators and constraints when needed.

Stress testing uses available parallelism, finds discrepancies, and intelligently minimizes counterexamples.

Our aspiration is that common implementation bugs can be exposed in seconds, with minimal manual setup.

**Execution tracing and time-travel debugging**

When users need deeper understanding, they can open an exceptionally well-designed execution viewer.

They can:

- Step forward and backward through execution.
- Inspect variables, data structures, and relevant program state.
- Navigate to particular test cases, function calls, or stages.
- Filter enormous traces to focus on relevant events.
- Examine runtime failures with the offending values and context.
- Compare executions and investigate divergence.
- Edit code and rerun the relevant case conveniently.

The experience should work through the editor and, where practical, through a browser-based viewer.

**An integrated debugging loop**

A particularly compelling experience is:

Wrong answer → stress test → minimized counterexample → execution trace → identify the problem → edit → rerun → resume testing.

These should feel like steps within one workflow, not separate tools requiring manual coordination.

**The desired experience:** From "I don't know why this is wrong" to a concrete explanation or counterexample as quickly and effortlessly as possible.

### J8. Understanding and optimizing performance

**User goal:** Understand the time and memory costs of their program and optimize it confidently.

CPL should make performance much more transparent than conventional competitive programming languages.

A dedicated command, potentially integrated with `cpl explain`, could analyze and annotate the program with information such as:

- Inferred or declared loop bounds and iteration counts.
- Memory consumption and maximum sizes of important data structures.
- Storage widths and representation choices.
- Allocations, copies, and potentially expensive operations.
- Estimated computational complexity where the analysis can establish it.
- Potential time-limit, memory-limit, or overflow problems.

Much of this information should also be available directly in the editor, through inline annotations, hover information, or a dedicated performance view.

**It's important to distinguish proven bounds, estimates, and unknowns.** We should never present a heuristic performance guess as a guarantee.

When performance problems arise, CPL should help users understand where the costs come from and how to address them.

Users should have precise control over representations and algorithms without abandoning the language.

**The desired experience:** Performance is understandable and predictable. Users can reason about and optimize their programs without repeatedly inspecting generated C++ or guessing what an abstraction costs.

### J9. Building and submitting solutions

**User goal:** Submit a correct, efficient program without worrying about the toolchain.

The basic workflow should be trivial.

For example:

- `cpl build` generates the judge-ready submission.
- `cpl build --copy`, or `cpl b -c`, copies it directly to the clipboard.
- The generated file is self-contained and compatible with the selected judge environment.

Users should be able to trust that what worked locally will behave consistently in the supported judge environment.

Compiler versions, build settings, and judge profiles should be reproducible.

Submission failures related to source-size limits, unsupported environments, or configuration should be caught locally where possible and explained clearly.

Eventually, CPL could support `cpl submit` with direct integrations for major online judges, but that doesn't need to be part of the initial product.

**The desired experience:** Once your solution is ready, submitting is almost thoughtless. CPL never becomes the reason you lose a contest.

### J10. Managing libraries, configuration, and existing solutions

**User goal:** Maintain a personalized competitive programming environment that remains reliable over time.

Users should be able to combine:

- CPL's standard library.
- Their own local libraries and templates.
- Libraries shared by other programmers, initially potentially through GitHub rather than a full package ecosystem.

Using and updating libraries should be straightforward.

Compiler and library upgrades must not unexpectedly break old solutions. Users should be able to pin versions, reproduce old builds, and migrate when desired.

Formatting, linting, editor behavior, contest organization, and other tools should be configurable while retaining strong defaults.

Configuration should be understandable, discoverable, and consistent. Users should not need elaborate setup files to accomplish ordinary tasks.

Their environment should work across different contests, machines, and evolving workflows.

**The desired experience:** CPL is pleasant not only on day one but after years of use. Their investment in code, knowledge, and tools continues to pay off.

---

## 3. Important contexts and variations

The journeys above must work in several different situations.

**Live contests:** Speed, reliability, predictability, minimal interruptions, and offline functionality where feasible are especially important. AI assistance must respect applicable contest rules.

**Upsolving and training:** Experimentation, learning, detailed diagnostics, tracing, and optional AI assistance can play larger roles.

**Easy versus difficult problems:** Trivial programs should require almost no ceremony. Highly complex algorithms should support sophisticated abstractions, decomposition, and precise control without becoming awkward.

**Ordinary versus unusual problems:** Multitest problems, interactive tasks, nonunique outputs, grader-based problems, and unusual input/output formats should all have understandable workflows.

**Success versus failure:** The product should be equally thoughtfully designed when code doesn't compile, a test fails, the program crashes, performance is poor, or a service is unavailable.

An important cross-cutting principle is **progressive disclosure**: the simplest workflow should be extremely simple, while more sophisticated functionality becomes available naturally when needed.

## 4. Required product capability areas

The journeys imply the following high-level capabilities. These are product-facing groupings, not necessarily engineering teams.

1. **Language and standard library:** Expressive syntax, coherent semantics, safety, powerful abstractions, predictable costs, expert control.
2. **Discovery and learning:** Website, searchable documentation, interactive guides, practice problems, idiom education, optional AI help.
3. **Authoring experience:** Editor integrations, completion, diagnostics, formatting, linting, navigation, inline information.
4. **Contest and workspace management:** Problem imports, sample retrieval, file organization, configurations, contest workflows.
5. **Execution and testing:** Fast local execution, sample testing, custom tests, checkers, partial execution, interactive harnesses.
6. **Debugging and experimentation:** Brute-force comparison, generators, shrinking, tracing, time-travel debugging, execution inspection.
7. **Performance understanding:** Static cost analysis, memory and loop-bound information, profiling, source-level explanations.
8. **Submission:** Self-contained code generation, judge profiles, reproducible builds, clipboard export, eventual direct submission.
9. **Long-term maintenance:** Personal libraries, external libraries, versioning, compatibility, migration, persistent configuration.

Each capability should eventually be given concrete behavioral requirements, quality standards, and acceptance tests.

## 5. Milestones

The current milestone, its acceptance scenario and its scope are in [MILESTONE-1.md](MILESTONE-1.md).

## 6. Long-term standard

See [VISION.md](VISION.md) §6 and §8. **The end state is a competitive programming environment where almost all unnecessary friction between algorithmic thinking and accepted solutions has disappeared.**
