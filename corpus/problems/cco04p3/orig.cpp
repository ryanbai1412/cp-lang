// CCC 2004 Stage 2 P3 Schonfinkel: reduce S/K trees to normal form. Graph reduction with sharing:
// the head of each spine is reduced in place (((K a) b) -> a, (((S a) b) c) -> ((a c)(b c)) with
// c shared), then the arguments of the head normal form are normalised in turn. Normal forms are
// unique, so this matches the statement's leftmost-outermost rule order.
#include <cstdio>
#include <string>
#include <vector>

struct Node {
    int l, r;  // l < 0: a leaf, r is 'S' or 'K'
};
static std::vector<Node> g;

static int leaf(char c) { g.push_back({-1, c}); return (int)g.size() - 1; }
static int app(int a, int b) { g.push_back({a, b}); return (int)g.size() - 1; }

static int parse(const std::string& s, size_t& i) {
    if (s[i] == '(') {
        i++;
        int a = parse(s, i);
        int b = parse(s, i);
        i++;  // ')'
        return app(a, b);
    }
    return leaf(s[i++]);
}

// Reduces t's spine until its head is S with < 3 or K with < 2 arguments.
static void whnf(int t) {
    std::vector<int> spine;
    for (;;) {
        spine.clear();
        int h = t;
        while (g[h].l >= 0) spine.push_back(h), h = g[h].l;
        int k = (int)spine.size();  // spine[k-1] is (h a1), spine[k-2] is ((h a1) a2), ...
        if (g[h].r == 'K' && k >= 2) {
            int redex = spine[k - 2], a = g[spine[k - 1]].r;
            g[redex] = g[a];
        } else if (g[h].r == 'S' && k >= 3) {
            int redex = spine[k - 3];
            int a = g[spine[k - 1]].r, b = g[spine[k - 2]].r, c = g[redex].r;
            int ac = app(a, c), bc = app(b, c);
            g[redex] = {ac, bc};
        } else {
            return;
        }
    }
}

static void normalise(int t) {
    whnf(t);
    for (int h = t; g[h].l >= 0; h = g[h].l) normalise(g[h].r);
}

static void print(int t, std::string& out) {
    if (g[t].l < 0) {
        out += (char)g[t].r;
        return;
    }
    out += '(';
    print(g[t].l, out);
    print(g[t].r, out);
    out += ')';
}

int main() {
    static char buf[1 << 16];
    while (std::fgets(buf, sizeof buf, stdin)) {
        std::string s;
        for (char* p = buf; *p; p++)
            if (*p == '(' || *p == ')' || *p == 'S' || *p == 'K') s += *p;
        if (s.empty()) break;
        g.clear();
        size_t i = 0;
        int t = parse(s, i);
        normalise(t);
        std::string out;
        print(t, out);
        std::puts(out.c_str());
    }
}
