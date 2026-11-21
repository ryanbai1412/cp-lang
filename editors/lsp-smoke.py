#!/usr/bin/env python3
"""python3 editors/lsp-smoke.py EDITOR COMMAND [ARGS...]

Starts the language server the way EDITOR's setup does (COMMAND is read from
that setup by its smoke script), then checks the handshake every client makes:
`initialize` answers with the capabilities the editors rely on, and opening a
file with a type error publishes that error on its line. For editors that cannot
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
    send({"id": 2, "method": "shutdown"})
    until(lambda m: m.get("id") == 2)
    send({"method": "exit"})
    if server.wait(timeout=10) != 0:
        fail(f"exit status {server.returncode}")
print(f"{editor} lsp smoke: ok ({errors[0]['code']} on line 3)")
