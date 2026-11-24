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

using vvc = vector<vector<char>>;

struct Psum {
    int n;
    vector<int> psum;

    Psum(const vector<int>& arr) : n(sz(arr)), psum(n + 1) {
        partial_sum(begin(arr), end(arr), psum.begin() + 1);
    }

    int query(int l, int r) const {
        return psum[r + 1] - psum[l];
    }
};

struct Rsum {
    vector<int> psum;

    Rsum(int n) : psum(n + 1) {}

    void add(int l, int r) {
        psum[l]++;
        psum[r + 1]--;
    }

    vector<int> query() {
        vector<int> tmp(sz(psum));
        partial_sum(begin(psum), end(psum), tmp.begin());
        return tmp;
    }
};

Psum pseq(const string& s, char c) {
    vector<int> tmp(sz(s));
    for (int i = 0; i < sz(s); i++) {
        tmp[i] = s[i] == c;
    }
    return Psum(tmp);
}

pair<vvc, vvc> solve(const string& s, const vector<int>& arr) {
    int n = sz(s), m = sz(arr);
    Psum whites = pseq(s, '_');
    vvc dp(n + 1, vector<char>(m + 1)), start(n + 1, vector<char>(m + 1)),
        xstart(n + 1, vector<char>(m + 1));
    dp[n][m] = start[n][m] = xstart[n][m] = true;
    for (int i = n - 1; i >= 0; i--) {
        if (s[i] != 'X') {
            dp[i][m] = start[i][m] = xstart[i][m] = dp[i + 1][m];
        }
        for (int j = 0; j < m; j++) {
            if (i + arr[j] <= n && !whites.query(i, i + arr[j] - 1)) {
                dp[i][j] = start[i][j] = xstart[i + arr[j]][j + 1];
            }
            if (s[i] != 'X') {
                xstart[i][j] = dp[i + 1][j];
                dp[i][j] |= dp[i + 1][j];
            }
        }
    }
    return {dp, start};
}

template <typename T>
T reversed(T x) {
    reverse(begin(x), end(x));
    return x;
}

string solve_puzzle(string s, vector<int> arr) {
    int n = sz(s), m = sz(arr);
    auto [suff, sufx] = solve(s, arr);
    auto [pref, _] = solve(reversed(s), reversed(arr));
    Rsum cblack(n);
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            bool pa;
            if (!i) {
                pa = !j;
            } else {
                pa = s[i - 1] != 'X' && pref[n - i + 1][m - j];
            }
            if (pa && sufx[i][j]) {
                dbg(i, j, i + arr[j] - 1);
                cblack.add(i, i + arr[j] - 1);
            }
        }
    }
    auto qblack = cblack.query();
    string ans;
    for (int i = 0; i < n; i++) {
        bool white = false;
        for (int j = 0; j <= m; j++) {
            white |= s[i] != 'X' && pref[n - i][m - j] && suff[i + 1][j];
        }
        bool black = !!qblack[i];
        dbg(i, white, black);
        if (white && !black) {
            ans.push_back('_');
        } else if (black && !white) {
            ans.push_back('X');
        } else if (black && white) {
            ans.push_back('?');
        } else {
            assert(false);
        }
    }
    return ans;
}

string solve_puzzle(string s, int n, int* arr) {
    return solve_puzzle(s, vector<int>(arr, arr + n));
}
// The original is the function the IOI grader calls. This `main` stands in for
// the grader: the row, k, then the k clues.
int main() {
	static char buf[200005];
	int k;
	if (scanf("%200004s %d", buf, &k) != 2) return 1;
	vector<int> c(k);
	for (auto& v : c) scanf("%d", &v);
	printf("%s\n", solve_puzzle(string(buf), c).c_str());
}
