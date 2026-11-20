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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n, m;
	cin >> n >> m;
	vector<pair<int, string>> arr[100];
	for (int i = 0; i < n; i++) {
		int ind;
		string s;
		cin >> ind >> s;
		arr[ind].emplace_back(i, s);
	}
	int prev = 0;
	for (int i = 0; i < m; i++) {
		int cur;
		cin >> cur;
		vector<pair<int, string>> c;
		for (int j = prev + 1; j <= cur; j++) {
			c.insert(c.end(), begin(arr[j]), end(arr[j]));
		}
		sort(begin(c), end(c));
		for (auto& [_, s] : c) {
			cout << s << endl;
		}
		prev = cur;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}