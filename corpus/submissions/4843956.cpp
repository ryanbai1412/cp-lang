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

struct N1 {
    int v, lazy, lc, rc;
};

struct N2 {
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
    
    void reserve() {
        if (v.size() + 1000 > v.capacity()) {
            v.reserve(max(v.size() + 1000, v.capacity() * 2));
        }
    }
};

Alloc<N1> h1;
Alloc<N2> h2;

constexpr long maxn = 1e18 + 5;

void update1(int& o, long x, int val) {
    h1(o);
    auto& [v, lazy, lc, rc] = h1[o];
    if (x == 1) {
        v += val;
        lazy += val;
        return;
    }
    if (x % 2 == 0) {
        update1(lc, x >> 1, val);
    } else {
        update1(rc, x >> 1, val);
    }
    v = h1[lc].v + h1[rc].v + lazy;
}

void update2(int& o, long x, long y, int val) {
    h2(o);
    auto& [v, lc, rc] = h2[o];
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
    auto& [v, lazy, lc, rc] = h1[o];
    if (x == 1) {
        return lazy;
    }
    if (x % 2 == 0) {
        return query1(lc, x >> 1) + lazy;
    } else {
        return query1(rc, x >> 1) + lazy;
    }
}

int query2(int o, long x, long y) {
    if (!o) {
        return 0;
    }
    auto& [v, lc, rc] = h2[o];
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
    int n, q, root = 0;
    cin >> n >> q;
    for (int i = 0; i < n; i++) {
        long x, y;
        int v;
        cin >> x >> y >> v;
        h1.reserve();
        h2.reserve();
        update2(root, x, y, v);
    }
    while (q--) {
        int x, y;
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