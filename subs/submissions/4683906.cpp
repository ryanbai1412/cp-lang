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

struct Node {
    int l, r;

    Node operator+(const Node& n) const {
        if (n.r < l) {
            return {n.r, n.r};
        } else if (r < n.l) {
            return {n.l, n.l};
        }
        return {max(l, n.l), min(r, n.r)};
    }
} ndef {0, int(1e9)};

struct ST {
    int n;
    vector<Node> v;

    ST(int n) : n(n), v(4 * n, ndef) {}

    void set(int o, int l, int r, int ind, const Node& x) {
        if (l == r) {
            v[o] = x;
        } else {
            int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
            if (ind <= mid) {
                set(lc, l, mid, ind, x);
            } else {
                set(rc, mid + 1, r, ind, x);
            }
            v[o] = v[lc] + v[rc];
        }
    }

    void set(int ind, const Node& x) {
        set(1, 0, n - 1, ind, x);
    }

    Node query_all() {
        return v[1];
    }
};

void buildWall(int n,
               int q,
               int op[],
               int ql[],
               int qr[],
               int qx[],
               int out[]) {
    vector<pair<bool, int>> ops[n + 1];
    for (int i = 0; i < q; i++) {
        ops[ql[i]].emplace_back(true, i);
        ops[qr[i] + 1].emplace_back(false, i);
    }
    ST st(q);
    for (int i = 0; i < n; i++) {
        for (auto& [add, j] : ops[i]) {
            if (add) {
                if (op[j] == 1) {
                    st.set(j, {qx[j], int(1e9)});
                } else {
                    st.set(j, {0, qx[j]});
                }
            } else {
                st.set(j, ndef);
            }
        }
        out[i] = st.query_all().l;
    }
}