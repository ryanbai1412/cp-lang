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

// mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
mt19937 cowng(1412);

template <typename T, size_t N>
struct A {
    T heap[N];
    int free_inds[N - 1], free_inds_ptr = N - 1;

    A() {
        iota(rbegin(free_inds), rend(free_inds), 1);
    }

    int alloc() {
        if (!sz(free_inds)) {
            throw bad_alloc();
        }
        int ans = free_inds[--free_inds_ptr];
        return ans;
    }

    int alloc(const T& t) {
        int ptr = alloc();
        heap[ptr] = t;
        return ptr;
    }

    void free(int ind) {
        free_inds[free_inds_ptr++] = ind;
    }

    T& operator[](int i) {
        return heap[i];
    }
};

struct DS {
    struct NV {
        static constexpr int NO_NV = INT_MIN;

        int sum, pref, suff, v;

        friend NV operator+(const NV& l, const NV& r) {
            if (l.sum == NO_NV) {
                return r;
            } else if (r.sum == NO_NV) {
                return l;
            }
            return {l.sum + r.sum, max(l.pref, l.sum + r.pref),
                    max(r.suff, l.suff + r.sum),
                    max({l.v, r.v, l.suff + r.pref})};
        }

        static NV from(int x) {
            return {x, x, x, x};
        }

        static NV from(int len, int x) {
            assert(len);
            return {x * len, max(x, x * len), max(x, x * len), max(x, x * len)};
        }

        static NV c_default() {
            return {NO_NV, 0, 0, 0};
        }
    };

    struct Lazy {
        static constexpr int NO_SET = INT_MIN;

        int set;
        bool rev;

        Lazy& operator+=(const Lazy& l) {
            if (l.set != NO_SET) {
                return *this = l;
            }
            rev ^= l.rev;
            return *this;
        }

        friend Lazy operator+(Lazy l, const Lazy& r) {
            return l += r;
        }

        NV apply(const NV& n, int siz) const {
            if (set != NO_SET) {
                return NV::from(siz, set);
            }
            if (rev) {
                return {n.sum, n.suff, n.pref, n.v};
            }
            return n;
        }

        static Lazy c_def() {
            return {NO_SET, false};
        }

        static Lazy c_set(int x) {
            return {x, false};
        }

        static Lazy c_rev() {
            return {NO_SET, true};
        }
    };

    struct Node {
        NV v;
        Lazy lazy;
        int ox, siz, lc, rc, pri;

        static Node from(int x) {
            return {NV::from(x), Lazy::c_def(), x, 1, 0, 0, int(cowng())};
        }
    };

    A<Node, 1 << 19> heap;

    int root = 0;

    DS() {
        heap[0].v = NV::c_default();
    }

    void maintain(int o) {
        if (!o) {
            return;
        }
        auto& [v, lazy, ox, siz, lc, rc, _pri] = heap[o];
        siz = heap[lc].siz + heap[rc].siz + 1;
        v = heap[rc].v + NV::from(ox) + heap[lc].v;
        v = lazy.apply(v, siz);
    }

    void pushdown(int o) {
        assert(o);
        auto& lazy = heap[o].lazy;
        auto &lc = heap[o].lc, &rc = heap[o].rc;
        if (lazy.rev) {
            swap(lc, rc);
        }
        auto go = [&](int c) -> void {
            if (!c) {
                return;
            }
            heap[c].lazy = heap[c].lazy + lazy;
        };
        go(lc);
        go(rc);
        if (lazy.set != Lazy::NO_SET) {
            heap[o].ox = lazy.set;
        }
        lazy = Lazy::c_def();
    }

    // kv <= sz(o)
    // [0, kv) goes to l, [kv, inf) goes to r
    void split(int o, int kv, int& l, int& r) {
        if (!o) {
            assert(!kv);
            l = r = 0;
            return;
        }
        pushdown(o);
        auto &lc = heap[o].lc, &rc = heap[o].rc;
        int lsiz = heap[lc].siz;
        if (kv <= lsiz) {
            r = o;
            split(lc, kv, l, lc);
            maintain(rc);
        } else {
            l = o;
            split(rc, kv - lsiz - 1, rc, r);
            maintain(lc);
        }
        maintain(o);
    }

    pair<int, int> split(int o, int kv) {
        int l, r;
        split(o, kv, l, r);
        return {l, r};
    }

    tuple<int, int, int> split(int o, int ind, int len) {
        auto [l, r1] = split(o, ind);
        auto [mid, r] = split(r1, len);
        return {l, mid, r};
    }

    int merge(int l, int r) {
        if (!l || !r) {
            maintain(l);
            maintain(r);
            return l + r;
        }
        if (heap[l].pri < heap[r].pri) {
            pushdown(l);
            maintain(heap[l].lc);
            heap[l].rc = merge(heap[l].rc, r);
            maintain(l);
            return l;
        } else {
            pushdown(r);
            maintain(heap[r].rc);
            heap[r].lc = merge(l, heap[r].lc);
            maintain(r);
            return r;
        }
    }

    int merge(int a, int b, int c) {
        return merge(merge(a, b), c);
    }

    void free(int o) {
        if (!o) {
            return;
        }
        free(heap[o].lc);
        free(heap[o].rc);
        heap.free(o);
    }

    void update_insert(int ind, const vector<int>& arr) {
        auto [l, r] = split(root, ind);
        for (auto& a : arr) {
            l = merge(l, heap.alloc(Node::from(a)));
        }
        root = merge(l, r);
    }

    void update_delete(int ind, int len) {
        auto [l, mid, r] = split(root, ind, len);
        free(mid);
        root = merge(l, r);
    }

    void update_set(int ind, int len, int x) {
        auto [l, mid, r] = split(root, ind, len);
        heap[mid].lazy += Lazy::c_set(x);
        maintain(mid);
        dbg("A");
        root = merge(l, mid, r);
    }

    void update_reverse(int ind, int len) {
        auto [l, mid, r] = split(root, ind, len);
        heap[mid].lazy += Lazy::c_rev();
        maintain(mid);
        root = merge(l, mid, r);
    }

    int query_sum(int ind, int len) {
        auto [l, mid, r] = split(root, ind, len);
        int ans = heap[mid].v.sum;
        root = merge(l, mid, r);
        return ans;
    }

    int query_max_subarray_sum() {
        return heap[root].v.v;
    }
};

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    DS ds;
    ds.update_insert(0, arr);
    dbg("START");
    while (q--) {
        string s;
        cin >> s;
        if (s == "MAX-SUM") {
            cout << ds.query_max_subarray_sum() << endl;
            continue;
        }
        int ind, len;
        cin >> ind >> len;
        if (s == "INSERT") {
            vector<int> carr(len);
            for (auto& a : carr) {
                cin >> a;
            }
            ds.update_insert(ind, carr);
            continue;
        }
        ind--;
        if (s == "DELETE") {
            ds.update_delete(ind, len);
        } else if (s == "REVERSE") {
            ds.update_reverse(ind, len);
        } else if (s == "GET-SUM") {
            if (!len) {
                cout << 0 << endl;
            } else {
                cout << ds.query_sum(ind, len) << endl;
            }
        } else if (s == "MAKE-SAME") {
            int x;
            cin >> x;
            ds.update_set(ind, len, x);
        } else {
            assert(false);
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}