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

Ground every design in a person in a moment: who they are, where they are
in the contest, what they just did and what they want next. This is how you
think, not a template; a doc about the model of sequences can open with
the model, as long as its examples come from real moments.

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
- **Pick examples that could change the decision.** Often an ordinary
  workflow that makes two designs feel different, not a corner case.
- **Say why it is better.** "Simpler" or "more intuitive" needs a side by
  side comparison behind it.

## Enough, and no more

Write every document the hierarchy needs: an area without a README, or a
surface without its use cases, is a gap. Keep each one tight. A surface doc
is under a page, a use case about five lines, an area README a paragraph per
surface. If a document grows, split it or move history into an RFC;
product documents say what we intend today.

## Checklist

- [ ] Its examples come from concrete contestant moments.
- [ ] The interaction is shown step by step: input, what they see, what's next.
- [ ] The ideal is written down, and today's gaps against it are named.
- [ ] At least one off-the-happy-path case is covered.
- [ ] Needs and designs are stated separately.
- [ ] Claims that a design is better rest on a comparison.
- [ ] It fits its length budget.

## Changes

- 2026-11-23: first version.
