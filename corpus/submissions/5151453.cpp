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
    static constexpr int maxn = 6e7 + 5;

    struct N {
        int v, lc, rc;
    } heap[1 << 26] {};
    int heap_ptr = 1, root = 0;

    int alloc() {
        return heap_ptr++;
    }
    void alloc(int& o) {
        if (!o) {
            o = alloc();
        }
    }

    template <typename... Args>
    void update(int& o, int, int, bool) {
        alloc(o);
    }

    template <typename... Args>
    void update(int& o, int l, int r, int ind, Args... args) {
        alloc(o);
        auto& [v, lc, rc] = heap[o];
        update(v, 0, maxn, args...);
        if (l == r) {
            return;
        }
        int mid = (l + r) / 2;
        if (ind <= mid) {
            update(lc, l, mid, ind, args...);
        } else {
            update(rc, mid + 1, r, ind, args...);
        }
    }

    void update(int x, int y, int z) {
        update(root, 0, maxn, x, y, z, false);
    }

    template <typename... Args>
    bool query(int o, int, int) const {
        return !!o;
    }

    template <typename... Args>
    bool query(int o, int l, int r, int ql, int qr, Args... args) const {
        if (!o) {
            return false;
        }
        auto& [v, lc, rc] = heap[o];
        if (ql <= l && r <= qr) {
            return query(v, 0, maxn, args...);
        }
        int mid = (l + r) / 2;
        if (ql <= mid) {
            if (query(lc, l, mid, ql, qr, args...)) {
                return true;
            }
        }
        if (mid < qr) {
            if (query(rc, mid + 1, r, ql, qr, args...)) {
                return true;
            }
        }
        return false;
    }

    bool query(int xl, int xr, int yl, int yr, int zl, int zr) const {
        return query(root, 0, maxn, xl, xr, yl, yr, zl, zr);
    }
};

void solve() {
    int n, q;
    cin >> n >> q;
    int arr[n][3];
    for (auto& [x, y, z] : arr) {
        cin >> x >> y >> z;
    }
    DS ds;
    for (auto& [x, y, z] : arr) {
        ds.update(x, y, z);
    }
    while (q--) {
        int x, y, z;
        cin >> x >> y >> z;
        auto check = [&](int kv) -> bool {
            return ds.query(x - kv, x, y - kv, y, z - kv, z);
        };
        int l = 0, r = 1e8;
        while (l < r) {
            int mid = (l + r) / 2;
            if (check(mid)) {
                r = mid;
            } else {
                l = mid + 1;
            }
        }
        if (l == 1e8) {
            cout << -1 << endl;
        } else {
            cout << l << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}