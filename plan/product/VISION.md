<!-- Ryan's product doc. Frozen after the 2026-11-22 edit pass; change only with Ryan's say-so (propose in plan/QUESTIONS.md). -->

# CPL — Product vision

Who CPL is for, why it should exist, and how we decide. The experience it delivers is in [EXPERIENCE.md](EXPERIENCE.md); the current milestone is in [MILESTONE-1.md](MILESTONE-1.md). AGENTS.md turns this into measurable goals and design rules.

## 1. Who we're building for

We're building CPL for **intermediate to elite competitive programmers**, initially targeting roughly Codeforces 1500+.

Our users already know how to program and are fluent in at least one language. They regularly participate in contests, upsolve problems, and train.

We want CPL to become their **primary language and toolchain** for all three activities.

Our main competitor is C++ with years of accumulated personal libraries, templates, scripts, and editor customizations. We're not competing against vanilla C++; we're competing against an experienced programmer's highly optimized workflow.

Beginners are a potential future audience, but we're currently optimizing for experienced competitive programmers.

## 2. Mission

**Make turning an algorithmic idea into a correct, efficient, accepted solution as fast, natural, and enjoyable as possible.**

We want programmers to spend their time thinking about the actual problem rather than fighting the mechanics of implementation.

Crucially, we're optimizing for the **entire problem-solving process**, not merely writing the final implementation. Ideas evolve, solutions get revised, hypotheses need testing, and bugs need diagnosing.

Our ambition is to make the experience of competitive programming substantially better, not just to create a more convenient programming language.

## 3. Core value propositions

There are three main ways we want CPL to be genuinely better than existing alternatives.

### V1. Effortless, elegant expression of ideas

**The language should feel like a natural extension of algorithmic thinking.**

You think of an algorithm, and translating it into code is straightforward, fluent, and satisfying.

- Code expresses the underlying idea directly, rather than requiring substantial incidental machinery.
- The language is expressive, concise, coherent, and genuinely pleasant to write.
- A powerful standard library makes common operations and sophisticated algorithms easy to express.
- The obvious, natural way to write something is generally the best way.
- Revising an idea is effortless. Changes stay local rather than requiring cascading rewrites.
- Programs are easy to read, reason about, and understand, including difficult solutions.

We want the experience of writing CPL to feel almost frictionless.

**The ideal:** You know what you want to express, and you almost never need to stop and figure out *how* to express it in the language.

### V2. Correctness by construction

**Make incorrect code difficult to write, while keeping correct code effortless to express.**

Competitive programmers waste substantial time on implementation bugs unrelated to the underlying algorithm.

We want to systematically eliminate that burden.

- Language semantics prevent common mistakes rather than merely making them easier to debug.
- Types and static analysis capture facts the programmer already knows, without requiring excessive annotations.
- Common bug classes—off-by-one errors, wrong index spaces, overflow, accidental mutation, and so on—become harder to introduce.
- Compiler errors are exceptionally clear, intuitive, and actionable.
- When something compiles, the programmer has much greater confidence that it implements their intended logic.

Importantly, we don't want safety at the cost of constantly fighting the compiler. The safety mechanisms themselves must feel natural.

**The ideal:** You rarely encounter mysterious implementation bugs, and you rarely encounter confusing compiler restrictions. The language helps you write correct code without getting in your way.

### V3. An exceptional problem-solving and debugging environment

**Make the entire loop of experimenting, testing, understanding, and debugging dramatically better.**

Competitive programming involves uncertainty. You frequently have a conjecture you want to check, an algorithm you're unsure about, or a wrong answer you don't understand.

CPL should make resolving that uncertainty extraordinarily easy.

- Experimenting with algorithms and testing small cases is effortless.
- Writing brute-force solutions, generators, and checkers feels natural.
- Stress testing automatically finds, minimizes, and reproduces counterexamples.
- Runtime failures explain what went wrong, where, and with what state.
- Tracing and time-travel debugging make execution easy to understand.
- The CLI, editor, and tools are deeply integrated into one coherent workflow.
- Every part of the experience is fast, responsive, and requires minimal setup.

The tooling should feel like one beautifully designed system, not a loose collection of utilities.

**The ideal:** Whenever you're uncertain about something, the path from doubt to evidence is extremely short.

## 4. Non-negotiable foundation: Competitive viability and trust

The three value propositions above are why someone would *want* to use CPL.

But before someone can realistically adopt it as their primary contest language, we must meet a very high baseline.

**Choosing CPL must not put you at a competitive disadvantage relative to C++.**

This means:

- **Correctness:** The compiler, interpreter, and generated code faithfully implement the specified language semantics.
- **Performance:** Idiomatic CPL runs comparably to optimized C++ implementing the same algorithm, with a path to equivalent performance when additional tuning is necessary.
- **Resource efficiency:** Memory usage and generated submission size are competitive and predictable.
- **Judge compatibility:** Generated code works reliably across supported online judges and their constraints.
- **Reliability:** Compilation, local execution, and submission behave consistently. The toolchain is stable and dependable.
- **Responsiveness:** Compilation, feedback, and local tooling are fast enough to never meaningfully interrupt the problem-solving process.

A significant failure in any of these areas can make the entire product unusable, regardless of how good the language feels.

We should treat them as **hard product quality gates**. The measurable targets are AGENTS.md goals 3 and 5 and each domain's `metrics.toml`.

## 5. Product principles

Our value propositions describe what we want to achieve. Our principles describe how we'll make decisions when desirable properties conflict.

### P1. Optimize the entire problem-solving experience

Don't optimize isolated features at the expense of the overall workflow.

The unit of value is the whole journey from an idea to an accepted solution, including revisions, experimentation, and debugging.

A feature that makes code shorter but harder to revise or debug may be a net loss.

### P2. The obvious way should be the correct and fast way

Natural-looking code should behave as expected and perform predictably.

If idiomatic code is unexpectedly wrong, inefficient, or surprising, we should treat that as a language-design problem rather than something users need to learn to work around.

### P3. Minimize cognitive overhead, not merely keystrokes

Optimize for fluency, clarity, and the amount of mental effort required.

Conciseness is valuable when it makes the underlying idea easier to express and understand. It isn't an end in itself.

Avoid unnecessary ceremony, duplicated information, obscure rules, and concepts that users must repeatedly hold in their heads.

### P4. Prevent errors without introducing unnecessary friction

Prefer preventing mistakes over detecting them later, and detecting them early over leaving them for the judge.

But safety mechanisms should follow naturally from the program and information already available.

We should be extremely skeptical of restrictions that make correct programs awkward or whose behavior is difficult to understand.

### P5. Coherence over feature count

Build a small number of powerful, general, composable concepts rather than accumulating special cases.

Every new construct should fit naturally with the rest of the language and justify the additional complexity it introduces.

The standard library should generally be built from mechanisms available to user code rather than bespoke compiler privileges.

### P6. Powerful defaults, predictable behavior, explicit control

Common operations should require minimal configuration or ceremony.

At the same time, users should understand what their programs do and what they cost. Avoid surprising implicit choices of algorithms, representations, or expensive operations.

Advanced users must retain the ability to control performance and handle unusual situations without fighting the language.

### P7. Everything should fit together beautifully

Treat the language, standard library, diagnostics, CLI, editor, debugger, documentation, and other tools as parts of a single product.

Use consistent concepts and terminology. Eliminate awkward handoffs, unnecessary configuration, and workarounds.

**A user should never feel that they're hacking together multiple systems to accomplish something that ought to be straightforward.**

This is also an aesthetic principle: CPL should feel thoughtfully, deliberately designed, with a level of polish and coherence that is immediately apparent in use.

## 6. What success looks like

Ultimately, success is not a particular syntax, feature count, benchmark score, or number of users.

Success means that an experienced competitive programmer can use CPL as their primary language and **meaningfully prefers it to their existing C++ workflow**.

We'd expect to observe:

- **Higher fluency:** Users translate ideas into code and revise implementations more quickly, with less mental overhead.
- **Fewer implementation mistakes:** Users introduce fewer bugs and discover them earlier.
- **Faster resolution of uncertainty:** Testing and debugging become substantially more efficient.
- **Greater satisfaction:** Users find the language intuitive, elegant, and enjoyable.
- **No material competitive disadvantage:** Users retain the performance, control, and reliability they expect from C++.
- **Sustained preference:** After becoming familiar with CPL, users voluntarily choose it for real training and contests rather than reverting to C++.

We should eventually measure these outcomes through representative problem corpora, controlled comparisons, realistic contest sessions, and actual user feedback.

## 7. Scope and non-goals

We're designing specifically for competitive programming, not general production or systems programming.

We're prioritizing intermediate and advanced competitors, not beginners.

We're not trying to automatically discover or choose algorithms on the programmer's behalf. We want to empower their own algorithmic thinking.

We're not optimizing for familiarity, minimal syntax, or maximal feature count at the expense of long-term usability and coherence.

We're willing to rethink established programming language conventions when doing so produces a substantially better experience.

## 8. The long-term ambition

We want CPL to become the language that serious competitive programmers would choose if they could design their ideal programming environment from scratch.

A language that feels extraordinarily natural to write, systematically prevents implementation mistakes, makes debugging and experimentation effortless, and sacrifices none of C++'s competitive capabilities.

**CPL should make competitive programming feel closer to pure algorithmic thinking.**
