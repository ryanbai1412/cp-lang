# Friction reports

One file per port, `NAME.md`, written by whoever ports `NAME`. Triage reads
them all and clusters them into general mechanisms; a report is closed when
the fix lands and the port that raised it uses it.

Each friction is one section:

    ## short title
    - **Where:** the lines of `NAME.cp` (quote the snippet).
    - **What felt wrong:** hacky, repeated, unsafe, slow, unguessable, a
      fact written twice, a revision that would not stay local.
    - **Goal it hurts:** express, hard-to-get-wrong, performance, evidence,
      submission, accumulate, or coherence.
    - **Ideal code:** what you wish you could have written, even if no
      compiler accepts it today.
    - **General fix:** the mechanism (core, stdlib, sugar or tooling) that
      removes this and its whole class, and the other ports it would help.
      "Add a helper for this problem" is not a general fix.
    - **Severity:** blocker (could not port), high (wrong-answer risk or
      large cost), medium, low.

End the file with the measurements: contestant-written tokens of the port
and the original, runtime and peak memory of both on the largest input, and
emitted C++ bytes.
