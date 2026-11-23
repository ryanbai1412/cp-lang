# Product: who CP Lang is for and what they get

These docs are the direction and product layers. Ryan owns them; propose
changes in `plan/QUESTIONS.md`. Everything else (spec, architecture, domain
plans) serves them.

| File | Answers |
|---|---|
| [VISION.md](VISION.md) | Who we build for, the mission, the three value propositions (V1-V3), the trust foundation, principles P1-P7, success, non-goals. |
| [EXPERIENCE.md](EXPERIENCE.md) | The user journeys J1-J10, contexts, the nine capability areas, the long-term standard. |
| [MILESTONE-1.md](MILESTONE-1.md) | The current milestone: a friend can learn CPL, solve real problems and compete with it. Acceptance scenario, what must be ready, what can wait. |
| [notes.md](notes.md) | Ryan's raw notes behind the three docs. |

## How the layers fit

| Layer | Question | Where it lives |
|---|---|---|
| Direction | Why are we doing this? | `VISION.md` |
| Product | What should exist from the user's side? | `EXPERIENCE.md`, `MILESTONE-1.md` |
| System | How will the product work? | `spec/` (normative language), `plan/ARCHITECTURE.md`, each domain's `ARCHITECTURE.md` |
| Work | What needs to change? | each domain's `PLAN.md` |
| Organization | Who is accountable for what? | `plan/DOMAINS.md` |
| Execution | How do intentions become shipped changes? | `plan/OPERATING.md`, `scripts/land.sh`, CI |
| Evidence | How do we know what is true? | `metrics.toml`, `plan/STATUS.md`, evals, the milestone acceptance run |

Vocabulary: a *mission* is the overarching purpose; a *principle* a durable
decision rule; a *goal* a desired outcome; a *metric* a number that indicates
it; a *milestone* a recognizable, testable state; a *user journey* an
end-to-end sequence of user actions; a *capability* something the product
enables; a *requirement* a property that must hold; a *specification* the
precise contract for behavior; a *quality attribute* a system-wide dimension
of goodness.

## Using these docs

- Every lane's priorities trace to the current milestone. Work that blocks
  a step of its acceptance scenario comes first; work on "what can wait"
  happens only when it is cheap or a hard gate.
- Each domain's `README.md` says what the milestone needs from it.
- Domain metrics are means; the milestone acceptance run is the end.
