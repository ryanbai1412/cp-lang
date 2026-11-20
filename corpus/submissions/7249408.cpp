// from o3

#include <bits/stdc++.h>
using namespace std;
using ll = long long;
const ll LINF = LLONG_MIN;

//--- Treap node ---
struct Node {
    int val;               // node's value
    uint32_t prio;         // heap priority
    int size;              // subtree size
    ll sum, pref, suf, best; // subtree sum / max-prefix / max-suffix / max-subarray
    bool rev, same;        // lazy flags: reverse, assign-all
    int same_val;          // value for assign-all
    Node *l, *r;           // children
};

// maximum number of concurrent nodes (sequence length ≤500 000)
static const int MAXN = 600000;
static Node pool[MAXN];
static int pool_ptr = 0;
static Node* freeList = nullptr;

// xorshift32 RNG for priorities
static uint32_t rnd_seed = 712687126;
inline uint32_t rnd() {
    rnd_seed ^= rnd_seed << 13;
    rnd_seed ^= rnd_seed >> 17;
    rnd_seed ^= rnd_seed << 5;
    return rnd_seed;
}

// allocate a fresh node (reuse from freeList if possible)
inline Node* newNode(int v) {
    Node* t;
    if (freeList) {
        t = freeList;
        freeList = freeList->l;  // use t->l as next-pointer in free list
    } else {
        t = &pool[pool_ptr++];
    }
    t->val       = v;
    t->prio      = rnd();
    t->size      = 1;
    t->sum       = v;
    t->pref      = v;
    t->suf       = v;
    t->best      = v;
    t->rev       = false;
    t->same      = false;
    t->same_val  = 0;
    t->l = t->r  = nullptr;
    return t;
}

inline int   getSize(Node* t) { return t ? t->size : 0; }
inline ll    getSum (Node* t) { return t ? t->sum  : 0;    }
inline ll    getPref(Node* t) { return t ? t->pref : LINF; }
inline ll    getSuf (Node* t) { return t ? t->suf  : LINF; }
inline ll    getBest(Node* t) { return t ? t->best : LINF; }

// apply a range-assign(v) to the entire subtree t
void applySame(Node* t, int v) {
    if (!t) return;
    t->same     = true;
    t->same_val = v;
    t->rev      = false;        // cancel any pending reverse
    t->val      = v;
    t->sum      = ll(t->size) * v;
    if (v >= 0) {
        t->pref = t->suf = t->best = t->sum;
    } else {
        // all elements = v < 0 → best non-empty subarray is just one element
        t->pref = t->suf = t->best = v;
    }
}

// apply a range-reverse to subtree t
void applyRev(Node* t) {
    if (!t) return;
    t->rev ^= 1;
    swap(t->l, t->r);
    swap(t->pref, t->suf);
}

// push lazy flags downward
void push(Node* t) {
    if (!t) return;
    if (t->same) {
        applySame(t->l, t->same_val);
        applySame(t->r, t->same_val);
        t->same = false;
    }
    if (t->rev) {
        applyRev(t->l);
        applyRev(t->r);
        t->rev = false;
    }
}

// recalc size, sum, pref, suf, best from children + own value
void update(Node* t) {
    if (!t) return;
    t->size = 1 + getSize(t->l) + getSize(t->r);
    ll Lsum = getSum(t->l);
    ll Rsum = getSum(t->r);
    t->sum = Lsum + t->val + Rsum;

    ll Lpref = getPref(t->l);
    ll Rpref = getPref(t->r);
    t->pref = max(Lpref, Lsum + t->val + max(0LL, Rpref));

    ll Lsuf = getSuf(t->l);
    ll Rsuf = getSuf(t->r);
    t->suf = max(Rsuf, Rsum + t->val + max(0LL, Lsuf));

    ll bestCross = max(0LL, Lsuf) + t->val + max(0LL, Rpref);
    t->best = bestCross;
    if (t->l) t->best = max(t->best, t->l->best);
    if (t->r) t->best = max(t->best, t->r->best);
}

// split t into [0..k-1] → l and [k..end] → r
void split(Node* t, int k, Node*& l, Node*& r) {
    if (!t) { l = r = nullptr; return; }
    push(t);
    if (getSize(t->l) >= k) {
        split(t->l, k, l, t->l);
        r = t;
        update(r);
    } else {
        split(t->r, k - getSize(t->l) - 1, t->r, r);
        l = t;
        update(l);
    }
}

// merge two treaps l + r (all keys in l come before keys in r)
Node* merge(Node* l, Node* r) {
    if (!l || !r) return l ? l : r;
    if (l->prio < r->prio) {
        push(l);
        l->r = merge(l->r, r);
        update(l);
        return l;
    } else {
        push(r);
        r->l = merge(l, r->l);
        update(r);
        return r;
    }
}

// free entire subtree t back into freeList
void freeNodes(Node* t) {
    if (!t) return;
    push(t);
    freeNodes(t->l);
    freeNodes(t->r);
    t->l = freeList;
    freeList = t;
}

//─── Main ───────────────────────────────────────────────────────────────────────
int main(){
    ios::sync_with_stdio(false);
    cin.tie(nullptr);

    int N, M;
    cin >> N >> M;
    Node* root = nullptr;

    // build initial sequence
    for(int i = 0; i < N; i++){
        int v; 
        cin >> v;
        root = merge(root, newNode(v));
    }

    while(M--){
        string op;
        cin >> op;
        if (op == "INSERT") {
            int pos, tot;
            cin >> pos >> tot;
            Node *mid = nullptr;
            while(tot--){
                int v; 
                cin >> v;
                mid = merge(mid, newNode(v));
            }
            Node *a, *b;
            split(root, pos, a, b);
            root = merge( merge(a, mid), b );

        } else if (op == "DELETE") {
            int pos, tot;
            cin >> pos >> tot;
            Node *a, *b, *mid;
            split(root, pos, a, b);
            split(b, tot, mid, b);
            freeNodes(mid);
            root = merge(a, b);

        } else if (op == "MAKE-SAME") {
            int pos, tot, c;
            cin >> pos >> tot >> c;
            Node *a, *b, *mid;
            split(root, pos, a, b);
            split(b, tot, mid, b);
            applySame(mid, c);
            root = merge( merge(a, mid), b );

        } else if (op == "REVERSE") {
            int pos, tot;
            cin >> pos >> tot;
            Node *a, *b, *mid;
            split(root, pos, a, b);
            split(b, tot, mid, b);
            applyRev(mid);
            root = merge( merge(a, mid), b );

        } else if (op == "GET-SUM") {
            int pos, tot;
            cin >> pos >> tot;
            Node *a, *b, *mid;
            split(root, pos, a, b);
            split(b, tot, mid, b);
            ll ans = getSum(mid);
            cout << ans << "\n";
            root = merge( merge(a, mid), b );

        } else if (op == "MAX-SUM") {
            ll ans = root ? root->best : 0;
            cout << ans << "\n";
        }
    }
    return 0;
}