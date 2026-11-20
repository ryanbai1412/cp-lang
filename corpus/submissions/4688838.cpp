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
#define long long long
#define sz(x) int((x).size())

vector<long> vals {0};
map<long, int> vis;

struct N {
    int lc, rc, v;

    void setv(long x) {
        auto [it, inserted] = vis.emplace(x, sz(vals));
        if (inserted) {
            vals.push_back(x);
        }
        v = it->second;
    }

    int getv() {
        return vals[v];
    }
} heap[(220 << 20) / sizeof(N)];

int alloc() {
    static int hind = 1;
    return hind++;
}

int n, m, root;

extern "C" void init(int _n, int _m) {
    n = _n;
    m = _m;
}

long query2(int o, int l, int r, int ql, int qr) {
    if (!o) {
        return 0;
    } else if (ql <= l && r <= qr) {
        return heap[o].getv();
    }
    auto& [lc, rc, _] = heap[o];
    int mid = (l + r) / 2;
    long ans = 0;
    if (ql <= mid) {
        ans = query2(lc, l, mid, ql, qr);
    }
    if (mid < qr) {
        ans = gcd(ans, query2(rc, mid + 1, r, ql, qr));
    }
    return ans;
}

long query1(int o, int l, int r, int ql, int qr, int ql2, int qr2) {
    if (!o) {
        return 0;
    } else if (ql <= l && r <= qr) {
        return query2(heap[o].v, 0, m - 1, ql2, qr2);
    }
    auto& [lc, rc, _] = heap[o];
    int mid = (l + r) / 2;
    long ans = 0;
    if (ql <= mid) {
        ans = query1(lc, l, mid, ql, qr, ql2, qr2);
    }
    if (mid < qr) {
        ans = gcd(ans, query1(rc, mid + 1, r, ql, qr, ql2, qr2));
    }
    return ans;
}

void update2(int& o, int l, int r, int y, long v) {
    if (!o) {
        o = alloc();
    }
    if (l == r) {
        heap[o].setv(v);
        return;
    }
    int mid = (l + r) / 2;
    auto& [lc, rc, _] = heap[o];
    if (y <= mid) {
        update2(lc, l, mid, y, v);
    } else {
        update2(rc, mid + 1, r, y, v);
    }
    heap[o].setv(gcd(heap[lc].getv(), heap[rc].getv()));
}

void update1(int& o, int l, int r, int x, int y, long v) {
    if (!o) {
        o = alloc();
    }
    auto& [lc, rc, cv] = heap[o];
    if (l == r) {
        update2(cv, 0, m - 1, y, v);
        return;
    }
    int mid = (l + r) / 2;
    if (x <= mid) {
        update1(lc, l, mid, x, y, v);
    } else {
        update1(rc, mid + 1, r, x, y, v);
    }
    update2(cv, 0, m - 1, y,
            gcd(query2(heap[lc].v, 0, m - 1, y, y),
                query2(heap[rc].v, 0, m - 1, y, y)));
}

extern "C" void update(int x, int y, long v) {
    update1(root, 0, n - 1, x, y, v);
}

extern "C" long calculate(int x1, int y1, int x2, int y2) {
    return query1(root, 0, n - 1, x1, x2, y1, y2);
}