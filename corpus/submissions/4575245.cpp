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

struct Solver {
	vector<int> arr;
	vector<pair<int, int>> ans;

	Solver(const vector<int>& arr) : arr(arr) {}

	void swp(int i, int j) {
		if (i == j) {
			return;
		}
		swap(arr[i], arr[j]);
		ans.emplace_back(i, j);
	}

	void reverse(int l, int r) {
		while (l < r) {
			swp(l++, --r);
		}
	}

	void merge(int l, int mid, int r) {
		if (l == mid || mid == r) {
			return;
		}
		auto cur = vector<int> (arr.begin() + l, arr.begin() + r);
		sort(begin(cur), end(cur));
		int med = cur[sz(cur) / 2];
		int m1 = lower_bound(arr.begin() + l, arr.begin() + mid, med) - arr.begin();
		int m2 = lower_bound(arr.begin() + mid, arr.begin() + r, med) - arr.begin();
		reverse(m1, mid);
		reverse(mid, m2);
		reverse(m1, m2);
		int len1 = m1 - l, len2 = mid - m1, len3 = m2 - mid, len4 = r - m2;
		merge(l, l + len1, l + len1 + len3);
		merge(r - len4 - len2, r - len4, r);
	}

	void solve(int l, int r) {
		if (r - l == 1) {
			return;
		}
		int mid = (l + r) / 2;
		solve(l, mid);
		solve(mid, r);
		merge(l, mid, r);
	}

	vector<pair<int, int>> solve() {
		solve(0, sz(arr));
		return ans;
	}
};

void solve() {
	int n;
	cin >> n;
	vector<int> arra(n), arrb(n);
	for (auto& a : arra) {
		cin >> a;
		a--;
	}
	for (auto& a : arrb) {
		cin >> a;
		a--;
	}
	Solver a(arra), b(arrb);
	auto bx = b.solve();
	auto ax = a.solve();
	reverse(begin(bx), end(bx));
	dbg(sz(ax), sz(bx));
	cout << sz(ax) + sz(bx) << endl;
	for (auto& [x, y] : ax) {
		cout << x + 1 << " " << y + 1 << endl;
	}
	for (auto& [x, y] : bx) {
		cout << x + 1 << " " << y + 1 << endl;
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}