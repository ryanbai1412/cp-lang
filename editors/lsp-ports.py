#!/usr/bin/env python3
"""python3 editors/lsp-ports.py [--cpl PATH] [--json OUT] [IDS...]

Drives `cpl lsp` over the hill-climb ports (corpus/hillclimb.json, each
corpus/problems/<id>/spec.cp) the way an editor does and reports, per
feature, on how many ports it works:

  diagnostics  published on open, no internal error, and the same errors
               (line and code) as `cpl check` in the same environment
  hover        every identifier answers without error, and every name a
               `let`/`var`/`fn`/`struct`/`domain` declares shows its type
  types        no declared name's hover type is `_` (unknown to the checker)
  storage      every declared array, struct or optional also shows its
               storage (bytes, and for structs the field layout)
  completion   after every `name.` the server offers at least one item
  definition   every use of a name declared in the file resolves
  latency      median didChange -> publishDiagnostics under 50 ms

A request that gets no answer within 5 s counts as a hang and fails every
feature of that port.
"""
import json
import os
import pathlib
import queue
import re
import statistics
import subprocess
import sys
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
TIMEOUT = 5.0
KEYWORDS = set(
    "let var fn struct enum domain type if else elif for in while return break continue "
    "and or not match case true false none use as from trait impl mut lends yield pub "
    "unchecked invariant assert test with where".split()
)
DECL = re.compile(r"\b(let|var|fn|struct|enum|domain|type)\s+(mut\s+)?\(?([A-Za-z_][A-Za-z0-9_]*)")
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
FEATURES = ["diagnostics", "hover", "types", "storage", "completion", "definition", "latency"]


class Hang(Exception):
    pass


class Client:
    def __init__(self, command):
        self.proc = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                     stderr=subprocess.DEVNULL)
        self.inbox = queue.Queue()
        self.next_id = 0
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        out = self.proc.stdout
        while True:
            length = None
            while True:
                line = out.readline()
                if not line:
                    self.inbox.put(None)
                    return
                if line in (b"\r\n", b"\n"):
                    break
                name, _, value = line.decode().partition(":")
                if name.lower() == "content-length":
                    length = int(value)
            self.inbox.put(json.loads(out.read(length)))

    def send(self, message):
        body = json.dumps({"jsonrpc": "2.0", **message}).encode()
        self.proc.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        self.proc.stdin.flush()

    def wait(self, want):
        deadline = time.monotonic() + TIMEOUT
        while True:
            left = deadline - time.monotonic()
            try:
                message = self.inbox.get(timeout=max(left, 0.001))
            except queue.Empty:
                raise Hang("no reply in %.0f s" % TIMEOUT)
            if message is None:
                raise Hang("the server exited")
            if want(message):
                return message

    def request(self, method, params):
        self.next_id += 1
        rid = self.next_id
        self.send({"id": rid, "method": method, "params": params})
        return self.wait(lambda m: m.get("id") == rid)

    def notify_and_wait_diagnostics(self, method, params, uri):
        self.send({"method": method, "params": params})
        return self.wait(lambda m: m.get("method") == "textDocument/publishDiagnostics"
                         and m["params"]["uri"] == uri)

    def close(self):
        try:
            self.send({"id": 0, "method": "shutdown"})
            self.send({"method": "exit"})
            self.proc.wait(timeout=2)
        except Exception:
            self.proc.kill()


def code_mask(text):
    """Text with comments and string literals blanked, so tokens found in it are code."""
    out = list(text)
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c == "/" and text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            out[i:j] = " " * (j - i)
            i = j
        elif c == '"':
            j = i + 1
            while j < n and text[j] != '"' and text[j] != "\n":
                j += 2 if text[j] == "\\" else 1
            j = min(j + 1, n)
            out[i:j] = " " * (j - i)
            i = j
        else:
            i += 1
    return "".join(out)


def position(text, offset):
    line = text.count("\n", 0, offset)
    start = text.rfind("\n", 0, offset) + 1
    return {"line": line, "character": len(text[start:offset].encode("utf-16-le")) // 2}


def check_errors(cpl, path):
    run = subprocess.run([cpl, "check", str(path)], capture_output=True, text=True)
    errors = set()
    lines = run.stderr.splitlines()
    for i, line in enumerate(lines):
        m = re.match(r"error\[(\w+)\]", line)
        if m:
            for nxt in lines[i + 1:i + 3]:
                at = re.search(r":(\d+):\d+$", nxt.strip())
                if at:
                    errors.add((int(at.group(1)), m.group(1)))
                    break
    return errors


def hover_text(reply):
    result = reply.get("result")
    if not result:
        return None
    contents = result.get("contents")
    return contents.get("value") if isinstance(contents, dict) else str(contents)


def needs_storage(first_line):
    _, _, ty = first_line.partition(": ")
    ty = ty.strip()
    if first_line.startswith("fn "):
        return False
    return ty.startswith("[") or ty.startswith("?") or first_line.startswith("struct ")


def measure(cpl, pid, latency_edits):
    path = ROOT / "corpus/problems" / pid / "spec.cp"
    text = path.read_text()
    uri = path.as_uri()
    code = code_mask(text)
    result = {f: False for f in FEATURES}
    notes = []
    client = Client([cpl, "lsp"])
    try:
        client.request("initialize", {"processId": None, "rootUri": ROOT.as_uri(), "capabilities": {}})
        client.send({"method": "initialized", "params": {}})
        published = client.notify_and_wait_diagnostics("textDocument/didOpen", {"textDocument": {
            "uri": uri, "languageId": "cp", "version": 1, "text": text}}, uri)
        diags = published["params"]["diagnostics"]
        internal = [d for d in diags if "internal error" in d.get("message", "")]
        got = {(d["range"]["start"]["line"] + 1, d.get("code"))
               for d in diags if d.get("severity") == 1}
        want = check_errors(cpl, path)
        result["diagnostics"] = not internal and got == want
        if internal:
            notes.append("diagnostics: " + internal[0]["message"][:160])
        elif got != want:
            notes.append(f"diagnostics: lsp {sorted(got)[:4]} vs check {sorted(want)[:4]}")

        decls = {}
        for m in DECL.finditer(code):
            decls.setdefault(m.group(3), m.start(3))
        hover_ok, types_ok, storage_ok, def_ok = True, True, True, True
        for m in IDENT.finditer(code):
            name = m.group(0)
            if name in KEYWORDS or name in ("Some", "None", "_") or name[0].isdigit():
                continue
            before = code[:m.start()].rstrip()
            after = code[m.end():]
            member = code[m.start() - 1:m.start()] == "."
            label = after.startswith(":") and not after.startswith("::") and before[-1:] in ("(", ",")
            at = position(text, m.start() + len(name) // 2)
            doc = {"textDocument": {"uri": uri}, "position": at}
            reply = client.request("textDocument/hover", doc)
            if "error" in reply:
                hover_ok = False
                notes.append(f"hover {pid}:{at['line']+1}: {reply['error']['message'][:120]}")
                continue
            shown = hover_text(reply)
            if decls.get(name) == m.start():
                first = shown.split("```cp\n", 1)[-1].split("\n", 1)[0] if shown else ""
                if not shown or (":" not in first and not first.startswith(("fn ", "struct ", "domain ", "type ", "enum "))):
                    hover_ok = False
                    notes.append(f"hover {name}@{at['line']+1}: {first[:80]!r}")
                elif first.endswith(": _"):
                    types_ok = False
                    notes.append(f"types {name}@{at['line']+1}: {first[:80]!r}")
                elif needs_storage(first) and "storage" not in shown:
                    storage_ok = False
                    notes.append(f"storage {name}@{at['line']+1}: {first[:80]!r}")
            elif name in decls and m.start() > decls[name] and not member and not label:
                reply = client.request("textDocument/definition", doc)
                if "error" in reply or not reply.get("result"):
                    def_ok = False
                    notes.append(f"definition {name}@{at['line']+1}:{at['character']+1}: {reply.get('error', {}).get('message', 'null')[:100]}")
        completion_ok = True
        for m in re.finditer(r"[A-Za-z_)\]][A-Za-z0-9_]*\.(?=[A-Za-z_])", code):
            at = position(text, m.end())
            reply = client.request("textDocument/completion", {"textDocument": {"uri": uri}, "position": at})
            items = reply.get("result")
            items = items.get("items", []) if isinstance(items, dict) else (items or [])
            if "error" in reply or not items:
                completion_ok = False
                notes.append(f"completion {m.group(0)}@{at['line']+1}:{at['character']+1}: {reply.get('error', {}).get('message', 'empty')[:100]}")
        result["hover"], result["types"], result["storage"] = hover_ok, types_ok, storage_ok
        result["definition"], result["completion"] = def_ok, completion_ok

        samples = []
        lines = text.split("\n")
        version = 1
        for k in range(latency_edits):
            line = (k * 7919) % max(len(lines) - 1, 1)
            for edit in ("x", ""):
                version += 1
                change = {"range": {"start": {"line": line, "character": 0},
                                    "end": {"line": line, "character": 1 if edit == "" else 0}},
                          "text": edit}
                start = time.perf_counter()
                client.notify_and_wait_diagnostics("textDocument/didChange", {
                    "textDocument": {"uri": uri, "version": version},
                    "contentChanges": [change]}, uri)
                samples.append((time.perf_counter() - start) * 1000)
        result["latency_ms"] = statistics.median(samples) if samples else None
        result["latency"] = bool(samples) and statistics.median(samples) < 50
        if not result["latency"]:
            notes.append(f"latency: median {result['latency_ms']:.1f} ms")
    except Hang as hang:
        result = {f: False for f in FEATURES}
        notes.append(f"hang: {hang}")
    finally:
        client.close()
    result["notes"] = notes
    return result


def main():
    args = sys.argv[1:]
    cpl = str(ROOT / "target/release/cpl")
    out = None
    while args and args[0].startswith("--"):
        flag = args.pop(0)
        if flag == "--cpl":
            cpl = args.pop(0)
        elif flag == "--json":
            out = args.pop(0)
    ids = args or json.loads((ROOT / "corpus/hillclimb.json").read_text())["problems"]
    from concurrent.futures import ThreadPoolExecutor
    with ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as pool:
        results = dict(zip(ids, pool.map(lambda p: measure(cpl, p, 10), ids)))
    for pid, r in results.items():
        for note in r["notes"][:3]:
            print(f"{pid}: {note}")
    print()
    for f in FEATURES:
        print(f"{f:12} {sum(r[f] for r in results.values())}/{len(ids)}")
    lat = sorted(r["latency_ms"] for r in results.values() if r.get("latency_ms") is not None)
    if lat:
        print(f"latency ms   median {statistics.median(lat):.1f}, max {lat[-1]:.1f}")
    if out:
        pathlib.Path(out).write_text(json.dumps(results, indent=1))


if __name__ == "__main__":
    main()
