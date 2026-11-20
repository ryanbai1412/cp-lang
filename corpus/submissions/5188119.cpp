#pragma GCC optimize("Ofast")

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

template <typename T, size_t N>
struct A {
    T v[N];
    int f[N], f_p = N;

    A() {
        iota(begin(f), end(f), 0);
    }

    T* alloc() {
        return &v[f[--f_p]];
    }

    T* alloc(const T& t) {
        T* ptr = alloc();
        *ptr = t;
        return ptr;
    }

    void free(T* t) {
        f[f_p++] = int(t - v);
    }
};

struct NV {
    int sum, pref, suff, v;

    NV& operator+=(const NV& n) {
        v = max({v, n.v, suff + n.pref});
        pref = max(pref, sum + n.pref);
        suff = max(n.suff, n.sum + suff);
        sum += n.sum;
        return *this;
    }

    friend NV operator+(NV a, const NV& b) {
        return a += b;
    }

    static NV c_from(int x) {
        return {x, x, x, x};
    }

    static NV c_def() {
        return {0, int(-1e9), int(-1e9), int(-1e9)};
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

    void apply_inplace(NV& n, int siz) const {
        if (set != NO_SET) {
            n.sum = set * siz;
            n.pref = n.suff = n.v = max(set, set * siz);
        } else if (rev) {
            swap(n.pref, n.suff);
        }
    }

    void apply_inplace(int& x) const {
        if (set != NO_SET) {
            x = set;
        }
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
    int x, siz, pri;
    Node *lc, *rc;
};

int get_siz(Node* o) {
    return o ? o->siz : 0;
}

NV get_v(Node* o) {
    return o ? o->v : NV::c_def();
}

void pushdown(Node* o) {
    auto go = [&](Node* c) -> void {
        if (!c) {
            return;
        }
        o->lazy.apply_inplace(c->v, c->siz);
        c->lazy += o->lazy;
    };
    go(o->lc);
    go(o->rc);
    o->lazy.apply_inplace(o->x);
    if (o->lazy.rev) {
        swap(o->lc, o->rc);
    }
    o->lazy = Lazy::c_def();
}

void maintain(Node* o) {
    if (!o) {
        return;
    }
    o->siz = get_siz(o->lc) + 1 + get_siz(o->rc);
    o->v = get_v(o->lc) + NV::c_from(o->x) + get_v(o->rc);
    o->lazy.apply_inplace(o->v, o->siz);
}

void split(Node* o, int ind, Node*& l, Node*& r) {
    if (!o) {
        l = r = nullptr;
        return;
    }
    pushdown(o);
    if (ind <= get_siz(o->lc)) {
        split(o->lc, ind, l, o->lc);
        r = o;
    } else {
        split(o->rc, ind - get_siz(o->lc) - 1, o->rc, r);
        l = o;
    }
    maintain(o);
}

Node* merge(Node* l, Node* r) {
    if (!l) {
        return r;
    } else if (!r) {
        return l;
    }
    if (l->pri < r->pri) {
        pushdown(l);
        l->rc = merge(l->rc, r);
        maintain(l);
        return l;
    } else {
        pushdown(r);
        r->lc = merge(l, r->lc);
        maintain(r);
        return r;
    }
}

pair<Node*, Node*> split(Node* o, int ind) {
    Node *l, *r;
    split(o, ind, l, r);
    return {l, r};
}

tuple<Node*, Node*, Node*> split(Node* o, int ind, int len) {
    Node *l, *m, *r;
    split(o, ind, l, r);
    split(r, len, m, r);
    return {l, m, r};
}

Node* merge(Node* l, Node* m, Node* r) {
    return merge(merge(l, m), r);
}

A<Node, 1 << 19> heap;

void free(Node* o) {
    if (!o) {
        return;
    }
    free(o->lc);
    free(o->rc);
    heap.free(o);
}

void solve() {
    mt19937 cowng(1412);
    auto alloc_node = [&](int x) -> Node* {
        return heap.alloc({NV::c_from(x), Lazy::c_def(), x, 1, int(cowng()),
                           nullptr, nullptr});
    };
    int n, q;
    cin >> n >> q;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    Node* root {};
    for (auto& a : arr) {
        root = merge(root, alloc_node(a));
    }
    while (q--) {
        string s;
        cin >> s;
        if (s == "MAX-SUM") {
            cout << root->v.v << endl;
            continue;
        }
        int ind, len;
        cin >> ind >> len;
        if (s == "INSERT") {
            auto [nl, nr] = split(root, ind);
            for (int i = 0; i < len; i++) {
                int x;
                cin >> x;
                nl = merge(nl, alloc_node(x));
            }
            root = merge(nl, nr);
            continue;
        }
        ind--;
        auto [nl, nm, nr] = split(root, ind, len);
        if (s == "DELETE") {
            free(nm);
            root = merge(nl, nr);
            continue;
        }
        if (s == "MAKE-SAME") {
            int x;
            cin >> x;
            nm->lazy += Lazy::c_set(x);
            maintain(nm);
        } else if (s == "REVERSE") {
            nm->lazy += Lazy::c_rev();
            maintain(nm);
        } else if (s == "GET-SUM") {
            cout << get_v(nm).sum << endl;
        } else {
            assert(false);
        }
        root = merge(nl, nm, nr);
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}