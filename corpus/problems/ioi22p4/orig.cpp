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
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 2e5 + 5;
const long mod = 1e9 + 2022;

pair<long, long> operator+(pair<long, long> a, pair<long, long> b) {
    return {a.first + b.first, a.second + b.second};
}

struct DS {
    int n;
    vector<int> lazy;
    vector<pair<long, long>> arr, v;

    DS() {}
    DS(vector<int> v, vector<int> on)
        : n(sz(v)), lazy(4 * n), arr(n), v(4 * n) {
        for (int i = 0; i < n; i++) {
            arr[i] = {0, v[i]};
            if (on[i]) {
                swap(arr[i].first, arr[i].second);
            }
        }
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = arr[l];
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void update(int o, int l, int r, int ql, int qr) {
        if (ql <= l && r <= qr) {
            swap(v[o].first, v[o].second);
            lazy[o] ^= 1;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr);
        }
        v[o] = v[lc] + v[rc];
        if (lazy[o]) {
            swap(v[o].first, v[o].second);
        }
    }

    void update(int l, int r) {
        update(1, 0, n - 1, l, r);
    }

    int query() const {
        return int(v[1].first % mod);
    }
} ds;

int n;
vector<int> arr, graph[maxn];
long ways[maxn], contrib[maxn];

void pdfs(int u) {
    ways[u] = 1;
    for (auto& v : graph[u]) {
        pdfs(v);
        ways[u] = (ways[u] * ways[v]) % mod;
    }
    if (sz(graph[u])) {
        ways[u] = (ways[u] * sz(graph[u])) % mod;
    }
    dbg(u, ways[u]);
}

void dfs(int u, long x = 1) {
    int m = sz(graph[u]);
    if (!m) {
        dbg(u, x);
        contrib[u] = x;
        return;
    }
    long psum[m + 1], ssum[m + 1];
    psum[0] = ssum[m] = 1;
    for (int i = 0; i < m; i++) {
        psum[i + 1] = (psum[i] * ways[graph[u][i]]) % mod;
    }
    for (int i = m - 1; i >= 0; i--) {
        ssum[i] = (ssum[i + 1] * ways[graph[u][i]]) % mod;
    }
    for (int i = 0; i < m; i++) {
        dfs(graph[u][i], x * psum[i] % mod * ssum[i + 1] % mod);
    }
}

void init(int _n, int m, vector<int> p, vector<int> _arr) {
    n = _n;
    arr = _arr;
    for (int i = 1; i < n + m; i++) {
        graph[p[i]].push_back(i);
    }
    pdfs(0);
    dfs(0);
    ds = DS(vector<int>(contrib + n, contrib + n + m), arr);
}

int count_ways(int l, int r) {
    l -= n;
    r -= n;
    ds.update(l, r);
    return ds.query();
}
// The original is the two functions the IOI grader calls. This `main` stands in for the
// grader: n m q, the n + m parents, the m source states, then q toggles `l r`; it prints
// count_ways per toggle.
int main() {
	int n, m, q;
	if (scanf("%d %d %d", &n, &m, &q) != 3) return 1;
	vector<int> p(n + m), a(m);
	for (int &x : p) scanf("%d", &x);
	for (int &x : a) scanf("%d", &x);
	init(n, m, p, a);
	while (q--) {
		int l, r;
		scanf("%d %d", &l, &r);
		printf("%d\n", count_ways(l, r));
	}
}
