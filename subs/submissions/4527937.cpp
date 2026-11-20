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

void solve(int m, vector<int> arr) {
	int n = sz(arr);
	int nxt[n + 1] {};
	for (int i = 1; i <= n; i++) {
		int j = i - 1;
		while (j) {
			j = nxt[j];
			if (arr[j] == arr[i - 1]) {
				nxt[i] = j + 1;
				break;
			}
		}
		dbg(i, nxt[i]);
	}
	for (int i = 0; i < n; i++) {
		if (!i) {
			printf("dp[%d] = 1 + (dp[%d] + %d * dp[0]) / %d\n", i, i + 1, m - 1, m);
			continue;
		}
		int j = i;
		while (j) {
			j = nxt[j];
			if (arr[j] == arr[i]) {
				j++;
				break;
			}
		}
		printf("dp[%d] = dp[%d] + (dp[%d] - dp[%d]) / %d\n", i, nxt[i], i + 1, j, m);
	}
}

void solve() {
	int m, t;
	cin >> m >> t;
	while (t--) {
		int n;
		cin >> n;
		vector<int> arr(n);
		for (auto& a : arr) {
			cin >> a;
		}
		solve(m, arr);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}