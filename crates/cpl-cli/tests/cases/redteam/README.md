# Red-team cases

The smallest programs that used to be surprising or broken at the edges of
the language, one per file. `plan/19-index-and-lends.md` section 5.6
(cases 11 to 23) says what each must do.

`x.cp` is built, run on `x.in`, and its output and error text together
must equal `x.out`. `../../run_all.sh` builds them the way a local run
does (`-DLOCAL`) with AddressSanitizer and UndefinedBehaviorSanitizer on,
so a program that has to stop does so with a `line N: ...` message and
never by undefined behavior. The `trap_*` files are those programs.

The cases that must be rejected by the compiler are `../bad/rt_*.cp`, each
with the text of its error on the first line.

To add a case, write the three files and check the `.out` by hand: it is
the specification, not a recording of what the compiler does today.
