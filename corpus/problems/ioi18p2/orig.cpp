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

const int dx[] = {0, 0, -1, 1}, dy[] = {-1, 1, 0, 0};

struct Node {
    int mn, cnt;

    void apply_lazy(int x) {
        mn += x;
    }

    Node operator+(const Node& n) const {
        if (mn < n.mn) {
            return *this;
        } else if (n.mn < mn) {
            return n;
        }
        return {mn, cnt + n.cnt};
    }
};

struct DS {
    int n;
    vector<Node> v;
    vector<int> lazy;

    DS() {}
    DS(int n): n(n), v(4 * n, {0, 1}), lazy(4 * n) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o] += x;
            v[o].apply_lazy(x);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = v[lc] + v[rc];
        v[o].apply_lazy(lazy[o]);
    }

    void update(int l, int r, int x) {
        if (l > r) {
            return;
        }
        update(1, 0, n - 1, l, r, x);
    }

    int query() const {
        return v[1].cnt;
    }
} ds;

int n, m;
vector<vector<int>> arr;
vector<pair<int, int>> pos;

bool ibs(int x, int y) {
    return 0 <= x && x < n && 0 <= y && y < m;
}

void update1(int x, int y, int v) {
    int ans = n * m;
    auto go = [&](int x, int y) -> void {
        if (ibs(x, y)) {
            ans = min(ans, arr[x][y]);
        }
    };
    go(x - 1, y);
    go(x, y - 1);
    ds.update(arr[x][y], ans - 1, v);
}

void update2(int x, int y, int v) {
    int adj[4];
    for (int i = 0; i < 4; i++) {
        int cx = x + dx[i], cy = y + dy[i];
        if (ibs(cx, cy)) {
            adj[i] = arr[cx][cy];
        } else {
            adj[i] = 1e9;
        }
    }
    sort(begin(adj), end(adj));
    ds.update(adj[1], arr[x][y] - 1, v);
}

void update(int x, int y, int v) {
    update1(x, y, v);
    update2(x, y, v);
}

void set_pos(int u) {
    auto& [x, y] = pos[u];
    arr[x][y] = u;
}

void give_initial_chart(int _n, int _m, vector<int> r, vector<int> c) {
    n = _n;
    m = _m;
    ds = DS(n * m);
    pos.resize(n * m);
    arr.resize(n, vector<int>(m));
    for (int i = 0; i < n * m; i++) {
        pos[i] = {r[i], c[i]};
        set_pos(i);
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            update(i, j, 1);
        }
    }
}

int swap_seats(int u, int v) {
    set<pair<int, int>> changed;
    for (auto& [x, y] : {pos[u], pos[v]}) {
        changed.emplace(x, y);
        for (int i = 0; i < 4; i++) {
            int cx = x + dx[i], cy = y + dy[i];
            if (ibs(cx, cy)) {
                changed.emplace(cx, cy);
            }
        }
    }
    for (auto& [x, y] : changed) {
        update(x, y, -1);
    }
    swap(pos[u], pos[v]);
    set_pos(u);
    set_pos(v);
    for (auto& [x, y] : changed) {
        update(x, y, 1);
    }
    return ds.query();
}
// The original is the functions the IOI grader calls. This `main` stands in for
// the grader: h w q, the h * w seats (row, column), then q swaps `a b`.
int main() {
	int h, w, q;
	if (scanf("%d %d %d", &h, &w, &q) != 3) return 1;
	vector<int> r(h * w), c(h * w);
	for (int i = 0; i < h * w; i++) scanf("%d %d", &r[i], &c[i]);
	give_initial_chart(h, w, r, c);
	while (q--) {
		int a, b;
		scanf("%d %d", &a, &b);
		printf("%d\n", swap_seats(a, b));
	}
}
