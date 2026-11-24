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
// Sample grader: n, each station's block, then its kind (1 = C, 2 = D); prints the found
// block and kind of every station. Distances follow the rails: C goes left, D goes right.
static vector<int> grader_at, grader_kind, grader_right_d, grader_left_c;

extern "C" int getDistance(int i, int j) {
    if (i == j) return 0;
    long a = grader_at[i], b = grader_at[j];
    int ki = grader_kind[i], kj = grader_kind[j];
    if (ki == 1 && kj == 1) return 2 * grader_right_d[max(a, b)] - a - b;
    if (ki == 2 && kj == 2) return a + b - 2 * grader_left_c[min(a, b)];
    if (ki == 1) return a < b ? b - a : 2 * (grader_right_d[a] - grader_left_c[b]) - a + b;
    return b < a ? a - b : 2 * (grader_right_d[b] - grader_left_c[a]) + a - b;
}

int main() {
    int n;
    scanf("%d", &n);
    grader_at.resize(n);
    grader_kind.resize(n);
    for (auto& x : grader_at) scanf("%d", &x);
    for (auto& x : grader_kind) scanf("%d", &x);
    const int blocks = 1000000;
    vector<char> has_c(blocks), has_d(blocks);
    for (int i = 0; i < n; i++) (grader_kind[i] == 1 ? has_c : has_d)[grader_at[i]] = 1;
    grader_right_d.assign(blocks, 0);
    grader_left_c.assign(blocks, 0);
    for (int x = blocks - 2; x >= 0; x--) grader_right_d[x] = has_d[x + 1] ? x + 1 : grader_right_d[x + 1];
    for (int x = 1; x < blocks; x++) grader_left_c[x] = has_c[x - 1] ? x - 1 : grader_left_c[x - 1];
    vector<int> loc(n), type(n);
    findLocation(n, grader_at[0], loc.data(), type.data());
    for (int i = 0; i < n; i++) printf("%d %d\n", loc[i], type[i]);
}
