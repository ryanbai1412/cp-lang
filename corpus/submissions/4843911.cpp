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

struct Node {
    int l, r;

    Node operator+(const Node& n) const {
        if (r <= n.l) {
            return {n.l, n.l};
        } else if (l >= n.r) {
            return {n.r, n.r};
        } else {
            return {max(l, n.l), min(r, n.r)};
        }
    }

    static Node def() {
        return {0, int(1e9)};
    }

    static Node adding(int x) {
        return {x, int(1e9)};
    }

    static Node removing(int x) {
        return {0, x};
    }
};

struct ST {
    int n;
    vector<Node> v;

    ST(int n) : n(n), v(4 * n, Node::def()) {}

    void update(int o, int l, int r, int ind, const Node& x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = v[lc] + v[rc];
    }

    void update(int ind, const Node& x) {
        update(1, 0, n - 1, ind, x);
    }

    int query() const {
        return (Node {0, 0} + v[1]).l;
    }
};

void buildWall(int n,
               int q,
               int qops[],
               int qls[],
               int qrs[],
               int qxs[],
               int qout[]) {
//    {
//        ST st(3);
//        st.update(0, Node::adding(91220));
//        st.update(2, Node::removing(39412));
//        dbg(st.query());
//        return;
//    }
    vector<pair<bool, int>> evts[n + 1];
    for (int i = 0; i < q; i++) {
        evts[qls[i]].emplace_back(false, i);
        evts[qrs[i] + 1].emplace_back(true, i);
    }
    ST st(q);
    for (int i = 0; i < n; i++) {
        for (auto& [b, j] : evts[i]) {
            if (b) {
                st.update(j, Node::def());
                dbg("DEF", j);
            } else if (qops[j] == 1) {
                st.update(j, Node::adding(qxs[j]));
                dbg("ADD", j, qxs[j]);
            } else {
                st.update(j, Node::removing(qxs[j]));
                dbg("REM", j, qxs[j]);
            }
        }
        qout[i] = st.query();
    }
}