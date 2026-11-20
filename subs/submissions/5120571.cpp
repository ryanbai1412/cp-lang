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

void dfs(vector<pair<int, int>> arr) {
    if (sz(arr) == 1) {
        answer(arr[0].second, arr[0].first);
        return;
    }
    sort(begin(arr), end(arr));
    int x = arr[(sz(arr) - 1) / 2].first;
    bool b = query(0, x);
    vector<pair<int, int>> narr;
    for (auto& [p, v] : arr) {
        dbg(p, v);
        if ((b && p <= x) || (!b && p > x)) {
            narr.emplace_back((p + v) % n, v);
        }
    }
    dfs(narr);
}

constexpr int bsize = 10;

void solve() {
    cout << "GetSize" << endl;
    n = read_int();
    mt19937 cowng(1412 * 2);
    int x;
    do {
        x = int(cowng() % (n - bsize + 1));
    } while (!query(x, x + bsize - 1));
    vector<pair<int, int>> arr;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < bsize; j++) {
            arr.emplace_back((x + i + j) % n, i);
        }
    }
    dfs(arr);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}