# The website: detailed plan

Status: decided in outline (Ryan, 2026-11-21: "we should just go and
build out everything"). Page-level content below is proposed; Ryan
reviews it on the preview, not here.

This directory specifies the new CP Lang website closely enough that an
agent with only these files and the repo could build it. `00-overview.md`
is the short version and the reasons; the rest says exactly what each
part contains.

## Documents

0. `00-overview.md`: who it's for, principles, components, stack,
   hosting, phases, open decisions.
1. `01-journey.md`: one new user's story, from a share link to a habit,
   plus the other entry points.
2. `10-landing.md`: the home page, section by section, with its copy and
   snippets.
3. `20-onboarding.md`: getting started, the four quickstarts and their
   six shared problems, the tour.
4. `30-course.md`: the problem format, the runner, verdicts, and the
   full list of tracks and problems.
5. `40-playground.md`: the editor component, the playground page, share
   links, `/s` and embeds.
6. `50-docs.md`: the docs tree page by page: guides, the language and
   library reference, cookbook, errors, cheat sheet, glossary, search,
   `llms.txt`.
7. `60-tools-setup.md`: the CLI reference, workflows, editors, the Setup
   checklist and submitting to a judge.
8. `70-project.md`: status, design, domains, decisions, real problems,
   performance, release notes, contributing.
9. `80-design-system.md`: shadcn/ui and Fumadocs as published, the page
   templates, the few product widgets, and copy style.
10. `90-build.md`: the `web/` layout, content loaders, the snippet gate,
    WASM, tests, CI, hosting, cutover, and each phase's acceptance
    checks.

## What "done" means

The site is done when all of these hold on a local build and a preview
deploy:

- Every page listed in `10` to `70` exists, with no placeholder copy.
- Every code fence on the site is gated: `runs` fences run on today's
  compiler and match their expected output; `checks` fences pass
  `CPL_SPEC=1 CPL_STD=spec/std cpl check`; `spec` fences are labelled.
- The playground runs, traces, shows C++17 and shares, offline after the
  first load.
- The course has at least 60 original problems, each with tests, a
  reference solution that passes them, and an editorial.
- The library reference covers every public function in `spec/std`
  with its labelled facts.
- The writable-from-docs eval run on `/llms-full.txt` passes at least as
  many problems as it does on today's docs (49 of 55).
- Lighthouse on the landing, a docs page and the playground: 95+ on
  performance, accessibility and best practices.
- `site/` is deleted and `scripts/check.sh --fast` gates `web/` instead.

What stays outside the site and depends on Ryan or other areas:
a Cloudflare account and domain, deploying the share Worker, publishing
editor extensions, prebuilt `cpl` binaries, and new `cpl` features
(`cpl doctor`, the Competitive Companion listener, the header link
behind `/s`).

## Rules for these documents

- Facts about today's repo are marked *(verified)*; the rest is the
  design.
- Content comes from the repo wherever it already exists; each page says
  its source. Hand-written copy is the exception and says so.
- `spec/` is normative. When a page here and the spec disagree about the
  language, the spec wins and the page is wrong.
