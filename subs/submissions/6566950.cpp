// cf bits/extc++.h nonsense
#ifdef ONLINE_JUDGE
#define _EXT_CODECVT_SPECIALIZATIONS_H 1
#define _EXT_ENC_FILEBUF_H 1
#endif
#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

inline void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

template <typename T>
struct PSA {
    int n;
    vector<T> psum;

    template <typename U>
    PSA(const vector<U>& arr) : n(sz(arr)), psum(n + 1) {
        for (int i = 0; i < n; i++) {
            psum[i + 1] = psum[i] + arr[i];
        }
    }

    T query(int l, int r) const {
        return psum[r] - psum[l];
    }
};

template <typename T>
bool on(T mask, int bit) {
    return (mask >> bit) & 1;
}

template <typename T>
vector<T> iota(int n, T x) {
    vector<T> arr(n);
    iota(begin(arr), end(arr), x);
    return arr;
}

struct NV {
    long pref, suff, sum, v;

    NV operator+(const NV& o) const {
        return {max(pref, sum + o.pref), max(suff + o.sum, o.suff), sum + o.sum,
                max({v, o.v, suff + o.pref})};
    }

    NV rev() const {
        return {suff, pref, sum, v};
    }

    static NV c_from(long x) {
        return {x, x, x, x};
    }
    static NV c_def() {
        return {long(-1e18), long(-1e18), 0, long(-1e18)};
    }
    static NV c_range(long x, int n) {
        long cx = max(x, n * x);
        return {cx, cx, n * x, cx};
    }
};

constexpr int NO_SET = -1e9;

mt19937 cowng(1412);

struct Node {
    Node *lc, *rc;
    NV v;
    int pri, siz, l_set, x;
    bool rev;

    Node(long x)
        : lc {},
          rc {},
          v(NV::c_from(x)),
          pri(cowng()),
          siz(1),
          l_set(NO_SET),
          x(x),
          rev(false) {}
};

NV g_v(Node* o) {
    return o ? o->v : NV::c_def();
}

int g_siz(Node* o) {
    return o ? o->siz : 0;
}

void maintain(Node* o) {
    if (!o) {
        return;
    }

    o->siz = g_siz(o->lc) + 1 + g_siz(o->rc);

    if (o->l_set != NO_SET) {
        o->v = NV::c_range(o->l_set, o->siz);
        return;
    }

    o->v = g_v(o->lc) + NV::c_from(o->x) + g_v(o->rc);
    if (o->rev) {
        o->v = o->v.rev();
    }
}

void pushdown(Node* o) {
    if (!o) {
        return;
    }

    if (o->l_set != NO_SET) {
        o->x = o->l_set;
        for (auto& ch : {o->lc, o->rc}) {
            if (ch) {
                ch->l_set = o->l_set;
            }
        }
        o->rev = false;
        o->l_set = NO_SET;
        maintain(o->lc);
        maintain(o->rc);
        maintain(o);
        return;
    }

    if (o->rev) {
        swap(o->lc, o->rc);

        for (auto& ch : {o->lc, o->rc}) {
            if (ch) {
                ch->rev ^= true;
            }
        }

        o->rev = false;
    }
    maintain(o->lc);
    maintain(o->rc);
    maintain(o);
}

void split(Node* o, int ind, Node*& nl, Node*& nr) {
    dbg(ind, g_siz(o));
    if (!o) {
        assert(!ind);
        nl = nr = nullptr;
        return;
    }

    pushdown(o);
    if (ind <= g_siz(o->lc)) {
        split(o->lc, ind, nl, o->lc);
        nr = o;
        maintain(o->rc);
    } else {
        split(o->rc, ind - g_siz(o->lc) - 1, o->rc, nr);
        nl = o;
        maintain(o->lc);
    }
    maintain(o);
}

pair<Node*, Node*> split(Node* o, int ind) {
    Node *nl, *nr;
    split(o, ind, nl, nr);
    return {nl, nr};
}

array<Node*, 3> split(Node* o, int ind, int len) {
    Node *nl, *nm, *nr;
    split(o, ind, nl, nm);
    split(nm, len, nm, nr);
    return {nl, nm, nr};
}

Node* merge(Node* u, Node* v) {
    if (!u) {
        return v;
    } else if (!v) {
        return u;
    }

    if (u->pri < v->pri) {
        pushdown(u);
        u->rc = merge(u->rc, v);
        maintain(u->lc);
        maintain(u);
        return u;
    } else {
        pushdown(v);
        v->lc = merge(u, v->lc);
        maintain(v->rc);
        maintain(v);
        return v;
    }
}

Node* merge(Node* a, Node* b, Node* c) {
    return merge(merge(a, b), c);
}

void tfree(Node* o) {
    if (!o) {
        return;
    }

    tfree(o->lc);
    tfree(o->rc);
    delete o;
}

void solve() {
    int n, q;
    cin >> n >> q;

    auto build_tree = [&](int n) -> Node* {
        Node* o {};
        for (int i = 0; i < n; i++) {
            int x;
            cin >> x;
            o = merge(o, new Node(x));
        }
        return o;
    };

    Node* root = build_tree(n);
    while (q--) {
        string s;
        cin >> s;

        if (s == "MAX-SUM") {
            dbg(root, root->lc, root->rc, root->x);
            cout << g_v(root).v << endl;
            continue;
        }

        int ind, len;
        cin >> ind >> len;

        if (s == "INSERT") {
            auto [nl, nr] = split(root, ind);
            root = merge(nl, build_tree(len), nr);
            continue;
        }

        ind--;
        auto [nl, nm, nr] = split(root, ind, len);
        if (s == "DELETE") {
            tfree(nm);
            root = merge(nl, nr);
            continue;
        }

        if (s == "MAKE-SAME") {
            int x;
            cin >> x;
            nm->l_set = x;
            maintain(nm);
        } else if (s == "REVERSE") {
            nm->rev ^= true;
            maintain(nm);
        } else if (s == "GET-SUM") {
            cout << g_v(nm).sum << endl;
        }

        root = merge(nl, nm, nr);
    }

    tfree(root);
}

int main() {
    init_io();
    solve();
}