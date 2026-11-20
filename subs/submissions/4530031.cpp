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

int n;
string s;
vector<pair<int, int>> arr;

bool check(bool* picked) {
	int cur = 0;
	for (int j = 0; j < 2 * n; j++) {
		if (picked[j]) {
			if (s[j] == '(') {
				cur++;
			} else {
				cur--;
			}
		}
		if (cur < 0) {
			return false;
		}
	}
	return !cur;
}

namespace s1 {

void solve() {
	for (int i = 0; i < (1 << n); i++) {
		bool picked[2 * n] {};
		for (int j = 0; j < n; j++) {
			if ((i >> j) & 1) {
				picked[arr[j].second] = true;
			} else {
				picked[arr[j].first] = true;
			}
		}
		if (check(picked)) {
			for (int j = 0; j < n; j++) {
				cout << ((i >> j) & 1) << " \n"[j == n - 1];
			}
			return;
		}
	}
	cout << -1 << endl;
}

}  // namespace s1

namespace s2 {

void solve() {
	bool picked[2 * n] {};
	for (auto& [x, y] : arr) {
		picked[min(x, y)] = true;
	}
	if (check(picked)) {
		for (int i = 0; i < n; i++) {
			cout << (arr[i].second > arr[i].first) << " \n"[i == n - 1];
		}
	} else {
		cout << -1 << endl;
	}
}

}  // namespace s2

void solve() {
	cin >> n >> s;
	arr.resize(n);
	for (auto& [x, y] : arr) {
		cin >> x >> y;
		x--;
		y--;
	}
	{
		bool b = true;
		for (auto& [x, y] : arr) {
			b &= s[x] == s[y];
		}
		if (b) {
			s2::solve();
			return;
		}
	}
	if (n <= 10) {
		s1::solve();
		return;
	}
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