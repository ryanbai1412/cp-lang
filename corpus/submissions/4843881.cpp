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

constexpr int nlazy = INT_MIN, maxn = 1e9 + 5;

struct Node {
    int pref, sm;

    Node operator+(const Node& n) const {
        return {max(pref, sm + n.pref), sm + n.sm};
    }

    Node apply(int lazy, int l, int r) const {
        if (lazy == nlazy) {
            return *this;
        } else {
            int x = lazy * (r - l + 1);
            return {max(0, x), x};
        }
    }

    static Node from(int x) {
        return {max(0, x), x};
    }
};

struct N {
    Node v;
    int lazy, lc, rc;
} heap[(200 << 20) / sizeof(Node)];
int root, hind = 1;

int alloc() {
    heap[hind].lazy = nlazy;
    return hind++;
}

void alloc(int& o) {
    if (!o) {
        o = alloc();
    }
}

void maintain(int o, int l, int r) {
    if (o) {
        auto& [v, lazy, lc, rc] = heap[o];
        v = (heap[lc].v + heap[rc].v).apply(lazy, l, r);
    }
}

void update(int& o, int l, int r, int ql, int qr, int x) {
    alloc(o);
    auto& [_, lazy, lc, rc] = heap[o];
    if (ql <= l && r <= qr) {
        lazy = x;
    } else {
        int mid = (l + r) / 2;
        alloc(lc);
        alloc(rc);
        if (lazy != nlazy) {
            heap[lc].lazy = heap[rc].lazy = lazy;
            lazy = nlazy;
        }
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        } else {
            maintain(lc, l, mid);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        } else {
            maintain(rc, mid + 1, r);
        }
    }
    maintain(o, l, r);
}

void update(int l, int r, int x) {
    update(root, 0, maxn, l, r, x);
}

Node query(int o, int l, int r, int ql, int qr, int clazy) {
    if (!o) {
        return Node::from(0).apply(clazy, max(ql, l), min(qr, r));
    }
    auto& [v, lazy, lc, rc] = heap[o];
    if (ql <= l && r <= qr) {
        return v.apply(clazy, l, r);
    }
    if (clazy == nlazy) {
        clazy = lazy;
    }
    int mid = (l + r) / 2;
    if (ql <= mid) {
        if (mid < qr) {
            return query(lc, l, mid, ql, qr, clazy) +
                   query(rc, mid + 1, r, ql, qr, clazy);
        }
        return query(lc, l, mid, ql, qr, clazy);
    }
    return query(rc, mid + 1, r, ql, qr, clazy);
}

Node query(int l, int r) {
    return query(root, 0, maxn, l, r, nlazy);
}

void solve() {
//    {
//        update(0, 3, 2);
//        dbg(query(0, 2).pref);
//        return;
//    }
    int n;
    cin >> n;
    while (true) {
        char t;
        cin >> t;
        if (t == 'I') {
            int l, r, x;
            cin >> l >> r >> x;
            l--;
            r--;
            update(l, r, x);
        } else if (t == 'Q') {
            int x;
            cin >> x;
            int l = -1, r = n - 1;
            while (l < r) {
                int mid = (l + r + 1) / 2;
                if (query(0, mid).pref <= x) {
                    l = mid;
                } else {
                    r = mid - 1;
                }
            }
            cout << l + 1 << endl;
        } else {
            break;
        }
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}