# Site

The site is where most people meet CP Lang, and where many keep using it:
cp-lang.org in a browser, with the compiler running in the page. A
contestant should be able to go from "what is this?" to a solved problem
without installing anything, then install and keep the site open as the
reference beside their editor.

**Landing** answers "is this for me?" in one screen: a real problem solved
in a few lines next to its C++, a Run button that works, and the way to
install. It should take under a minute to decide to try it.

**Learn** is the course: chapters that teach the language in the order a
contestant needs it, and practice problems judged in the page. A reader
coming from C++ or Python should never hit a concept the course hasn't
introduced, and should be able to pass every problem in the browser.

**Reference** is the std library, the spec, the errors and the lints,
generated from the sources so they can't drift. You should find the item
for a task without knowing its name, and read its signature, options, cost
and edge cases at a glance.

**Playground** is the editor in the page: write, check, run, trace and
stress test a program, see the C++ it becomes, and share it as a link. It
should feel like a local editor with the language server, not a demo box.

**Practice** is the judge behind Learn's problems: hidden tests, verdicts
and hints, strict enough that a wrong solution never passes.

Each surface has its own doc and use cases: `playground.md` and
`playground-usecases.md` so far. Owner: site. How the site is built and
checked is in `handbook/domains/site.md`.
