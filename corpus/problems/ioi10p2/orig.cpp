// IOI 2010 Hotter Colder. No accepted submission exists (the one C++, 5109023, is WA). HC
// follows a public 100-point solution (hoanxuanbach/Competitive-Programming,
// OI/IOI/2010/hottercolder.cpp): against a wall it guesses x, then y, splitting at a point
// that leaves a tail of the next smaller wall size, and in the middle it reflects the last
// guess about the candidates' centre, at most W guesses (2^W <= 3N) in every game. Written as
// a resumable state per game, since the main stands in for the grader with all games in
// lockstep: read Q and the Q values of N; each round print one token per unfinished game,
// in order, a guess G or -X to answer X, then read one reply (1, -1 or 0) per guess.
#include <bits/stdc++.h>
#include <unistd.h>
using namespace std;
typedef long long ll;

static ll tail(ll n) { return n < 5 ? 1 : ((n + 1) >> 1) - tail(n >> 1); }

struct Game {
    ll n, l = 1, r, w, x = 0, y = 0;
    int phase = 0;
    ll outer() {
        if (l >= r) return -l;
        if (r == 2) return phase = 1, 1;
        ll s = (2LL << (w - 2)) / 3 + 2 - (w & 1);
        y = r == n ? s + tail(r - s) : 2 * s - 1, x = 2 * s - y;
        return phase = 3, x;
    }
    ll inner() {
        if (l >= r) return w -= 2, outer();
        y = ((l + r) >> 1 << 1) - x;
        if (x == y) y++;
        y = min(n, max(y, 1LL));
        return phase = 5, y;
    }
    ll step(int k) {  // the reply to the last guess -> the next token
        switch (phase) {
        case 1: return phase = 2, 2;
        case 2: return k < 0 ? -1 : -2;
        case 3: return phase = 4, y;
        case 4:
            if (k == -1) return r = (x + y - 1) >> 1, w -= 2, outer();
            if (k == 0) return -((x + y) >> 1);
            l = (x + y + 2) >> 1, x = y;
            return inner();
        case 5:
            if (k == 0) return -((x + y) >> 1);
            if ((k == 1) ^ (x > y)) l = (x + y + 2) >> 1;
            else r = (x + y - 1) >> 1;
            x = y;
            return inner();
        }
        return outer();
    }
};

#ifndef SIM
static char ibuf[1 << 16];
static size_t ilen, ipos;
static int gc() {
    if (ipos == ilen) {  // read(2), not fread: a reply line is all there is until we answer it
        ssize_t got = read(0, ibuf, sizeof ibuf);
        ilen = got > 0 ? got : 0, ipos = 0;
    }
    return ipos < ilen ? ibuf[ipos++] : -1;
}
static ll readInt() {
    int c = gc();
    while (c == ' ' || c == '\n' || c == '\r') c = gc();
    if (c < 0) exit(1);
    bool neg = c == '-';
    if (neg) c = gc();
    ll v = 0;
    for (; c >= '0' && c <= '9'; c = gc()) v = v * 10 + c - '0';
    return neg ? -v : v;
}

int main() {
    int q = readInt();
    vector<Game> g(q);
    vector<int> act(q);
    vector<ll> tok(q);
    for (int i = 0; i < q; i++) {
        g[i].n = g[i].r = readInt();
        g[i].w = 63 - __builtin_clzll(3 * g[i].n);
        act[i] = i, tok[i] = g[i].step(0);
    }
    string line;
    while (!act.empty()) {
        line.clear();
        vector<int> next;
        for (int i : act) {
            line += to_string(tok[i]), line += ' ';
            if (tok[i] > 0) next.push_back(i);
        }
        line += '\n';
        fwrite(line.data(), 1, line.size(), stdout);
        fflush(stdout);
        for (int i : next) tok[i] = g[i].step(readInt());
        act.swap(next);
    }
}
#endif
