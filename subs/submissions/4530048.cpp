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

bool check(const bool* v) {
	bool picked[2 * n] {};
	for (int i = 0; i < n; i++) {
		auto& [x, y] = arr[i];
		if (v[i]) {
			picked[y] = true;
		} else {
			picked[x] = true;
		}
	}
	int cur = 0;
	for (int i = 0; i < 2 * n; i++) {
		if (picked[i]) {
			if (s[i] == '(') {
				cur++;
			} else {
				cur--;
			}
		}
		if (cur < 0) {
			return false;
		}
	}
	if (cur) {
		return false;
	}
	for (int i = 0; i < n; i++) {
		cout << v[i] << " \n"[i == n - 1];
	}
	//	cout << "YES" << endl;
	return true;
}

namespace s1 {

void solve() {
	for (int i = 0; i < (1 << n); i++) {
		bool cur[n];
		for (int j = 0; j < n; j++) {
			cur[j] = (i >> j) & 1;
		}
		if (check(cur)) {
			return;
		}
	}
	cout << -1 << endl;
}

}  // namespace s1

namespace s2 {

void solve() {
	bool cur[n];
	for (int i = 0; i < n; i++) {
		cur[i] = s[arr[i].first] == ')';
	}
	if (!check(cur)) {
		cout << -1 << endl;
	}
}

}  // namespace s2

namespace s3 {

void solve() {
	bool cur[n] {};
	vector<int> free;
	int tot = 0;
	for (int i = 0; i < n; i++) {
		if (s[arr[i].first] != s[arr[i].second]) {
			free.push_back(i);
		} else {
			if (s[arr[i].first] == ')') {
				tot++;
			} else {
				tot--;
			}
		}
	}
	int m = sz(free), mid = (tot + m) / 2;
	if (mid < 0 || mid > m) {
		cout << -1 << endl;
		return;
	}
	for (int i = 0; i < mid; i++) {
		int j = free[i];
		if (s[arr[j].second] == '(') {
			cur[j] = true;
		}
	}
	for (int i = mid; i < m; i++) {
		int j = free[i];
		if (s[arr[j].second] == ')') {
			cur[j] = true;
		}
	}
	if (!check(cur)) {
		cout << -1 << endl;
	}
}

}  // namespace s3

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
			b &= x + 1 == y;
		}
		if (b) {
			s3::solve();
			return;
		}
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