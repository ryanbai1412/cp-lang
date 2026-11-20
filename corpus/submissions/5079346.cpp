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

constexpr int maxn = 50;

bitset<2> bfrom(bool b0, bool b1) {
    return bitset<2>((b1 << 1) | b0);
}

bool matches(int x, const bitset<2>& bs) {
    return x == -1 || bs[x];
}

struct S {
    const vector<int>* arr;
    int ind, len;

    S(const vector<int>& arr) : arr(&arr), ind(0), len(0) {}

    bitset<2> peek_next() const {
        if (len > 1) {
            return bfrom(false, true);
        } else if (len == 1 || ind == sz(*arr)) {
            return bfrom(true, false);
        } else {
            return bfrom(true, true);
        }
    }

    bool peek_matches(int x) const {
        return matches(x, peek_next());
    }

    void next(bool b) {
        assert(peek_matches(b));
        if (len > 0) {
            len--;
        } else if (b) {
            len = (*arr)[ind++];
        }
    }

    bool finished() const {
        return ind == sz(*arr) && len <= 1;
    }
};

int n, m, arr[maxn][maxn];
vector<S> indc;
vector<int> arrr[maxn], arrc[maxn];

void dfs(int r, int c, const S& indr) {
    if (r == n + 1) {
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < m; j++) {
                cout << ".*"[arr[i][j]];
            }
            cout << endl;
        }
        exit(0);
    } else if (c == m) {
        if (indr.finished()) {
            dfs(r + 1, 0, S(arrr[r + 1]));
        }
        return;
    }
    for (int i = 0; i < 2; i++) {
        if (indr.peek_matches(i) && indc[c].peek_matches(i)) {
            auto tmpr = indr, tmpc = indc[c];
            int tmpv = i;
            tmpr.next(i);
            indc[c].next(i);
            swap(tmpv, arr[r][c]);
            dfs(r, c + 1, tmpr);
            swap(tmpv, arr[r][c]);
            arr[r][c] = -1;
            indc[c] = tmpc;
        }
    }
}

void solve() {
    cin >> n >> m;
    auto read = [&](vector<int>& arr) -> void {
        int x;
        while (true) {
            cin >> x;
            if (!x) {
                return;
            }
            arr.push_back(x);
        }
    };
    for (int i = 0; i < n; i++) {
        read(arrr[i]);
    }
    for (int i = 0; i < m; i++) {
        read(arrc[i]);
        indc.emplace_back(arrc[i]);
    }
    vector<int> tmp;
    indc.emplace_back(tmp);
    memset(arr, -1, sizeof(arr));
    dfs(0, 0, S(arrr[0]));
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}