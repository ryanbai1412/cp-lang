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
#define sz(x) int(std::size(x))

struct DS {
    static constexpr int maxn = 1e4 + 5;

    struct N {
        int v, lc, rc;
    } heap[1 << 23] {};
    int heap_ptr = 1, root = 0;
    vector<int> upds {-1};

    int alloc() {
        return heap_ptr++;
    }
    void alloc(int& o) {
        if (!o) {
            o = alloc();
        }
    }

    void update1(int& o, int l, int r, int ql, int qr, int x) {
        alloc(o);
        auto& [v, lc, rc] = heap[o];
        if (ql <= l && r <= qr) {
            v = x;
            return;
        }
        int mid = (l + r) / 2;
        if (v) {
            alloc(lc);
            alloc(rc);
            heap[lc].v = v;
            heap[rc].v = v;
        }
        if (ql <= mid) {
            update1(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update1(rc, mid + 1, r, ql, qr, x);
        }
    }

    void update2(int& o, int l, int r, int xl, int xr, int yl, int yr, int x) {
        alloc(o);
        auto& [v, lc, rc] = heap[o];
        if (xl <= l && r <= xr) {
            update1(v, 0, maxn, yl, yr, x);
            return;
        }
        int mid = (l + r) / 2;
        if (xl <= mid) {
            update2(lc, l, mid, xl, xr, yl, yr, x);
        }
        if (mid < xr) {
            update2(rc, mid + 1, r, xl, xr, yl, yr, x);
        }
    }

    void update(int xl, int xr, int yl, int yr, int x) {
        update2(root, 0, maxn, xl, xr, yl, yr, sz(upds));
        upds.push_back(x);
    }

    int query1(int o, int l, int r, int ind) const {
        if (!o) {
            return 0;
        }
        auto& [v, lc, rc] = heap[o];
        if (l == r) {
            return v;
        }
        int mid = (l + r) / 2;
        if (ind <= mid) {
            return max(v, query1(lc, l, mid, ind));
        } else {
            return max(v, query1(rc, mid + 1, r, ind));
        }
    }

    int query2(int o, int l, int r, int ind, int yind) const {
        if (!o) {
            return 0;
        }
        auto& [v, lc, rc] = heap[o];
        if (l == r) {
            return query1(v, 0, maxn, yind);
        }
        int mid = (l + r) / 2;
        if (ind <= mid) {
            return max(query1(v, 0, maxn, yind), query2(lc, l, mid, ind, yind));
        } else {
            return max(query1(v, 0, maxn, yind),
                       query2(rc, mid + 1, r, ind, yind));
        }
    }

    int query(int x, int y) const {
        return upds[query2(root, 0, maxn, x, y)];
    }
};

void solve() {
    int _dim, n;
    cin >> _dim >> _dim >> n;
    int arr[n][4];
    for (auto& [x1, y1, x2, y2] : arr) {
        cin >> x1 >> y2 >> x2 >> y1;
    }
    DS ds;
    for (int i = 0; i < n; i++) {
        auto& [x1, y1, x2, y2] = arr[i];
        ds.update(x1, x2, y1, y2, i);
    }
    int q;
    cin >> q;
    while (q--) {
        int x, y;
        cin >> x >> y;
        int ci = ds.query(x, y);
        if (ci == -1) {
            cout << 0 << endl;
            continue;
        }
        cout << ci + 1 << endl;
        auto& [x1, y1, x2, y2] = arr[ci];
        ds.update(x1, x2, y1, y2, ci);
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}