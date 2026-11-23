# Playground

> 25 minutes into a Div. 2 round, on a laptop without CP Lang installed.
> You have an idea for C and want to see whether `scan` and `first_true`
> do what you think before you commit to it.

You open cp-lang.org/play. The editor has a small program in it; you select
all and type. As you type, completion offers `first_true` after `.fi` with
its signature and cost, parameter hints show `by:` and its default, and an
inlay hint shows the type of each untyped `let`. A red underline appears
under a wrong argument within half a second of your last keystroke, with
the same message `cpl check` would print, and a link to its error page. You
paste the sample into the input pane, press Run (Cmd+Enter) and see the
output and the time. You Cmd+click `first_true` and its reference entry
opens beside you. Satisfied, you open the C++ tab, copy one file and submit.

**Off the happy path.** The program traps: the error names the line and
the local values in source terms, and Trace shows the state that went
wrong. It is wrong, not crashing: the Stress tab runs your solution against
a brute force on small generated inputs and shows the first input where they
disagree. You reload the tab by accident: your code is still there. You want
a teammate to look: Share gives a link that opens the same code and input.

**Ideal against today.** Today the playground does all of the above except
the Stress tab being tested in the browser, sharing that survives the share
service being down, and diagnostics that keep their place in a file with a
broken line above. Those are listed as gaps in the use cases.

**Dimensions.** Program state (empty, partial, type-correct, trapping,
wrong), input (none, sample, large), surface (playground, embedded example
with a collapsed setup, practice judge), device (desktop, phone, keyboard
only, screen reader), and network (online, wasm still loading, a service
down).
