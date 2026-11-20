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
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n;
	cin >> n;
	string sa, sb;
	cin >> sa;
	int arra[n], arrb[n];
	for (auto& a : arra) {
		cin >> a;
	}
	cin >> sb;
	for (auto& a : arrb) {
		cin >> a;
	}
	int dp[n + 1][n + 1] {};
	for (int i = n - 1; i >= 0; i--) {
		for (int j = n - 1; j >= 0; j--) {
			dp[i][j] = max(dp[i + 1][j], dp[i][j + 1]);
			if (sa[i] != sb[j] &&
				((sa[i] == 'W' && arra[i] > arrb[j]) || (sa[i] == 'L' && arra[i] < arrb[j]))) {
				dp[i][j] = max(dp[i][j], arra[i] + arrb[j] + dp[i + 1][j + 1]);
			}
		}
	}
	cout << dp[0][0] << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}