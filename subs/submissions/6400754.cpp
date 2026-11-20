#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

struct R {
    mt19937_64 cowng;

    // R() : cowng(chrono::steady_clock::now().time_since_epoch().count()) {}
    R() : cowng(1412) {}
} R;

struct NV {
    long pref, suff, sum, v;

    NV operator+(const NV& x) const {
        return {max(pref, sum + x.pref), max(x.suff, x.sum + suff), sum + x.sum,
                max({v, x.v, suff + x.pref})};
    }

    static NV c_def() {
        constexpr long ninf = -1e18;
        return {ninf, ninf, 0, ninf};
    }

    static NV c_from(long x) {
        return {x, x, x, x};
    }

    static NV c_set(long x, int len) {
        long v = max(x, x * len);
        return {v, v, x * len, v};
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

    friend Lazy operator+(Lazy a, const Lazy& b) {
        return a += b;
    }

    NV apply(NV o, int siz) {
        if (set != NO_SET) {
            return NV::c_set(set, siz);
        }
        if (rev) {
            swap(o.pref, o.suff);
        }
        return o;
    }

    static Lazy c_def() {
        return {NO_SET, false};
    }

    static Lazy c_set(int x) {
        assert(x != NO_SET);
        return {x, false};
    }

    static Lazy c_rev() {
        return {NO_SET, true};
    }
};

struct Node {
    int pri, siz, x;
    NV v;
    Lazy lazy;
    Node *lc, *rc;

    Node(int x)
        : pri(R.cowng()),
          siz(1),
          x(x),
          v(NV::c_from(x)),
          lazy(Lazy::c_def()),
          lc {},
          rc {} {}
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
    o->v = g_v(o->lc) + NV::c_from(o->x) + g_v(o->rc);

    o->v = o->lazy.apply(o->v, o->siz);
}

void pushdown(Node* o) {
    if (!o) {
        return;
    }

    for (auto& c : {o->lc, o->rc}) {
        if (!c) {
            continue;
        }

        c->lazy += o->lazy;
    }

    if (o->lazy.set != Lazy::NO_SET) {
        o->x = o->lazy.set;
    }
    if (o->lazy.rev) {
        swap(o->lc, o->rc);
    }
    o->lazy = Lazy::c_def();
}

void split(Node* o, int kv, Node*& l, Node*& r) {
    if (!o) {
        l = r = nullptr;
        return;
    }

    pushdown(o);
    if (kv <= g_siz(o->lc)) {
        split(o->lc, kv, l, o->lc);
        r = o;
        maintain(o->rc);
    } else {
        split(o->rc, kv - g_siz(o->lc) - 1, o->rc, r);
        l = o;
        maintain(o->lc);
    }
    maintain(o);
}

pair<Node*, Node*> split(Node* o, int kv) {
    Node *l {}, *r {};
    split(o, kv, l, r);
    return {l, r};
}

array<Node*, 3> split(Node* o, int kv, int len) {
    auto [l, mid] = split(o, kv);
    auto [m, r] = split(mid, len);
    return {l, m, r};
}

Node* merge(Node* l, Node* r) {
    if (!l) {
        maintain(r);
        return r;
    } else if (!r) {
        maintain(l);
        return l;
    }

    if (l->pri < r->pri) {
        pushdown(l);
        l->rc = merge(l->rc, r);
        maintain(l->lc);
        maintain(l);
        return l;
    } else {
        pushdown(r);
        r->lc = merge(l, r->lc);
        maintain(r->rc);
        maintain(r);
        return r;
    }
}

Node* merge(Node* a, Node* b, Node* c) {
    return merge(merge(a, b), c);
}

void rfree(Node* o) {
    if (!o) {
        return;
    }

    rfree(o->lc);
    rfree(o->rc);

    delete o;
}

void solve() {
    int n, q;
    cin >> n >> q;

    auto build_read = [&](int len) -> Node* {
        Node* o {};

        for (int i = 0; i < len; i++) {
            int x;
            cin >> x;
            o = merge(o, new Node(x));
        }

        return o;
    };

    auto root = build_read(n);

    while (q--) {
        string ty;
        cin >> ty;

        if (ty == "INSERT") {
            int ind, len;
            cin >> ind >> len;

            auto co = build_read(len);
            auto [rl, rr] = split(root, ind);
            root = merge(rl, co, rr);
            continue;
        } else if (ty == "MAX-SUM") {
            cout << root->v.v << endl;
            continue;
        }

        int ind, len;
        cin >> ind >> len;
        ind--;
        auto [rl, rm, rr] = split(root, ind, len);

        if (ty == "DELETE") {
            rfree(rm);
            root = merge(rl, rr);
            continue;
        }

        if (ty == "MAKE-SAME") {
            int x;
            cin >> x;
            rm->lazy += Lazy::c_set(x);
        } else if (ty == "REVERSE") {
            rm->lazy += Lazy::c_rev();
        } else if (ty == "GET-SUM") {
            cout << g_v(rm).sum << endl;
        }

        maintain(rm);
        root = merge(rl, rm, rr);
        dbg(root->v.sum);
    }

    rfree(root);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}