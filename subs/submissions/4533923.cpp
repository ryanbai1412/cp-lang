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
#define sz(x) int((x).size())

enum Type {
	// person below me is x
	Below,
	// person above me is x
	Above,
	Other
};

struct A {
	string poster, msg;
	Type type;
	int ind;
};

void solve(const vector<A> &arr) {
	int n = sz(arr);
	auto cnt = [&](int a, int b) -> int {
		if (a == n || b == n) {
			return 0;
		}
		int ans = 0;
		if (arr[a].type == Below && arr[a].msg == arr[b].poster) {
			ans++;
		}
		if (arr[b].type == Above && arr[a].poster == arr[b].msg) {
			ans++;
		}
		return ans;
	};
	int dp[1 << n][n + 1] {};
	for (int i = (1 << n) - 1; i >= 0; i--) {
		for (int j = 0; j <= n; j++) {
			for (int k = 0; k < n; k++) {
				if (!((i >> k) & 1)) {
					dp[i][j] = max(dp[i][j], dp[i | (1 << k)][k] + cnt(j, k));
				}
			}
		}	
	}
	cout << dp[0][n] << endl;
	int cur = 0, prev = n;
	for (int i = 0; i < n; i++) {
		pair<int, int> ans {-1, -1};
		for (int k = 0; k < n; k++) {
			if (!((cur >> k) & 1)) {
				ans = max(ans, pair<int, int> {dp[i | (1 << k)][k] + cnt(prev, k), k});
			}
		}
		int k = ans.second;
		cur |= 1 << k;
		prev = k;
		cout << k << " \n"[i == n - 1];
	}
}

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
	}
	vector<A> arr(m);
	for (int i = 0; i < m; i++) {
		string a, b, c;
		cin >> a >> b >> c;
		Type type = Other;
		if (c == "loushang") {
			type = Below;
		} else if (c == "louxia") {
			type = Above;
		}
		arr[i] = {a, b, type, i};
	}
	solve(arr);
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int tcs;
	cin >> tcs;
	while (tcs--) {
		solve();
	}
}