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
    long pref, suff, sum, v;

    Node operator+(const Node& n) const {
        return {max(pref, sum + n.pref), max(n.suff, suff + n.sum), sum + n.sum,
                max({v, n.v, suff + n.pref})};
    }

    static Node from(int x) {
        int y = max(0, x);
        return {y, y, x, y};
    }
};

struct DS {
    int n;
    vector<Node> v;

    DS(int n) : n(n), v(4 * n, Node::from(0)) {}

    void update(int o, int l, int r, int ind, int x) {
        if (l == r) {
            v[o] = Node::from(x);
        } else {
            int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
            if (ind <= mid) {
                update(lc, l, mid, ind, x);
            } else {
                update(rc, mid + 1, r, ind, x);
            }
            v[o] = v[lc] + v[rc];
        }
    }

    void set(int ind, int x) {
        update(1, 0, n - 1, ind, x);
    }

    Node query(int o, int l, int r, int ql, int qr) {
        if (ql <= l && r <= qr) {
            return v[o];
        } else {
            int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
            if (ql <= mid) {
                if (mid < qr) {
                    return query(lc, l, mid, ql, qr) +
                           query(rc, mid + 1, r, ql, qr);
                }
                return query(lc, l, mid, ql, qr);
            } else {
                return query(rc, mid + 1, r, ql, qr);
            }
        }
    }

    Node query(int l, int r) {
        if (l > r) {
            return Node::from(0);
        }
        return query(1, 0, n - 1, l, r);
    }

    int bsearch(int o, int l, int r, int x, Node& suff) {
        Node nxt = v[o] + suff;
        if (nxt.v < x) {
            suff = nxt;
            return l;
        } else if (l == r) {
            return r + 1;
        } else {
            int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
            int ans = bsearch(rc, mid + 1, r, x, suff);
            if (ans > mid + 1) {
                return ans;
            }
            return bsearch(lc, l, mid, x, suff);
        }
    }

    int bsearch(int x) {
        Node suff = Node::from(0);
        return bsearch(1, 0, n - 1, x, suff);
    }
};

vector<int> distribute_candies(vector<int> arr,
                               vector<int> ql,
                               vector<int> qr,
                               vector<int> qv) {
    int n = sz(arr), q = sz(ql);
    vector<pair<int, int>> upds[n + 1];
    for (int i = 0; i < q; i++) {
        upds[ql[i]].emplace_back(i, qv[i]);
        upds[qr[i] + 1].emplace_back(i, 0);
    }
    DS pos(q), neg(q);
    vector<int> ans(n);
    for (int i = 0; i < n; i++) {
        for (auto& [ind, x] : upds[i]) {
            pos.set(ind, x);
            neg.set(ind, -x);
        }
        int posi = pos.bsearch(arr[i]), negi = neg.bsearch(arr[i]);
        dbg(posi, negi);
        if (posi == negi) {
            assert(!posi);
        }
        if (posi > negi) {
            ans[i] = arr[i] - neg.query(posi, q - 1).suff;
        } else {
            ans[i] = pos.query(negi, q - 1).suff;
        }
    }
    return ans;
}