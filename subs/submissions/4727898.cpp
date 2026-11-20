#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 1e6 + 5;

struct Node {
    bool mn;
    int lc, rc;
} heap[maxn];

pair<int, int> merge(pair<int, int> a, pair<int, int> b) {
    if (a.first <= b.first && a.second <= b.second) {
        return a;
    } else if (a.first >= b.first && a.second >= b.second) {
        return b;
    }
    assert(false);
};

pair<int, int> operator+(pair<int, int> a, pair<int, int> b) {
    return {a.first + b.first, a.second + b.second};
}

struct Comb {
    pair<int, int> lt, eq, gt;
};

string s;
int n, sind, hind, root;

char nc() {
    return s[sind++];
}

int parse() {
    if (s[sind] == '?') {
        n++;
        sind++;
        return -1;
    }
    int o = hind++;
    if (s.substr(sind, 3) == "max") {
        heap[o].mn = false;
    } else if (s.substr(sind, 3) == "min") {
        heap[o].mn = true;
    } else {
        assert(false);
    }
    sind += 3;
    assert(nc() == '(');
    heap[o].lc = parse();
    assert(nc() == ',');
    heap[o].rc = parse();
    assert(nc() == ')');
    return o;
}

Comb dfs(int o) {
    if (o == -1) {
        return {{1, 0}, {0, 0}, {0, 1}};
    }
    Comb l = dfs(heap[o].lc), r = dfs(heap[o].rc);
    if (heap[o].mn) {
        return {
            merge(l.lt, r.lt),
            merge(l.eq + r.gt, l.gt + r.eq),
            l.gt + r.gt,
        };
    } else {
        return {
            l.lt + r.lt,
            merge(l.eq + r.lt, l.lt + r.eq),
            merge(l.gt, r.gt),
        };
    }
}

void solve() {
    cin >> s;
    auto [l, r] = dfs(parse()).eq;
    cout << n - l - r << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}