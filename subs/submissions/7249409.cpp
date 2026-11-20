// from o3

#include <bits/stdc++.h>
using namespace std;

/* ---------- fast input (≈2 × faster than cin) ---------- */
inline int fastInt() {
    int x = 0, f = 1;  char c = getchar_unlocked();
    while (c != '-' && (c < '0' || c > '9')) c = getchar_unlocked();
    if (c == '-') { f = -1; c = getchar_unlocked(); }
    while (c >= '0' && c <= '9') { x = x * 10 + (c - '0'); c = getchar_unlocked(); }
    return x * f;
}
inline void fastStr(char *s) {
    char c = getchar_unlocked();
    while (isspace(c)) c = getchar_unlocked();
    int p = 0;
    while (!isspace(c) && c != EOF) { s[p++] = c; c = getchar_unlocked(); }
    s[p] = '\0';
}

/* ---------- node structure ---------- */
const long long NEG_INF = -4e18;
struct Node {
    Node *l, *r;
    int pr, val, size;
    long long sum, lmax, rmax, tmax;
    bool rev, same;
    int sameVal;
};
const int MAXN = 600000 + 5;          // 5 × 10^5 (max live) + cushion
Node pool[MAXN];
int poolPtr = 0;
vector<Node*> trash;

/* ---------- helpers ---------- */
inline int sz(Node *p)           { return p ? p->size : 0; }
inline long long sm(Node *p)     { return p ? p->sum  : 0; }
inline long long lmx(Node *p)    { return p ? p->lmax : NEG_INF; }
inline long long rmx(Node *p)    { return p ? p->rmax : NEG_INF; }
inline long long tmx(Node *p)    { return p ? p->tmax : NEG_INF; }

inline Node* newNode(int v) {
    Node* p;
    if (!trash.empty()) { p = trash.back(); trash.pop_back(); }
    else                { p = &pool[poolPtr++]; }
    p->l = p->r = nullptr;
    p->pr = rand();
    p->val = v;
    p->size = 1;
    p->sum = p->lmax = p->rmax = p->tmax = v;
    p->rev = p->same = false;
    return p;
}

/* ---------- lazy operations ---------- */
inline void applySame(Node* p, int v) {
    if (!p) return;
    p->val     = v;
    p->sum     = 1LL * v * p->size;
    if (v >= 0) p->lmax = p->rmax = p->tmax = p->sum;
    else        p->lmax = p->rmax = p->tmax = v;
    p->same    = true;
    p->sameVal = v;
    p->rev     = false;
}
inline void applyRev(Node* p) {
    if (!p) return;
    p->rev ^= 1;
    swap(p->l, p->r);
    swap(p->lmax, p->rmax);
}
inline void pushDown(Node* p) {
    if (!p) return;
    if (p->same) {
        applySame(p->l, p->sameVal);
        applySame(p->r, p->sameVal);
        p->same = false;
    }
    if (p->rev) {
        applyRev(p->l);
        applyRev(p->r);
        p->rev = false;
    }
}

/* ---------- recalc ---------- */
inline void pushUp(Node* p) {
    if (!p) return;
    p->size = sz(p->l) + sz(p->r) + 1;
    p->sum  = sm(p->l) + sm(p->r) + p->val;

    p->lmax = max(lmx(p->l),
                  sm(p->l) + p->val + max(0LL, lmx(p->r)));
    p->rmax = max(rmx(p->r),
                  sm(p->r) + p->val + max(0LL, rmx(p->l)));

    long long cross = p->val +
                      max(0LL, rmx(p->l)) +
                      max(0LL, lmx(p->r));
    p->tmax = max(cross, max(tmx(p->l), tmx(p->r)));
}

/* ---------- FHQ split / merge ---------- */
void split(Node* root, int k, Node* &a, Node* &b) {   // first k to a
    if (!root) { a = b = nullptr; return; }
    pushDown(root);
    if (sz(root->l) >= k) {
        split(root->l, k, a, root->l);
        b = root;
    } else {
        split(root->r, k - sz(root->l) - 1, root->r, b);
        a = root;
    }
    pushUp(root);
}
Node* merge(Node* a, Node* b) {
    if (!a || !b) return a ? a : b;
    if (a->pr < b->pr) {
        pushDown(a);
        a->r = merge(a->r, b);
        pushUp(a);
        return a;
    } else {
        pushDown(b);
        b->l = merge(a, b->l);
        pushUp(b);
        return b;
    }
}

/* ---------- recycle subtree ---------- */
void reclaim(Node* p) {
    if (!p) return;
    static Node* stack[MAXN];
    int top = 0;
    stack[top++] = p;
    while (top) {
        Node* cur = stack[--top];
        if (cur->l) stack[top++] = cur->l;
        if (cur->r) stack[top++] = cur->r;
        trash.push_back(cur);
    }
}

/* ---------- build treap from stdin (O(n)) ---------- */
Node* buildSeq(int n) {
    vector<Node*> st; st.reserve(n);
    Node* last = nullptr;
    for (int i = 0; i < n; ++i) {
        int v = fastInt();
        Node* cur = newNode(v);
        last = nullptr;
        while (!st.empty() && st.back()->pr > cur->pr) {
            pushUp(st.back());
            last = st.back();
            st.pop_back();
        }
        cur->l = last;
        if (!st.empty()) st.back()->r = cur;
        st.push_back(cur);
    }
    while (st.size() > 1) { pushUp(st.back()); st.pop_back(); }
    pushUp(st.back());
    return st.back();
}

/* ---------- main ---------- */
int main() {
    srand(712367);                 // deterministic for reproducibility
    int N = fastInt(), M = fastInt();

    Node* root = buildSeq(N);

    char cmd[15];
    for (int op = 0; op < M; ++op) {
        fastStr(cmd);

        if (cmd[0] == 'I') {                            // INSERT
            int pos = fastInt(), tot = fastInt();
            Node *L, *R;
            split(root, pos, L, R);                     // after pos ⇒ first pos go left
            Node* mid = buildSeq(tot);
            root = merge(merge(L, mid), R);

        } else if (cmd[0] == 'D') {                     // DELETE
            int pos = fastInt(), tot = fastInt();
            Node *L, *tmp, *mid, *R;
            split(root, pos - 1, L, tmp);
            split(tmp, tot, mid, R);
            reclaim(mid);
            root = merge(L, R);

        } else if (cmd[0] == 'M' && cmd[2] == 'K') {    // MAKE-SAME
            int pos = fastInt(), tot = fastInt(), c = fastInt();
            Node *L, *tmp, *mid, *R;
            split(root, pos - 1, L, tmp);
            split(tmp, tot, mid, R);
            applySame(mid, c);
            root = merge(merge(L, mid), R);

        } else if (cmd[0] == 'R') {                     // REVERSE
            int pos = fastInt(), tot = fastInt();
            Node *L, *tmp, *mid, *R;
            split(root, pos - 1, L, tmp);
            split(tmp, tot, mid, R);
            applyRev(mid);
            root = merge(merge(L, mid), R);

        } else if (cmd[0] == 'G') {                     // GET-SUM
            int pos = fastInt(), tot = fastInt();
            if (tot == 0) { puts("0"); continue; }
            Node *L, *tmp, *mid, *R;
            split(root, pos - 1, L, tmp);
            split(tmp, tot, mid, R);
            printf("%lld\n", mid ? mid->sum : 0LL);
            root = merge(merge(L, mid), R);

        } else {                                        // MAX-SUM
            printf("%lld\n", tmx(root));
        }
    }
    return 0;
}