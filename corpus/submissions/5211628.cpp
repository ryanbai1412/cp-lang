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

constexpr int IINF = 1e9;

struct NV {
    int sum, pref, suff, v;

    NV& operator+=(const NV& n) {
        v = max({v, n.v, suff + n.pref});
        pref = max(pref, sum + n.pref);
        suff = max(n.suff, suff + n.sum);
        sum += n.sum;
        return *this;
    }

    friend NV operator+(NV a, const NV& b) {
        return a += b;
    }

    static NV c_def() {
        return {0, -IINF, -IINF, -IINF};
    }

    static NV c_from(int x) {
        return {x, x, x, x};
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

    void apply(NV& nv, int siz) {
        if (set != NO_SET) {
            nv.sum = set * siz;
            nv.pref = nv.suff = nv.v = max(set, set * siz);
        } else if (rev) {
            swap(nv.pref, nv.suff);
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
    int x, siz;
    Node *p, *c[2];

    Node(int x)
        : v(NV::c_from(x)), lazy(Lazy::c_def()), x(x), siz(1), p {}, c {} {}
    ~Node() {
        delete c[0];
        delete c[1];
    }

    void pushdown() {
        for (auto& a : c) {
            if (!a) {
                continue;
            }
            a->lazy += lazy;
            lazy.apply(a->v, a->siz);
        }
        if (lazy.set != Lazy::NO_SET) {
            x = lazy.set;
        }
        if (lazy.rev) {
            swap(c[0], c[1]);
        }
        lazy = Lazy::c_def();
    }

    void maintain() {
        siz = g_siz(c[0]) + 1 + g_siz(c[1]);
        v = g_v(c[0]) + NV::c_from(x) + g_v(c[1]);
        lazy.apply(v, siz);
    }

    int side() const {
        return p->c[1] == this;
    }

    void rotate() {
        p->pushdown();
        pushdown();
        int s = side();
        Node* cp = p;
        if (cp->p) {
            cp->p->c[cp->side()] = this;
        }
        p = cp->p;
        cp->c[s] = c[!s];
        if (c[!s]) {
            c[!s]->p = cp;
        }
        c[!s] = cp;
        cp->p = this;
        cp->maintain();
        maintain();
    }

    void splay() {
        while (p) {
            if (!p->p) {
                rotate();
            } else if (side() == p->side()) {
                p->rotate();
                rotate();
            } else {
                rotate();
                rotate();
            }
        }
    }

    static pair<Node*, Node*> split(Node* o, int ind) {
        if (!ind) {
            return {nullptr, o};
        } else if (o->siz == ind) {
            return {o, nullptr};
        }
        while (true) {
            o->pushdown();
            int csz = g_siz(o->c[0]);
            if (ind == csz) {
                break;
            } else if (ind < csz) {
                o = o->c[0];
            } else {
                o = o->c[1];
                ind -= csz + 1;
            }
        }
        o->splay();
        Node* l = o->c[0];
        l->p = nullptr;
        o->c[0] = nullptr;
        o->maintain();
        return {l, o};
    }

    static Node* merge(Node* l, Node* r) {
        if (!l) {
            return r;
        } else if (!r) {
            return l;
        }
        while (true) {
            l->pushdown();
            if (!l->c[1]) {
                break;
            }
            l = l->c[1];
        }
        l->splay();
        l->c[1] = r;
        r->p = l;
        l->maintain();
        return l;
    }

    static tuple<Node*, Node*, Node*> split(Node* o, int ind, int len) {
        auto [l, tmp] = split(o, ind);
        auto [m, r] = split(tmp, len);
        return {l, m, r};
    }

    static Node* construct(const vector<int>& arr, int l, int r) {
        if (l == r) {
            return nullptr;
        }
        int mid = (l + r) / 2;
        Node* o = new Node(arr[mid]);
        o->c[0] = construct(arr, l, mid);
        o->c[1] = construct(arr, mid + 1, r);
        for (auto& a : o->c) {
            if (a) {
                a->p = o;
            }
        }
        o->maintain();
        return o;
    }

    static Node* construct(const vector<int>& arr) {
        return construct(arr, 0, sz(arr));
    }

    static Node* merge(Node* a, Node* b, Node* c) {
        return merge(merge(a, b), c);
    }

    static int g_siz(Node* o) {
        return o ? o->siz : 0;
    }

    static NV g_v(Node* o) {
        return o ? o->v : NV::c_def();
    }
};

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    Node* root = Node::construct(arr);
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
            auto [nl, nr] = Node::split(root, ind);
            vector<int> carr(len);
            for (auto& a : carr) {
                cin >> a;
            }
            root = Node::merge(nl, Node::construct(carr), nr);
            continue;
        }
        ind--;
        auto [nl, nm, nr] = Node::split(root, ind, len);
        if (s == "DELETE") {
            delete nm;
            root = Node::merge(nl, nr);
            continue;
        }
        if (s == "MAKE-SAME") {
            int x;
            cin >> x;
            nm->lazy += Lazy::c_set(x);
            nm->maintain();
        } else if (s == "REVERSE") {
            nm->lazy += Lazy::c_rev();
            nm->maintain();
        } else if (s == "GET-SUM") {
            cout << Node::g_v(nm).sum << endl;
        } else {
            assert(false);
        }
        root = Node::merge(nl, nm, nr);
    }
    delete root;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}