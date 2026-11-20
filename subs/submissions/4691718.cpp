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

extern "C" int getDistance(int i, int j);

extern "C" void findLocation(int n, int first, int loc[], int type[]) {
    int dist[n];
    dist[0] = 0;
    for (int i = 1; i < n; i++) {
        dist[i] = getDistance(0, i);
    }
    int ord[n - 1];
    iota(ord, ord + n - 1, 1);
    sort(ord, ord + n - 1, [&](int a, int b) -> bool {
        return dist[a] < dist[b];
    });
    loc[0] = first;
    type[0] = 1;
    map<int, bool> m;
    auto meq = [&](int k, bool v) -> bool {
        auto it = m.find(k);
        return it != m.end() && it->second == v;
    };
    m[first] = false;
    int l = 0, r = -1, openr = first, closel = 1e9;
    bool bl = true, br = false;
    for (auto& a : ord) {
        if (r == -1) {
            r = a;
            loc[a] = first + dist[a];
            type[a] = 2;
            m[loc[a]] = 2;
            closel = loc[a];
            continue;
        }
        int ql = getDistance(l, a), qr = getDistance(r, a);
        int posl = loc[l] + ql, posr = loc[r] - qr, mid = (loc[r] + posl - qr) / 2;
        auto it = m.find(mid);
        bool cl;
        if (it != m.end()) {
            cl = !it->second;
        } else {
            cl = mid > openr;
        }
        if (cl) {
            loc[a] = posl;
            type[a] = 2;
            if (loc[a] > loc[r]) {
                r = a;
                br = false;
            } else {
                bl = true;
            }
            closel = min(closel, loc[a]);
        } else {
            loc[a] = posr;
            type[a] = 1;
            if (loc[a] < loc[l]) {
                l = a;
                bl = false;
            } else {
                br = true;
            }
            openr = max(openr, loc[a]);
        }
        m[loc[a]] = type[a] == 2;
        dbg(a, loc[a]);
    }
    for (int i = 0; i < n; i++) {
        dbg(loc[i], type[i]);
    }
}