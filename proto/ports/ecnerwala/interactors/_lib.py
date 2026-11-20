"""Minimal interactive judge harness for proto/ports/ecnerwala/gate.sh."""
import subprocess
import sys


class PipeIO:
    def __init__(self, proc, log_in=None, log_out=None):
        self.proc = proc
        self.log_in = open(log_in, "w") if log_in else None
        self.log_out = open(log_out, "w") if log_out else None

    def send(self, text):
        if not text.endswith("\n"):
            text += "\n"
        if self.log_in:
            self.log_in.write(text)
            self.log_in.flush()
        try:
            self.proc.stdin.write(text)
            self.proc.stdin.flush()
        except BrokenPipeError:
            raise RuntimeError("solution closed stdin early")

    def recv_line(self):
        line = self.proc.stdout.readline()
        if not line:
            raise RuntimeError("unexpected EOF from solution")
        if self.log_out:
            self.log_out.write(line)
            self.log_out.flush()
        return line.strip()

    def close(self):
        if self.log_in:
            self.log_in.close()
        if self.log_out:
            self.log_out.close()
        try:
            self.proc.stdin.close()
        except Exception:
            pass


def run_interactor(drive_fn):
    args = sys.argv[1:]
    case_path = args[0]
    i = 1
    log_in = log_out = None
    while i < len(args) and args[i] != "--":
        if args[i] == "--log-in":
            log_in = args[i + 1]
            i += 2
        elif args[i] == "--log-out":
            log_out = args[i + 1]
            i += 2
        else:
            raise SystemExit(f"unknown arg: {args[i]}")
    cmd = args[i + 1 :]
    if not cmd:
        raise SystemExit("missing command after --")
    case_text = open(case_path).read()
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )
    io = PipeIO(proc, log_in=log_in, log_out=log_out)
    try:
        drive_fn(case_text, io)
    except Exception as e:
        proc.kill()
        err = proc.stderr.read()
        io.close()
        sys.stderr.write(f"interactor error: {e}\n{err}\n")
        sys.exit(1)
    io.close()
    rc = proc.wait(timeout=10)
    err = proc.stderr.read()
    if rc != 0:
        sys.stderr.write(f"solution exited with code {rc}\n{err}\n")
        sys.exit(1)
