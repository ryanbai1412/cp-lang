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

const int maxn = 255;

void solve() {
	int n;
	cin >> n;
	string s;
	cin >> s;
	int arr[n];
	for (int i = 0; i < n; i++) {
		arr[i] = s[i] - 'A' + 1;
	}
	int cind = 0, pind = 1;
	vector<int> dp[2][n + 1];
	dp[cind][0].push_back(0);
	for (int i = n - 1; i >= 0; i--) {
		swap(cind, pind);
		for (auto& a : dp[cind]) {
			a.clear();
		}
		for (int j = 0; j < max(i + 1, n - i); j++) {
			auto &ans = dp[cind][j];
			ans = dp[pind][j];
			for (auto& a : dp[pind][j + 1]) {
				ans.push_back(arr[i] * arr[i] + a);
			}
			if (j) {
				for (auto& a : dp[pind][j - 1]) {
					ans.push_back(a - arr[i] * arr[i]);
				}
			}
			sort(begin(ans), end(ans));
			ans.erase(unique(begin(ans), end(ans)), ans.end());
		}
	}
	auto& ans = dp[cind][0];
	cout << sz(ans) << endl;
	for (auto& a : ans) {
		cout << a << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}