<!-- Moved from plan/site/user-journey.md (branch devin/wip-site-plans, 2026-11-21). Where it differs from the code, the code and handbook/domains/site.md win. -->

# CP Lang website: the new user's journey

A story of one new user, from the first visit to a habit, with what the
site does at each step and why. A companion to `91-website.md`. Mostly
proposal; where it relies on something that exists today, that is said.

Status: decided (Ryan agreed to the journey, 2026-11-21).

## The user

Mei is a Codeforces contestant rated about 1900. She writes C++ with a
personal template, knows Python, and has lost points this year to an
`int` overflow and to a multitest array she forgot to clear. A friend
posts a CP Lang share link in a Discord: a ten-line segment tree solution
next to its 60-line C++ original. She clicks it.

Her goal for the first visit is not "learn a language". It is "is this
worth my time?", and she will give it about two minutes.

## Act 1: the hook (0-2 minutes)

**The share link opens the playground**, not the landing page, because
that's what was shared. She sees the friend's program, the input, and a
*Run* button. The beta banner at the top says the language can still
change. She presses Run: the WASM compiler loads on that first click
(about 1.3 MB, *verified* size today) and the output matches. She clicks
*C++17* and sees the generated file, readable and short. That is the
first trust moment: it compiles to something she could have written.

A small strip over the editor says "New to CP Lang? Take the 10-minute
tour for C++ programmers ->". She clicks the logo instead and reaches the
landing page.

**The landing page** answers "why" in one screen. The first thing on it
is code: tabs of short programs (multitest A + B, a DP with `@cache`, a
grid BFS) she can edit, run and flip to C++17 in place. Below it, "The bugs
you can't write" shows four C++/CP Lang pairs. The second pair is exactly
her multitest bug: the C++ forgets to reset `cnt[]`, and the CP Lang
version declares per-test state so there is nothing to forget. That is
the second moment: the site knows her pain.

Two buttons: *Start with a problem* and *Open the playground*. A row
below says "Coming from C++ · Python · Java · Rust". She picks C++.

## Act 2: the quickstart (2-15 minutes)

**"CP Lang for C++ programmers"** walks the same six small problems every
quickstart uses: sum of n numbers, two pointers, grid BFS, knapsack, a
multitest problem, modular combinatorics. Each step shows her C++, the
CP Lang version, and one sentence on the trap removed. Every block is
runnable in place, with its own stdin.

At grid BFS she meets her first new idea: `domain Cell = ...` and
`?Int` distances with no `INF`. It looks odd. The step says why in two
lines and links "Domains in depth" to the guide, in a new tab, so she
doesn't lose her place. She edits the BFS to 8 directions, runs it, and
it works on the first try.

At the knapsack she tries `dp[i][w]` from habit. The compiler's error is
one line, names the rule and shows the fix (`dp[i, w]`), and links its
error page (the error index exists today, generated from the compiler).
She fixes it. That is the third moment: errors are help, not noise.

The quickstart ends: "You've seen the core. Now solve real problems."
Her progress so far is already saved locally, with no sign-up.

## Act 3: the course (first session, 15-60 minutes)

**Unit 1, "Read, compute, print"**, opens on a short concept page with
two runnable examples, then the problem list: six problems from trivial
to Div2-B. Each problem looks like a judge page: statement, samples, an
editor with the starter code, *Run samples* and *Submit*.

Submit runs the hidden tests in the browser. Problem 4 gets **WA on test
7**. The verdict panel shows the input, her output, the expected output
and the first differing line. A button says *Trace this test*: the trace
viewer (exists today) opens on test 7 and she steps to the line where
her loop bound is off by one. She sees the course's lesson for her: if
she'd written the loop over the domain instead of `0..n-1`, the bug
couldn't exist. She rewrites it that way. AC.

She finishes unit 1, skims unit 2 (arrays and ranges) because it's easy
for her, and stops in unit 3 (domains) after two problems. The sidebar
shows her units checked off. When she leaves, nothing asks for an email.

## Act 4: the first real use (the next contest)

Two days later, during a Div2 round, she opens the site to try CP Lang on
problem B for real.

- She writes it in the **playground**, runs the samples, clicks *C++17*,
  copies the file and submits it to Codeforces. AC. Install isn't needed
  yet. The page notes the output is one C++17 file under 64 KB, made for
  judges.
- On problem C she forgets what `lower_bound` is called. **Search**
  (Ctrl+K) finds `seq.partition_point` in the library reference. The
  entry gives the signature, `cost: O(log n)`, what it returns on an
  empty range, which index wins on ties, and an example she can run. That
  is the mid-contest promise: one search, one exact answer.
- One form she tries is marked `spec` (designed, not built yet). The
  badge says so before she wastes time on it, with a link to the status
  page.

After the round she shares her solution with the friend: the share link
holds the whole program in the URL.

## Act 5: becoming a regular (week 2 onward)

- The practice set lets her filter problems by tag; she does the DP and
  graph units.
- The cookbook answers "how do I compress coordinates" with a complete
  program.
- After her first AC the course had suggested "Solving in your editor is
  faster". Before the next round she opens **Setup**: the installer gives
  her `cpl`, the VS Code extension installs with one click (once it's
  published; until then the same tab shows the from-source install), and
  `cpl doctor` prints all green. In her editor she now gets diagnostics as she
  types, hover types and Run/Test/Stress lenses. She sets her default
  judge to Codeforces.
- She installs Competitive Companion. During the next round one click on
  the problem page creates the problem with its samples, `cpl watch`
  reruns them on every save, and `cpl submit` tells her the file is ready
  to send. `cpl stress` finds a counterexample to her greedy on problem
  D. The browser is now only for docs.
- She is curious why the language works this way and reads *Design*:
  the mission, the goals, and the searchable decisions index. She opens a
  docs bug from the "Report a docs bug" link on a page with a confusing
  example.

## Other entry points

The same site serves other arrivals; each has its own first step:

- **A link in a top competitor's submission.** Arjun reads a red
  coder's accepted C++ after a round. It starts with the header `cpl`
  already writes *(verified: the CP Lang source, quoted in comments, and
  the judge profile)*, plus one new line: `// Written in CP Lang:
  <site>/s`. He opens it. `/s` is a page for exactly this arrival: "You
  just read a CP Lang submission. Paste it here." He pastes the whole C++
  file; the playground pulls the CP Lang source back out of the header
  comment and opens it next to the C++, with the sample tests empty for
  him to fill. Nothing was uploaded by anyone: the source was already in
  the submission. He sees 25 lines next to 140, runs it, and goes on to
  the C++ quickstart like Mei. The submission is the advertisement, and
  the strongest proof the language works is that it got AC.
- **A friend's DM.** Mei, now hooked, DMs a teammate "try this". She
  clicks *Share* on a course problem she solved, which offers two links:
  *her solution* (a playground link) or *the problem as a challenge*
  (`/learn/<problem>`, with "Mei solved this in CP Lang" at the top and
  no spoilers). She sends the challenge. In Discord and Messenger the
  link unfurls with a card showing the problem title and a code snippet.
  The teammate lands straight on the problem page, which works cold: a
  collapsible "New here? 2-minute intro" above the editor, the same
  verdicts, and the course map one click away. Solving it is his tour.
- **A Python user** picks "Coming from Python": the same six problems,
  but the traps are speed (TLE in Python vs C++ speed here) and recursion
  depth. Same course afterwards.
- **A curious language designer** (from Hacker News) lands on the landing
  page, skips to *Design* and *Specification*, and judges the language
  from `spec/core` rendered with rule anchors.
- **A model asked to write CP Lang** reads `/llms.txt` and the one-file
  Markdown docs. The writable-from-docs eval measures this exact path.
- **A contributor** goes from *Contributing* to *Domains* and *Status* to
  find an area and its next step.

## What the journey asks of each component

- Playground: works from a share link with zero setup; the first run is
  fast; the C++ is one click away.
- Submissions: the generated C++ carries a one-line link back, and the
  playground can reopen any `cpl` output file from its header.
- Sharing: links unfurl with a code card; a problem can be shared as a
  challenge, and every problem page works for someone who has never
  seen the site.
- Landing: one screen of why, with bugs a contestant recognizes; a
  background picker.
- Quickstarts: parallel across languages, runnable, and linking out to
  guides without losing your place.
- Errors: one line, the rule, the fix, a link.
- Course: judge-like pages, verdicts with diffs, a trace on the failing
  test, progress with no account.
- Reference and search: one search gives an exact, complete answer
  (cost, empty, ties, example).
- Status badges: never let a reader write something that doesn't run
  without being told first.
- Setup: a checklist from install to editor extension to `cpl doctor`
  to the contest loop, with gentle nudges at the moments the browser
  starts to feel slow.

## What to measure (if we ever add analytics)

The journey has six checkpoints: opened the site, ran a program, finished
the quickstart, first AC in the course, installed `cpl` and an editor
extension, came back within a week. The
docs-quality checkpoint is already measured offline by the
writable-from-docs eval.
