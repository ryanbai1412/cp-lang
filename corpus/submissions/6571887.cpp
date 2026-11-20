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

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

struct NV {
    long pref, suff, v, sum;

    NV operator+(const NV& o) const {
        return {max(pref, sum + o.pref), max(suff + o.sum, o.suff),
                max({v, o.v, suff + o.pref}), sum + o.sum};
    }

    NV rev() const {
        return {suff, pref, v, sum};
    }

    static NV c_from(long x) {
        return {x, x, x, x};
    }

    static NV c_def() {
        return {long(-1e18), long(-1e18), long(-1e18), 0};
    }

    static NV c_range(long x, int len) {
        long cx = max(x, x * len);
        return {cx, cx, cx, x * len};
    }
};

constexpr int NO_SET = INT_MIN;

mt19937_64 cowng(1412);

struct Node {
    Node *lc, *rc;
    NV v;
    int pri, siz, x, l_set;
    bool rev;

    Node(int x)
        : lc {},
          rc {},
          v(NV::c_from(x)),
          pri(cowng()),
          siz(1),
          x(x),
          l_set(NO_SET),
          rev(false) {}
};

int g_siz(Node* o) {
    return o ? o->siz : 0;
}

NV g_v(Node* o) {
    return o ? o->v : NV::c_def();
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
        for (auto& ch : {o->lc, o->rc}) {
            if (ch) {
                ch->l_set = o->l_set;
            }
        }
        o->x = o->l_set;
        o->l_set = NO_SET;
        o->rev = false;
        return;
    }

    if (o->rev) {
        for (auto& ch : {o->lc, o->rc}) {
            if (ch) {
                ch->rev ^= o->rev;
            }
        }
        swap(o->lc, o->rc);
        o->rev = false;
    }
}

void split(Node* o, int ind, Node*& nl, Node*& nr) {
    if (!o) {
        assert(!ind);
        nl = nr = nullptr;
        return;
    }

    pushdown(o);
    if (ind <= g_siz(o->lc)) {
        split(o->lc, ind, nl, o->lc);
        maintain(o->rc);
        nr = o;
    } else {
        split(o->rc, ind - g_siz(o->lc) - 1, o->rc, nr);
        maintain(o->lc);
        nl = o;
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

Node* merge(Node* nl, Node* nr) {
    if (!nl) {
        maintain(nr);
        return nr;
    } else if (!nr) {
        maintain(nl);
        return nl;
    }

    if (nl->pri < nr->pri) {
        pushdown(nl);
        nl->rc = merge(nl->rc, nr);
        maintain(nl->lc);
        maintain(nl);
        return nl;
    } else {
        pushdown(nr);
        nr->lc = merge(nl, nr->lc);
        maintain(nr->rc);
        maintain(nr);
        return nr;
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

Node* build_arr(const vector<int>& arr, int l, int r) {
    if (l == r) {
        return {};
    }

    int mid = (l + r) / 2;
    Node* o = new Node(arr[mid]);
    o->lc = build_arr(arr, l, mid);
    o->rc = build_arr(arr, mid + 1, r);
    maintain(o);
    return o;
}

void solve() {
    int n, q;
    cin >> n >> q;

    vector<int> buf;
    auto build1 = [&](int len) -> Node* {
        buf.resize(len);
        for (auto& a : buf) {
            cin >> a;
        }
        return build_arr(buf, 0, len);
    };

    Node* root = build1(n);

    while (q--) {
        string ty;
        cin >> ty;

        if (ty == "MAX-SUM") {
            cout << g_v(root).v << endl;
            continue;
        }

        int ind, len;
        cin >> ind >> len;

        if (ty == "INSERT") {
            auto [nl, nr] = split(root, ind);
            root = merge(nl, build1(len), nr);
            continue;
        }

        ind--;

        auto [nl, nm, nr] = split(root, ind, len);
        if (ty == "DELETE") {
            tfree(nm);
            root = merge(nl, nr);
            continue;
        } else if (ty == "MAKE-SAME") {
            int x;
            cin >> x;
            nm->l_set = x;
            maintain(nm);
        } else if (ty == "REVERSE") {
            nm->rev ^= true;
            maintain(nm);
        } else {
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