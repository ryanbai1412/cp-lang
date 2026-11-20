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

#define long int64_t
#define sz(x) int(std::size(x))

int read_int() {
    int ans = 0;
    char c;
    while ((c = getchar_unlocked()) != '\n') {
        ans = ans * 10 + c - '0';
    }
    return ans;
}

int n;

bool query(int l, int r) {
    cout << "Look " << l << " " << r << endl;
    char c = getchar_unlocked();
    while (getchar_unlocked() != '\n')
        ;
    return c == 'T';
}

void answer(int v, int p) {
    cout << "Guess " << v << " " << p << endl;
}

template <int bsize>
void dfs1(array<int, bsize> p, int v) {
    int cx = p[bsize - 1];
    if (query(cx, cx)) {
        answer(v, cx);
        return;
    }
    array<int, bsize - 1> nxt;
    copy(begin(p), end(p) - 1, nxt.begin());
    for (auto& a : nxt) {
        a = (a + v) % n;
    }
    dfs1<bsize - 1>(nxt, v);
}

template <>
void dfs1<1>(array<int, 1> p, int v) {
    answer(v, p[0]);
}

template <int bsize>
void dfs(vector<pair<array<int, bsize>, int>> arr) {
    dbg(sz(arr));
    if (sz(arr) == 1) {
        dfs1<bsize>(arr[0].first, arr[0].second);
        return;
    } else if (sz(arr) <= 5) {
        exit(1);
    }
    for (auto& [p, _v] : arr) {
        sort(begin(p), end(p));
    }
    sort(begin(arr), end(arr));
    int x = arr[(sz(arr) - 1) / 2].first[0];
    bool b = query(0, x);
    vector<pair<array<int, bsize>, int>> narr;
    for (auto& [p, v] : arr) {
        if ((b && p[0] <= x) || (!b && p[bsize - 1] > x)) {
            for (auto& a : p) {
                a = (a + v) % n;
            }
            narr.emplace_back(p, v);
        }
    }
    dfs<bsize>(narr);
}

void solve() {
    cout << "GetSize" << endl;
    n = read_int();
    mt19937 cowng(1412);
    int x;
    do {
        x = int(cowng() % (n - 1));
    } while (!query(x, x + 1));
    vector<pair<array<int, 2>, int>> arr(n);
    for (int i = 0; i < n; i++) {
        arr[i] = {{(x + i) % n, (x + i + 1) % n}, i};
    }
    dfs<2>(arr);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}