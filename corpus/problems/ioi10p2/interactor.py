#!/usr/bin/env python3
# Judge for IOI 2010 Hotter Colder, stdio-adapted with the games in lockstep, so the
# statement's 1 000 000 calls of HC fit the judge budget. It sends Q and the Q values of N;
# each round the program prints one token per unfinished game, in order: a guess G
# (1 <= G <= N) or -X to answer X. The judge replies with one line, a token per guess: 0 for
# a game's first guess, then 1 (hotter), -1 (colder) or 0 (same) against its previous guess.
# Each game may use at most W guesses, 2^W <= 3N < 2^(W+1), the full score in every subtask.
# The case is Q, then Q lines `N secret`. The rounds are judged by the C++ below, built like
# a reference; this script relays its lines.
import os, subprocess, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "lib"))
sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "scripts"))
from interactor_lib import run_interactor

JUDGE = r"""
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
int main() {
    int q;
    if (scanf("%d", &q) != 1 || q < 1 || q > 1000000) return puts("E bad case"), 0;
    vector<ll> n(q), s(q), prev(q, 0);
    vector<int> used(q, 0), lim(q), act(q);
    string first = "S " + to_string(q) + " ";
    for (int i = 0; i < q; i++) {
        if (scanf("%lld %lld", &n[i], &s[i]) != 2 || s[i] < 1 || s[i] > n[i] || n[i] > 500000000)
            return puts("E bad case"), 0;
        lim[i] = 63 - __builtin_clzll(3 * n[i]), act[i] = i;
        first += to_string(n[i]) + (i + 1 < q ? ' ' : '\n');
    }
    fputs(first.c_str(), stdout), fflush(stdout);
    int c;
    while ((c = getchar()) != '\n' && c != EOF) {}
    string line, reply;
    int most = 0;
    while (true) {
        if (!getline(cin, line)) return puts("E no round"), 0;
        const char* p = line.c_str();
        char* e;
        vector<int> next;
        reply = "R";
        for (int i : act) {
            ll t = strtoll(p, &e, 10);
            if (e == p) return printf("E game %d: missing token\n", i + 1), 0;
            p = e;
            if (t < 0) {
                if (-t != s[i]) return printf("E game %d: answered %lld, Jill had %lld\n", i + 1, -t, s[i]), 0;
                most = max(most, used[i]);
                continue;
            }
            if (t < 1 || t > n[i] || ++used[i] > lim[i])
                return printf("E game %d: guess %d is %lld (N = %lld, limit W = %d)\n", i + 1, used[i], t, n[i], lim[i]), 0;
            ll a = llabs(t - s[i]), b = llabs(prev[i] - s[i]);
            reply += used[i] == 1 ? " 0" : a < b ? " 1" : a > b ? " -1" : " 0";
            prev[i] = t, next.push_back(i);
        }
        while (*p == ' ') p++;
        if (*p) return puts("E extra tokens"), 0;
        act.swap(next);
        if (act.empty()) return printf("D %d\n", most), 0;
        puts(reply.c_str()), fflush(stdout);
    }
}
"""


def drive(case_text, io):
    import corpus
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, "judge.cpp")
        with open(src, "w") as f:
            f.write(JUDGE)
        exe = corpus.binary(src)
    judge = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    judge.stdin.write(case_text.rstrip("\n") + "\n")
    judge.stdin.flush()
    while True:
        kind, _, rest = judge.stdout.readline().rstrip("\n").partition(" ")
        if kind == "D":
            sys.stderr.write(f"most guesses in a game: {rest}\n")
            return
        if kind not in ("S", "R"):
            raise RuntimeError(rest or "judge died")
        io.send(rest if kind == "R" else rest.replace(" ", "\n", 1))
        judge.stdin.write(io.recv_line() + "\n")
        judge.stdin.flush()


run_interactor(drive)
