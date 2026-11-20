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

mt19937 cowng(1412);

template <typename T, size_t N>
struct Allocator {
    T v[N];
    int st[N - 1], st_ptr = N - 1;

    Allocator() {
        iota(rbegin(st), rend(st), 1);
    }

    int alloc() {
        return st[--st_ptr];
    }

    int alloc(const T& t) {
        int o = alloc();
        v[o] = t;
        return o;
    }

    void free(int ptr) {
        st[st_ptr++] = ptr;
    }

    T& operator[](int ptr) {
        return v[ptr];
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

    friend ostream& operator<<(ostream& os, const NV& nv) {
        return os << "{sum: " << nv.sum << " pref: " << nv.pref << " suff: " << nv.suff
           << " v: " << nv.v << "}";
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

    void clear() {
        set = NO_SET;
        rev = false;
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

struct N;

struct R {
    int ptr;

    R(int ptr = 0): ptr(ptr) {}

    friend ostream& operator<<(ostream& out, R o) {
        return out << "&" << o.ptr;
    }

    operator bool() const;

    N& operator*() const;

    N* operator->() const;
};

struct N {
    NV v;
    Lazy lazy;
    int x, siz, pri;
    R lc, rc;

    static N c_from(int x) {
        return {NV::c_from(x), Lazy::c_def(), x, 1, int(cowng()), {}, {}};
    }
};

Allocator<N, 1 << 19> heap;

R::operator bool() const {
    return ptr;
}

N& R::operator*() const {
    return heap[ptr];
}

N* R::operator->() const {
    return &heap[ptr];
}

R alloc(const N& n) {
    return heap.alloc(n);
}

void free(R o) {
    if (!o) {
        return;
    }
    free(o->lc);
    free(o->rc);
    heap.free(o.ptr);
}

void pushdown(R o) {
    assert(o);
    auto go = [&](R c) -> void {
        if (!c) {
            return;
        }
        c->lazy += o->lazy;
        o->lazy.apply_inplace(c->v, c->siz);
    };
    go(o->lc);
    go(o->rc);
    o->lazy.apply_inplace(o->x);
    if (o->lazy.rev) {
        swap(o->lc, o->rc);
    }
    o->lazy.clear();
}

void maintain(R o) {
    if (!o) {
        return;
    }
    o->siz = o->lc->siz + o->rc->siz + 1;
    o->v = o->lc->v + NV::c_from(o->x) + o->rc->v;
    o->lazy.apply_inplace(o->v, o->siz);
}

void split(R o, int ind, R& l, R& r) {
    if (!o) {
        assert(!ind);
        l = r = 0;
        return;
    }
    pushdown(o);
    if (ind <= o->lc->siz) {
        split(o->lc, ind, l, o->lc);
        r = o;
    } else {
        split(o->rc, ind - o->lc->siz - 1, o->rc, r);
        l = o;
    }
    maintain(o);
}

R merge(R l, R r) {
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

pair<R, R> split(R o, int ind) {
    R l, r;
    split(o, ind, l, r);
    return {l, r};
}

tuple<R, R, R> split(R o, int ind, int len) {
    R l, mid, r;
    split(o, ind, l, r);
    split(r, len, mid, r);
    return {l, mid, r};
}

R merge(R a, R b, R c) {
    return merge(merge(a, b), c);
}

void solve() {
    heap[0].v = NV::c_def();
    int n, q;
    cin >> n >> q;
    R root {};
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        root = merge(root, alloc(N::c_from(a)));
    }
    dbg("START");
    maintain(root);
    while (q--) {
        char t[10] {};
        cin >> t;
        if (t[2] == 'X') {
            cout << root->v.v << endl;
            continue;
        }
        int ind, len;
        cin >> ind >> len;
        if (t[2] == 'S') {
            auto [nl, nr] = split(root, ind);
            for (int i = 0; i < len; i++) {
                int x;
                cin >> x;
                nl = merge(nl, alloc(N::c_from(x)));
            }
            root = merge(nl, nr);
            continue;
        }
        ind--;
        auto [nl, nm, nr] = split(root, ind, len);
        if (t[2] == 'L') {
            free(nm);
            root = merge(nl, nr);
            continue;
        }
        if (t[2] == 'K') {
            int x;
            cin >> x;
            nm->lazy += Lazy::c_set(x);
            maintain(nm);
        } else if (t[2] == 'V') {
            nm->lazy += Lazy::c_rev();
            maintain(nm);
        } else if (t[2] == 'T') {
            cout << nm->v.sum << endl;
        }
        dbg(nl, nm, nr);
        root = merge(nl, nm, nr);
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}