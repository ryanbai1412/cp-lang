# The command line, as a contestant meets it

| | |
|---|---|
| Kind | Product area |
| Owner | tools |
| Changes | Edit directly; a changed design gets a use-case entry |

`cpl` is one binary, and every step of a contest attempt is one of its
commands. Each surface below is one question you ask between writing code
and sending it. They share one front end, one check profile and one file
layout (`d.cp`, `d.tests/`), so a case one command saves is a case the
next one runs. `cpl --help` lists them all, and each command has its own
`--help`.

**Check and fix** (`cpl check`, `cpl fix`, `cpl migrate`) answer "is this
legal, and what's the fix?" with one error per root cause, then apply the
fixes. Not drafted yet.

**Run** (`cpl run`) answers "what does it print on this input?" in the
debug interpreter, where every check traps with the local values in
source terms. Not drafted yet.

**Test** (`cpl test`, `cpl watch`) answers "does it pass the samples and
my cases?" on the checked build, through the problem's checker or
interactor. Not drafted yet.

**Stress** (`cpl stress`, `cpl locate`) answers "is there an input where
I am wrong?" and shrinks the one it finds. [stress.md](stress.md),
[use cases](stress-usecases.md).

**Build and submit** (`cpl build`, `cpl submit`, `cpl judges`) answer
"what do I send, and will it pass there?": one C++17 file for the named
judge profile. Not drafted yet.

**Explain, profile and bench** (`cpl explain`, `cpl profile`, `cpl bench`,
`cpl cover`) answer "why is it this fast, and where is the time going?".
Not drafted yet.

**Format and lint** (`cpl fmt`, `cpl lint`) answer "is this the
idiomatic form?" without changing what the program means. Not drafted yet.

**New and fetch** (`cpl new`, `cpl fetch`) answer "how do I start this
problem?" from its URL, samples or constraints. Not drafted yet.
