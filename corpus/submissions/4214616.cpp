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
	bool swp = false;
	if (n > m) {
		swap(n, m);
		swp = true;
	}
	string arr[n];
	if (!swp) {
		for (int i = 0; i < n; i++) {
			cin >> arr[i]; 
		}
	} else {
		for (int i = 0; i < n; i++) {
			arr[i] = string(m, '\0');
		}
		for (int i = 0; i < m; i++) {
			string s;
			cin >> s;
			for (int j = 0; j < n; j++) {
				arr[j][i] = s[j];
			}
		}
	}
	int p1, p2;
	for (int i = 0; i < n; i++) {
		if (arr[i].find('U') != string::npos) {
			p1 = i;
		}
		if (arr[i].find('D') != string::npos) {
			p2 = i;
		}
	}
	if (p1 > p2) {
		swap(p1, p2);
	}
	if (n == 1) {
		int ans = 0;
		ans += p1 >= 2 && arr[p1 - 1][0] == '.' && arr[p1 - 2][0] == '.';
		ans += p2 < n - 2 && arr[p1 + 1][0] == '.' && arr[p1 + 2][0] == '.';
		cout << ans << endl;
	} else if (n == 2) {
		int ans = 0;
		for (int i = 0; i < n; i++) {
			ans += arr[i] == string("..");
		}
		cout << ans << endl;
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