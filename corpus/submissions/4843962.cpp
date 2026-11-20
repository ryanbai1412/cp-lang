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

struct N {
    int v, lc, rc;
};

template <typename T>
struct Alloc {
    vector<T> v {{}};

    int operator()() {
        v.push_back({});
        return sz(v) - 1;
    }

    void operator()(int& o) {
        if (!o) {
            o = (*this)();
        }
    }

    T& operator[](int i) {
        return v[i];
    }

    void reserve(int mn = 1000) {
        if (v.size() + mn > v.capacity()) {
            v.reserve(max(v.size() + mn, v.capacity() * 2));
        }
    }
};

Alloc<N> heap;

constexpr long maxn = 1e18 + 5;

void update1(int& o, long x, int val) {
    heap(o);
    auto& [v, lc, rc] = heap[o];
    if (x == 1) {
        v += val;
        return;
    }
    if (x % 2 == 0) {
        update1(lc, x >> 1, val);
    } else {
        update1(rc, x >> 1, val);
    }
}

void update2(int& o, long x, long y, int val) {
    heap(o);
    auto& [v, lc, rc] = heap[o];
    if (x == 1) {
        update1(v, y, val);
        return;
    }
    if (x % 2 == 0) {
        update2(lc, x >> 1, y, val);
    } else {
        update2(rc, x >> 1, y, val);
    }
}

int query1(int o, long x) {
    if (!o) {
        return 0;
    }
    auto& [v, lc, rc] = heap[o];
    if (x == 1) {
        return v;
    }
    if (x % 2 == 0) {
        return query1(lc, x >> 1) + v;
    } else {
        return query1(rc, x >> 1) + v;
    }
}

int query2(int o, long x, long y) {
    if (!o) {
        return 0;
    }
    auto& [v, lc, rc] = heap[o];
    int lazy = query1(v, y);
    if (x == 1) {
        return lazy;
    }
    if (x % 2 == 0) {
        dbg("L");
        return query2(lc, x >> 1, y) + lazy;
    } else {
        dbg("R");
        return query2(rc, x >> 1, y) + lazy;
    }
}

void solve() {
    dbg(sizeof(N));
    int n, q, root = 0;
    cin >> n >> q;
    heap.reserve(n * 70);
    for (int i = 0; i < n; i++) {
        long x, y;
        int v;
        cin >> x >> y >> v;
        update2(root, x, y, v);
    }
    while (q--) {
        long x, y;
        cin >> x >> y;
        cout << query2(root, x, y) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}