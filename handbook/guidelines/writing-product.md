# Writing product documents

| | |
|---|---|
| Kind | Guideline |
| Owner | Coordinator |
| Applies to | Journeys, area READMEs, surface docs, use cases and product RFCs |

A product document describes CP Lang from the contestant's seat. Its job is
to make the reader feel what using the product is like, good or bad, before
anyone builds it. `writing-docs.md` covers explaining how something works;
this guide covers describing what it is like to use.

## Sit in the user's seat

Start every description from a person in a moment: who they are, where they
are in the contest, what they just did and what they want next.

> 40 minutes in, WA on test 7. You suspect the greedy is wrong and want a
> counterexample before you rewrite it.

Then write what they do and what they see, in order: the code they type,
the command they run, the completion that appears, the output or error they
read, and what they do next. Show the session; don't list features.

## Say what good feels like, and where it hurts

- **The ideal first.** Write the best interaction you can imagine before
  looking at what CP Lang does today. Then compare.
- **Name the friction.** Hesitation, a lookup, a second command, a copy and
  paste, a sentinel check, an error that names the wrong thing. Each one is
  a cost to the contestant; say which step pays it.
- **Follow it off the happy path.** The empty input, the typo, the
  requirement that changes, the first idea that was wrong.
- **Keep the need apart from the design.** "Find where the first match is"
  is a need; `a.find_index(p)` is one design for it. Name the need first.

## Keep it short

A surface doc is under a page. A use case is about five lines. An area
README is a paragraph per surface. If a document grows, split it or cut
history out into an RFC; product documents say what we intend today.

## Checklist

- [ ] It opens with a person in a concrete moment.
- [ ] The interaction is shown step by step: input, what they see, what's next.
- [ ] The ideal is written down, and today's gaps against it are named.
- [ ] At least one off-the-happy-path case is covered.
- [ ] Needs and designs are stated separately.
- [ ] It fits its length budget.

## Changes

- 2026-11-23: first version.
