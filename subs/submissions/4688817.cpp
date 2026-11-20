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

const int maxn1 = 10 << 16, maxn2 = 300 << 16;

struct N1 {
    int lc, rc, v;
} h1[maxn1];

bitset<maxn2 * 2> h21;
uint16_t h216[maxn2];
uint32_t h232[maxn2];
int h2v[maxn2];

vector<long> vals {0};
map<long, int> vis;

void storev(int ind, long x) {
    auto [it, inserted] = vis.emplace(x, sz(vals));
    if (inserted) {
        vals.push_back(x);
    }
    h2v[ind] = it->second;
    dbg("STORE", ind, x);
}

long loadv(int ind) {
    dbg("LOAD", ind, vals[h2v[ind]]);
    return vals[h2v[ind]];
}

template <int offset>
void store(int ind, int x) {
    h21[ind * 2 + offset] = x >> 24;
    ((uint8_t*)(h216 + ind))[offset] = x >> 16;
    ((uint16_t*)(h232 + ind))[offset] = x & 65535;
}

template <int offset>
int load(int ind) {
    return h21[ind * 2 + offset] << 24 |
           ((uint8_t*)(h216 + ind))[offset] << 16 |
           ((uint16_t*)(h232 + ind))[offset];
}

int hind1 = 1, hind2 = 1;

int alloc1() {
    return hind1++;
}

int alloc2() {
    assert(hind2 < maxn2);
    return hind2++;
}

template <int offset>
int load_or_alloc(int ind) {
    int res = load<offset>(ind);
    if (!res) {
        store<offset>(ind, res = alloc2());
    }
    return res;
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
        return loadv(o);
    }
    int mid = (l + r) / 2;
    long ans = 0;
    if (ql <= mid) {
        ans = query2(load<0>(o), l, mid, ql, qr);
    }
    if (mid < qr) {
        ans = gcd(ans, query2(load<1>(o), mid + 1, r, ql, qr));
    }
    return ans;
}

long query1(int o, int l, int r, int ql, int qr, int ql2, int qr2) {
    if (!o) {
        return 0;
    } else if (ql <= l && r <= qr) {
        return query2(h1[o].v, 0, m - 1, ql2, qr2);
    }
    auto& [lc, rc, _] = h1[o];
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

void update2(int o, int l, int r, int y, long v) {
    assert(o);
    if (l == r) {
        storev(o, v);
        return;
    }
    int mid = (l + r) / 2;
    if (y <= mid) {
        update2(load_or_alloc<0>(o), l, mid, y, v);
    } else {
        update2(load_or_alloc<1>(o), mid + 1, r, y, v);
    }
    storev(o, gcd(loadv(load<0>(o)), loadv(load<1>(o))));
}

void update1(int& o, int l, int r, int x, int y, long v) {
    if (!o) {
        o = alloc1();
    }
    auto& [lc, rc, cv] = h1[o];
    if (!cv) {
        cv = alloc2();
    }
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
            gcd(query2(h1[lc].v, 0, m - 1, y, y),
                query2(h1[rc].v, 0, m - 1, y, y)));
}

extern "C" void update(int x, int y, long v) {
    update1(root, 0, n - 1, x, y, v);
}

extern "C" long calculate(int x1, int y1, int x2, int y2) {
    return query1(root, 0, n - 1, x1, x2, y1, y2);
}