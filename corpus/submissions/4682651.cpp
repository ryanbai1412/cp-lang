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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

struct Node {
    int siz, lc, rc;

    int gsize() {
        if (siz < 0) {
            return 1;
        }
        return siz;
    }
} heap[(250 << 20) / sizeof(Node)];
int hind = 1;

void push(int& o, int l, int r, int x) {
    heap[hind] = heap[o];
    o = hind++;
    auto &[siz, lc, rc] = heap[o];
    if (l == r) {
        dbg(l, x);
        siz = -x;
    } else {
        int mid = (l + r) / 2;
        if (heap[lc].gsize() == mid - l + 1) {
            push(rc, mid + 1, r, x);
        } else {
            push(lc, l, mid, x);
        }
        siz = heap[lc].gsize() + heap[rc].gsize();
    }
}

int query(int o, int l, int r, int ind) {
    auto& [siz, lc, rc] = heap[o];
    if (l == r) {
        return -siz;
    } else {
        int mid = (l + r) / 2;
        if (ind <= mid) {
            return query(lc, l, mid, ind);
        } else {
            return query(rc, mid + 1, r, ind);
        }
    }
}

vector<int> roots {hind++};
const int maxn = 1e6 + 5;

void Init() {}

void TypeLetter(char c) {
    roots.push_back(roots.back());
    push(roots.back(), 0, maxn, c);
}

void UndoCommands(int x) {
    roots.push_back(roots[sz(roots) - x - 1]);
}

char GetLetter(int ind) {
    return query(roots.back(), 0, maxn, ind);
}