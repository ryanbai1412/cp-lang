# The editor, as a contestant meets it

| | |
|---|---|
| Kind | Product area |
| Owner | tools |
| Changes | Edit directly; a changed design gets a use-case entry |

The editor is where the contest is written: VS Code, Neovim, Vim,
JetBrains or the site's browser editor, all talking to one server, `cpl
lsp`. Every surface below answers one question you have mid-line, without
leaving the line. Because there is one server, each editor gets the same
answers. An editor that offers less than the others is a gap in that
editor's client, not a different design.

**Completion** answers "what can I write here?": a member after `.`, an
argument's name in an open call, a name in scope, a module or item in a
`use`, a type. It has to work in every context, including while the file
doesn't parse. [completion.md](completion.md),
[use cases](completion-usecases.md).

**Hover** answers "what is this?": the type of any name, and for a library
call its declaration, documented cost and named options. Not drafted yet.

**Signature help** answers "what does this call take?" while you type its
arguments, and highlights the parameter under the cursor. Not drafted yet.

**Go to definition and references** jump into your own code and into std,
whose sources ship with the toolchain. Not drafted yet.

**Inlay hints** show the types and argument names the code leaves out,
`pow(base: 2, exp: 10)`. Not drafted yet.

**Rename and quick fixes** apply an error's or a lint's fix, or a rename,
across the file in one step. Not drafted yet.

**Install per editor** is `cpl setup vscode|neovim|vim|jetbrains`, and in
the browser it is nothing at all. Not drafted yet.
