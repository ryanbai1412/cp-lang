# Ownership and decisions

| | |
|---|---|
| Kind | Process |
| Owner | Coordinator |
| Describes | `handbook/domains/`, `decisions.md`, `plan/QUESTIONS.md` |

Who owns what, who may decide what, and what comes to Ryan.

## Goals

- **Every path, document and decision has exactly one owner.**
- **Decisions are made fast by the person closest to them,** and most are
  reversible, so being wrong is cheap.
- **Ryan's time is the scarcest resource.** Nothing waits for him.

## In short

Every domain has one owner: the lane that runs it. Within your domain you
change code and docs and make ordinary decisions yourself, logging the
reason. Anyone may fix an obvious bug, a stale line or a broken link
anywhere. Changing something another domain depends on, such as a crate
API, a std signature or a script's flags, needs that domain's owner, who
has 15 minutes to object before silence counts as consent. Changing a
principle, a domain's mission or the character of the language takes an
RFC, which goes ahead and is flagged for Ryan to review afterwards. The
exact roles and permissions are in the reference at the end.

## Making a decision

1. **Decide.** Pick the option that best serves the principles
   (`handbook/principles.md`) and the current milestone. Prefer the
   reversible option.
2. **Log it** in `decisions.md`: the call, the reason, and how to reverse
   it. One short entry.
3. **Keep moving.** Don't wait for agreement on a reversible call.

When a decision spans domains, the owner of the most affected domain
decides, after at most 15 minutes of discussion. Larger changes follow the
RFC process in `handbook-and-rfcs.md`.

## What comes to Ryan

Ryan wants to review afterwards, not approve beforehand:

- **Major changes** (principles, the language's character, overhauls of a
  process) are marked `Ryan to review` in their RFC or `decisions.md` entry.
  Work continues; he reads them later and can reverse them.
- **A question only he can answer**, that matters a lot and is hard to
  reverse, goes in `plan/QUESTIONS.md` with the options, real code, a
  recommendation and how to reverse it. Meanwhile, proceed on the most
  reasonable choice.

Everything else is an ordinary decision the owner makes.

---

# Reference

## Roles

| Role | Accountable for | Example |
|---|---|---|
| Domain owner | The long-term health of one domain: its code, its charter, its quality register, its metrics | The stdlib lane owns `std/` and `handbook/domains/stdlib.md` |
| Milestone owner | A user-facing outcome that crosses domains: finds gaps and asks the right domains to close them | The coordinator owns Milestone 1 end to end; idioms runs its acceptance run |
| Task owner | One bounded change, from start to landed | A helper agent porting ten solutions |
| Document owner | Keeping one handbook document coherent | Infra owns `process/release.md` |

The domains, what each owns and how they talk to each other are listed in
`handbook/domains/README.md` and each domain's charter. Two domains never
own the same path.

## Who may do what

| | Anyone | The owner | Needs the affected owner |
|---|---|---|---|
| Fix a bug, a stale doc line or a broken link anywhere | yes | | |
| Change code or docs inside a domain | small fixes | yes | |
| Change an interface another domain uses (a crate API, a std signature, a file format, a script's flags) | | proposes | yes: message them; no reply in 15 minutes counts as consent |
| Change a principle, a domain's mission, or the language's character | | writes an RFC | flagged `Ryan to review` |

If you need something from another domain and its owner is busy, make the
change yourself, keep it small, and tell the owner in their worklog. Don't
wait.

## Changes

- 2026-11-23: first version, from `plan/DOMAINS.md` and `AGENTS.md`.
