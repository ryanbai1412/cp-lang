#!/usr/bin/env python3
"""python3 editors/lsp-smoke.py EDITOR COMMAND [ARGS...]

Starts the language server the way EDITOR's setup does (COMMAND is read from
that setup by its smoke script), then checks the handshake every client makes:
`initialize` answers with the capabilities the editors rely on, opening a file
with a type error publishes that error on its line, completion, definition
and formatting answer as the editors' own smoke tests expect, and a lint
links to its page with a quick fix that applies. For editors that cannot
run headless (Helix, Zed), this is the part of the setup a script can drive.
"""
import json
import pathlib
import subprocess
import sys
import tempfile

editor, command = sys.argv[1], sys.argv[2:]


def fail(what):
    print(f"{editor} lsp smoke: {what}", file=sys.stderr)
    sys.exit(1)


server = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE)


def send(message):
    body = json.dumps({"jsonrpc": "2.0", **message}).encode()
    server.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    server.stdin.flush()


def receive():
    length = None
    while True:
        line = server.stdout.readline()
        if not line:
            fail("the server exited")
        if line in (b"\r\n", b"\n"):
            break
        name, _, value = line.decode().partition(":")
        if name.lower() == "content-length":
            length = int(value)
    return json.loads(server.stdout.read(length))


def apply(text, edits):
    lines = text.split("\n")
    offset = lambda at: sum(len(line) + 1 for line in lines[:at["line"]]) + at["character"]
    for e in sorted(edits, key=lambda e: offset(e["range"]["start"]), reverse=True):
        text = text[:offset(e["range"]["start"])] + e["newText"] + text[offset(e["range"]["end"]):]
    return text


def ask(id, method, uri, line=0, character=0, **params):
    send({"id": id, "method": method, "params": {
        "textDocument": {"uri": uri}, "position": {"line": line, "character": character}, **params}})
    return until(lambda m: m.get("id") == id).get("result")


def until(want):
    for _ in range(1000):
        message = receive()
        if want(message):
            return message
    fail("no reply")


with tempfile.TemporaryDirectory() as root:
    file = pathlib.Path(root, "a.cp")
    text = "fn main():\n    let n: Int = read()\n    let s: Str = n\n    print(n + 1)\n"
    file.write_text(text)
    uri = file.as_uri()
    send({"id": 1, "method": "initialize", "params": {
        "processId": None,
        "rootUri": pathlib.Path(root).as_uri(),
        "clientInfo": {"name": editor},
        "capabilities": {"textDocument": {"publishDiagnostics": {}}},
    }})
    reply = until(lambda m: m.get("id") == 1)
    capabilities = reply.get("result", {}).get("capabilities", {})
    wanted = ["hoverProvider", "completionProvider", "definitionProvider", "renameProvider",
              "codeActionProvider", "documentFormattingProvider", "inlayHintProvider"]
    missing = [c for c in wanted if not capabilities.get(c)]
    if missing:
        fail(f"initialize lacks {missing}: {reply}")
    send({"method": "initialized", "params": {}})
    send({"method": "textDocument/didOpen", "params": {"textDocument": {
        "uri": uri, "languageId": "cp", "version": 1, "text": text}}})
    published = until(lambda m: m.get("method") == "textDocument/publishDiagnostics"
                      and m["params"]["uri"] == uri)
    errors = [d for d in published["params"]["diagnostics"] if d.get("severity") == 1]
    if [d["range"]["start"]["line"] for d in errors] != [2]:
        fail(f"expected one error, on `let s: Str = n`: {published}")
    # Completion inside `print` lists the prelude and the locals in scope.
    items = ask(3, "textDocument/completion", uri, 3, 6) or []
    labels = {item["label"] for item in (items.get("items", []) if isinstance(items, dict) else items)}
    if not {"print", "n"} <= labels:
        fail(f"completion lacks print or n: {sorted(labels)[:20]}")
    # The `n` in `print(n + 1)` is defined by its `let`.
    found = ask(4, "textDocument/definition", uri, 3, 10)
    found = found[0] if isinstance(found, list) and found else found
    if not found or found.get("range", found.get("targetSelectionRange", {})).get("start", {}).get("line") != 1:
        fail(f"definition of n is {found}")
    # Formatting gives `cpl fmt`'s layout.
    messy = pathlib.Path(root, "messy.cp")
    messy.write_text("let x=1\nprint(x)\n")
    send({"method": "textDocument/didOpen", "params": {"textDocument": {
        "uri": messy.as_uri(), "languageId": "cp", "version": 1, "text": messy.read_text()}}})
    edits = ask(5, "textDocument/formatting", messy.as_uri(), options={"tabSize": 4, "insertSpaces": True})
    if apply(messy.read_text(), edits or []) != "let x = 1\nprint(x)\n":
        fail(f"formatting gave {edits}")
    # A lint is a warning linked to its page, and its quick fix rewrites the line.
    linted = pathlib.Path(root, "lint.cp")
    linted.write_text("var x = 1\nx = x + 1\nprint(x)\n")
    send({"method": "textDocument/didOpen", "params": {"textDocument": {
        "uri": linted.as_uri(), "languageId": "cp", "version": 1, "text": linted.read_text()}}})
    # The checker's report comes first; the lints follow in a second report.
    published = until(lambda m: m.get("method") == "textDocument/publishDiagnostics"
                      and m["params"]["uri"] == linted.as_uri()
                      and any(str(d.get("code", "")).startswith("L") for d in m["params"]["diagnostics"]))
    lint = next((d for d in published["params"]["diagnostics"] if d.get("code") == "L0210"), None)
    if not lint or lint.get("codeDescription", {}).get("href") != "https://cp-lang.org/lints/L0210":
        fail(f"expected L0210 linked to its page on `x = x + 1`: {published}")
    send({"id": 6, "method": "textDocument/codeAction", "params": {
        "textDocument": {"uri": linted.as_uri()}, "range": lint["range"],
        "context": {"diagnostics": [lint], "only": ["quickfix"]}}})
    actions = until(lambda m: m.get("id") == 6).get("result") or []
    edits = lambda a: a.get("edit", {}).get("changes", {}).get(linted.as_uri(), [])
    fixed = [apply(linted.read_text(), edits(a)) for a in actions]
    if "var x = 1\nx += 1\nprint(x)\n" not in fixed:
        fail(f"no quick fix writes `x += 1`: {[a.get('title') for a in actions]} give {fixed}")
    send({"id": 2, "method": "shutdown"})
    until(lambda m: m.get("id") == 2)
    send({"method": "exit"})
    if server.wait(timeout=10) != 0:
        fail(f"exit status {server.returncode}")
print(f"{editor} lsp smoke: ok ({errors[0]['code']} on line 3; completion, definition, formatting, lint fix)")
