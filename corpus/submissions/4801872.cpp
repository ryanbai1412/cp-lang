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

void hrot(vector<vector<int>>& arr, int ind, int k) {
	rotate(begin(arr[ind]), arr[ind].end() - k, end(arr[ind]));
}

void vrot(vector<vector<int>>& arr, int ind, int k) {
	int tmp[sz(arr)];
	for (int i = 0; i < sz(arr); i++) {
		tmp[i] = arr[i][ind];
	}
	rotate(tmp, tmp + sz(arr) - k, tmp + sz(arr));
	for (int i = 0; i < sz(arr); i++) {
		arr[i][ind] = tmp[i];
	}
}

namespace s1 {

const int maxn = 1 << 24;

int hsh(const vector<vector<int>>& arr) {
	int ans = 0;
	for (auto& a : arr) {
		for (auto& b : a) {
			ans = ans * 8 + b;
		}
	}
	return ans;
}

void solve(const vector<vector<int>>& arr) {
	int n = sz(arr), m = sz(arr[0]);
	map<int, tuple<bool, int, int>> vis;
	queue<vector<vector<int>>> q;
	auto push = [&](const vector<vector<int>>& u, const tuple<bool, int, int>& from) {
		if (vis.emplace(hsh(u), from).second) {
			q.push(u);
		}
	};
	push(arr, {false, -1, -1});
	while (sz(q)) {
		auto u = q.front();
		q.pop();
		for (int i = 1; i < n; i++) {
			for (int j = 0; j < m; j++) {
				auto tmp = u;
				vrot(tmp, j, i);
				push(tmp, {true, j, i});
			}
		}
		for (int i = 1; i < m; i++) {
			for (int j = 0; j < n; j++) {
				auto tmp = u;
				hrot(tmp, j, i);
				push(tmp, {false, j, i});
			}
		}
	}
	auto tmp = arr;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			tmp[i][j] = i * m + j;
		}
	}
	vector<tuple<bool, int, int>> ans;
	while (tmp != arr) {
		auto [vert, a, b] = vis[hsh(tmp)];
		ans.emplace_back(vert, a, b);
		if (vert) {
			vrot(tmp, a, n - b);
		} else {
			hrot(tmp, a, m - b);
		}
	}
	cout << sz(ans) << endl;
	reverse(begin(ans), end(ans));
	for (auto& [a, b, c] : ans) {
		cout << "12"[a] << " " << b + 1 << " " << c << endl;
	}
}

}  // namespace s1

const long m1 = 1e9 + 7, m2 = 1e9 + 9, base = 141214;

long hsh(const vector<vector<int>> &arr) {
	long a1 = 0, a2 = 0;
	for (auto& a : arr) {
		for (auto& b : a) {
			a1 = (a1 * base + b) % m1;
			a2 = (a2 * base + b) % m2;
		}
	}
	return (a1 << 31) | a2;
}

void solve() {
	int n, m;
	cin >> n >> m;
	vector<vector<int>> arr(n, vector<int>(m));
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	if (n * m <= 8) {
		s1::solve(arr);
		return;
	}
	vector<vector<int>> targ = arr;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			targ[i][j] = i * m + j;
		}
	}
	if (arr == targ) {
		cout << 2 << endl;
		cout << "1 1 1" << endl;
		cout << "1 1 " << m - 1 << endl;
		return;
	}
	int ca[n] {}, cb[m] {};
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (arr[i][j] != targ[i][j]) {
				ca[i]++;
				cb[j]++;
			}
		}
	}
	bool ia[n], ib[m];
	if (min(n, m) <= 10) {
		memset(ia, 1, sizeof(ia));
		memset(ib, 1, sizeof(ib));
	} else {
		for (int i = 0; i < n; i++) {
			ia[i] = ca[i] >= 3;
			if (ia[i]) {
				dbg(i);
			}
		}
		for (int i = 0; i < m; i++) {
			ib[i] = cb[i] >= 3;
			if (ib[i]) {
				dbg(i);
			}
		}
	}
	map<long, tuple<bool, int, int>> vis;
	auto print = [&](const tuple<bool, int, int> &t) -> void {
		auto& [a, b, c] = t;
		cout << "12"[a] << " " << b + 1 << " " << c << endl;
	};
	for (int i = 0; i < n; i++) {
		if (ia[i]) {
			for (int j = 1; j < m; j++) {
				auto tmp = arr;
				hrot(tmp, i, j);
				if (targ == tmp) {
					cout << 1 << endl;
					print({false, i, j});
					return;
				}
				vis.insert({hsh(tmp), {false, i, j}});
			}
		}
	}
	for (int i = 0; i < m; i++) {
		if (ib[i]) {
			for (int j = 1; j < n; j++) {
				auto tmp = arr;
				vrot(tmp, i, j);
				if (targ == tmp) {
					cout << 1 << endl;
					print({true, i, j});
					return;
				}
				vis.insert({hsh(tmp), {true, i, j}});
			}
		}
	}
	auto check = [&](const vector<vector<int>> &x, const tuple<bool, int, int> &y) -> void {
		auto it = vis.find(hsh(x));
		if (it != vis.end()) {
			cout << 2 << endl;
			print(it->second);
			print(y);
			exit(0);
		}
	};
	for (int i = 0; i < n; i++) {
		for (int j = 1; j < m; j++) {
			auto tmp = targ;
			hrot(tmp, i, m - j);
			check(tmp, {false, i, j});
		}
	}
	for (int i = 0; i < m; i++) {
		for (int j = 1; j < n; j++) {
			auto tmp = targ;
			vrot(tmp, i, n - j);
			check(tmp, {true, i, j});
		}
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}