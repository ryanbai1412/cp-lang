#ifndef LOCAL
#pragma GCC optimize("Ofast")
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

struct IO {
    char buf[21 << 20], *i = buf;

    IO() {
        std::ignore = fread_unlocked(buf, 1, sizeof(buf), stdin);
    }

    IO& operator>>(int& x) {
        while (*i < '-') {
            i++;
        }
        x = 0;
        bool neg = false;
        if (*i == '-') {
            neg = true;
            i++;
        }
        while (*i >= '0') {
            x = x * 10 + (*(i++) ^ '0');
        }
        if (neg) {
            x = -x;
        }
        return *this;
    }

    IO& operator>>(string& s) {
        while (*i < '-') {
            i++;
        }
        s.clear();
        while (*i >= '-') {
            s += *(i++);
        }
        return *this;
    }
} in;

struct R {
    mt19937_64 cowng = mt19937_64(1412);

    uint32_t operator()() {
        return cowng();
    }
} R;

struct NV {
    int pref, suff, sum, v;

    NV operator+(const NV& x) const {
        return {max(pref, sum + x.pref), max(x.suff, x.sum + suff), sum + x.sum,
                max({v, x.v, suff + x.pref})};
    }

    static NV c_def() {
        constexpr int ninf = -1e9;
        return {ninf, ninf, 0, ninf};
    }

    static NV c_from(int x) {
        return {x, x, x, x};
    }

    static NV c_set(int x, int len) {
        int v = max(x, x * len);
        return {v, v, x * len, v};
    }
};

struct Lazy {
    static constexpr int16_t NO_SET = INT16_MIN;

    int16_t set;
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
        return {int16_t(x), false};
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

    Node() {}
    Node(int x)
        : pri(R()),
          siz(1),
          x(x),
          v(NV::c_from(x)),
          lazy(Lazy::c_def()),
          lc {},
          rc {} {}
};

struct A {
    static constexpr int MAXN = 5e5 + 5;

    Node heap[MAXN];
    vector<int> inds;

    A() {
        inds.resize(MAXN);
        iota(begin(inds), end(inds), 0);
    }

    Node* alloc(int x) {
        int ind = inds.back();
        inds.pop_back();

        return &(heap[ind] = Node(x));
    }

    void free(Node* o) {
        int ind = int(o - heap);
        inds.push_back(ind);
    }
} A;

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

    if (o->lazy.set != Lazy::NO_SET) {
        o->v = NV::c_set(o->lazy.set, o->siz);
        return;
    }
    o->v = g_v(o->lc) + NV::c_from(o->x) + g_v(o->rc);

    if (o->lazy.rev) {
        swap(o->v.pref, o->v.suff);
    }
}

void pushdown(Node* o) {
    if (!o) {
        return;
    }

    if (o->lazy.set != Lazy::NO_SET) {
        int16_t cs = o->x = o->lazy.set;
        if (o->lc) {
            o->lc->lazy.set = cs;
        }
        if (o->rc) {
            o->rc->lazy.set = cs;
        }
        o->lazy.set = Lazy::NO_SET;
    }
    if (o->lazy.rev) {
        swap(o->lc, o->rc);
        if (o->lc) {
            o->lc->lazy.rev ^= true;
        }
        if (o->rc) {
            o->rc->lazy.rev ^= true;
        }
        o->lazy.rev = false;
    }
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

    A.free(o);
}

Node* build(const vector<int>& arr, int l, int r) {
    if (l == r) {
        return nullptr;
    }

    int mid = (l + r) / 2;
    auto o = A.alloc(arr[mid]);
    o->lc = build(arr, l, mid);
    o->rc = build(arr, mid + 1, r);
    maintain(o);
    return o;
}

void solve() {
    int n, q;
    in >> n >> q;

    vector<int> arr;
    auto build_read = [&](int len) -> Node* {
        arr.resize(len);
        for (auto& a : arr) {
            in >> a;
        }

        return build(arr, 0, len);
    };

    auto root = build_read(n);

    string ty;
    while (q--) {
        in >> ty;

        if (ty[0] == 'I') {  // INSERT
            int ind, len;
            in >> ind >> len;

            auto co = build_read(len);
            auto [rl, rr] = split(root, ind);
            root = merge(rl, co, rr);
            continue;
        } else if (ty[2] == 'X') {  // MAX-SUM
            cout << root->v.v << endl;
            continue;
        }

        int ind, len;
        in >> ind >> len;
        ind--;
        auto [rl, rm, rr] = split(root, ind, len);

        if (ty[0] == 'D') {  // DELETE
            rfree(rm);
            root = merge(rl, rr);
            continue;
        }

        if (ty[0] == 'M') {  // MAKE-SAME
            int x;
            in >> x;
            rm->lazy += Lazy::c_set(x);
            maintain(rm);
        } else if (ty[0] == 'R') {  // REVERSE
            rm->lazy += Lazy::c_rev();
            maintain(rm);
        } else if (ty[0] == 'G') {  // GET-SUM
            cout << g_v(rm).sum << endl;
        }

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