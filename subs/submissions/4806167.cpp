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

long memo[int(1e5)];

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

long dp(int mask) {
    if (mask == ((1 << 25) - 1)) {
        return 1;
    }
    auto id = [&](int x, int y) -> int {
        return x * 5 + y;
    };
    bool arr[5][5];
    for (int i = 0; i < 5; i++) {
        for (int j = 0; j < 5; j++) {
            arr[i][j] = on(mask, id(i, j));
        }
    }
    int len[5], key = 0;
    for (int i = 0; i < 5; i++) {
        int& j = len[i];
        for (j = 0; j < 5 && arr[i][j]; j++)
            ;
        key = key * 6 + j;
    }
    auto& ans = memo[key];
    if (ans != -1) {
        return ans;
    }
    ans = 0;
    for (int i = 0; i < 5; i++) {
        if (len[i] < 5 && (!i || len[i - 1] > len[i])) {
            ans += dp(mask | (1 << id(i, len[i])));
        }
    }
    return ans;
}

struct Solver1 {
    long memo[int(1e5)];
    vector<int> targ, itarg;

    Solver1(const vector<int>& targ) : targ(targ), itarg(25, -1) {
        memset(memo, -1, sizeof(memo));
        for (int i = 0; i < sz(targ); i++) {
            itarg[targ[i]] = i;
        }
    }

    long dp1(int mask) {
        if (mask == ((1 << 25) - 1)) {
            return 1;
        }
        auto id = [&](int x, int y) -> int {
            return x * 5 + y;
        };
        bool arr[5][5];
        for (int i = 0; i < 5; i++) {
            for (int j = 0; j < 5; j++) {
                arr[i][j] = on(mask, id(i, j));
            }
        }
        int len[5], key = 0;
        for (int i = 0; i < 5; i++) {
            int& j = len[i];
            for (j = 0; j < 5 && arr[i][j]; j++)
                ;
            key = key * 6 + j;
        }
        auto& ans = memo[key];
        if (ans != -1) {
            return ans;
        }
        ans = 0;
        int cur = __builtin_popcount(mask);
        for (int i = 0; i < 5; i++) {
            if (len[i] < 5 && (!i || len[i - 1] > len[i])) {
                int cid = id(i, len[i]);
                if (itarg[cur] != -1 && itarg[cur] != cid) {
                    continue;
                }
                ans += dp1(mask | (1 << cid));
            }
        }
        return ans;
    }
    
    static long go(const vector<int>& arr) {
        return Solver1(arr).dp1(0);
    }
};

void solve() {
    memset(memo, -1, sizeof(memo));
    char c;
    cin >> c;
    if (c == 'W') {
        string s;
        cin >> s;
        int arr[25];
        for (int i = 0; i < 25; i++) {
            arr[i] = s[i] - 'A';
        }
        bool vis[25] {};
        vector<int> cur;
        long ans = 0;
        for (int i = 0; i < 25; i++) {
            cur.push_back(0);
            for (int j = 0; j < arr[i]; j++) {
                if (!vis[j]) {
                    cur.back() = j;
                    for (int k = 0; k <= i; k++) {
                        cerr << cur[k] << " \n"[k == i];
                    }
                    ans += Solver1::go(cur);
                }
            }
            cur.back() = arr[i];
            vis[arr[i]] = true;
        }
        cout << ans + 1 << endl;
    } else {
        long x;
        cin >> x;
        x--;
        bool vis[25] {};
        vector<int> cur;
        for (int i = 0; i < 25; i++) {
            cur.push_back(0);
            for (int j = 0; j < 25; j++) {
                if (!vis[j]) {
                    cur.back() = j;
                    long cans = Solver1::go(cur);
                    if (x < cans) {
                        break;
                    } else {
                        x -= cans;
                    }
                }
            }
            vis[cur.back()] = true;
        }
        for (auto& a : cur) {
            cout << char('A' + a);
        }
        cout << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}