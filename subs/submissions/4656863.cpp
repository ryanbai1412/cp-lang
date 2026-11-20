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

void solve() {
	int n, m;
	cin >> n >> m;
	array<int, 3> arr[m];
	for (int i = 0; i < m; i++) {
		auto& [l, r, ci] = arr[i];
		cin >> l >> r;
		l--;
		r--;
		ci = i;
		if (l > r) {
			r += n;
		}
	}
	sort(arr, arr + m, [&](const auto& a, const auto& b) -> bool {
		if (a[0] != b[0]) {
			return a[0] < b[0];
		}
		return a[1] > b[1];
	});
	for (auto& [l, r, _] : arr) {
		if (r >= n) {
			r -= n;
		}
	}
	int par[m];
	pair<int, int> mx {-1, -1};
	for (int i = 0; i < 2 * m; i++) {
		int ci = i >= m ? i - m : i;
		auto [l, r, _] = arr[ci];
		if (l > r) {
			r += n;
		}
		if (i >= m) {
			l += n;
			r += n;
		}
		if (r <= mx.first) {
			par[ci] = mx.second;
		} else {
			par[ci] = ci;
			mx = {r, ci};
		}
	}
	for (int i = 0; i < m; i++) {
		int u = i;
		while (par[u] != u) {
			u = par[u];
		}
		par[i] = u;
	}
	vector<int> vals;
	int inds[m];
	for (int i = 0; i < m; i++) {
		if (par[i] == i) {
			inds[i] = sz(vals);
			vals.push_back(i);
		}
		dbg(i, arr[i][0], arr[i][1], par[i]);
	}
	auto covers_all = [&](const vector<pair<int, int>>& arr) -> bool {
		vector<int> e[n];
		for (auto& [l, r] : arr) {
			e[l].push_back(r);
		}
		int mx = -1;
		for (int i = 0; i < n; i++) {
			for (auto& a : e[i]) {
				mx = max(mx, a);
			}
			if (mx < i) {
				return false;
			}
		}
		return true;
	};
	auto check = [&](const vector<bool>& distr) -> void {
		vector<pair<int, int>> segs[2];
		auto add = [&](int i, int l, int r) -> void {
			if (r >= n) {
				r -= n;
			}
			if (l > r) {
				segs[i].emplace_back(l, n - 1);
				segs[i].emplace_back(0, r);
			} else {
				segs[i].emplace_back(l, r);
			}
		};
		bool ans[m];
		for (int i = 0; i < m; i++) {
			auto& [l, r, ci] = arr[i];
			if (par[i] == i) {
				ans[ci] = distr[inds[i]];
			} else {
				ans[ci] = !distr[inds[par[i]]];
			}
			dbg(i, inds[par[i]], ans[ci]);
			add(ans[ci], l, r);
		}
		if (covers_all(segs[0]) && covers_all(segs[1])) {
			for (auto& a : ans) {
				cout << a;
			}
			cout << endl;
			exit(0);
		}
	};
	int k = sz(vals);
	dbg(k);
	if (k % 2 == 0 || k == 1) {
		vector<bool> distr(k);
		for (int i = 0; i < k; i++) {
			distr[i] = i & 1;
		}
		check(distr);
		cout << "impossible" << endl;
		return;
	} else if (k <= 3) {
		for (int i = 0; i < k; i++) {
			vector<bool> distr(k);
			int j = i;
			bool cur = true;
			while (true) {
				distr[j] = cur;
				cur ^= true;
				j = j + 1 == k ? 0 : j + 1;
				if (j == i) {
					break;
				}
			}
			check(distr);
		}
		cout << "impossible" << endl;
		return;
	}
	{
		vector<pair<int, int>> cur;
		for (auto& [l, r, _] : arr) {
			if (l > r) {
				cur.emplace_back(l, n - 1);
				cur.emplace_back(0, r);
			} else {
				cur.emplace_back(l, r);
			}
		}
		if (!covers_all(cur)) {
			cout << "impossible" << endl;
			return;
		}
	}
	auto covers = [&](const vector<pair<int, int>>& segs, int ql, int qr) -> bool {
		if (!sz(segs)) {
			return false;
		}
		vector<pair<int, int>> evts;
		for (auto& [l, r] : segs) {
			evts.emplace_back(l - n, 1);
			evts.emplace_back(r - n + 1, -1);
			evts.emplace_back(l, 1);
			evts.emplace_back(r + 1, -1);
			evts.emplace_back(l + n, 1);
			evts.emplace_back(r + n + 1, -1);
		}
		sort(begin(evts), end(evts));
		int prev = -1, cur = 0;
		for (auto& [ind, x] : evts) {
			if (prev != -1 && ind != prev) {
				if (!cur) {
					if (ql <= ind - 1 && ind - 1 <= qr) {
						return false;
					} else if (ql <= prev && prev <= qr) {
						return false;
					}
				}
			}
			cur += x;
			prev = ind;
		}
		return true;
	};
	bool can[k];
	vector<pair<int, int>> child[k];
	for (int i = 0; i < m; i++) {
		auto& [l, r, _] = arr[i];
		if (l > r) {
			r += n;
		}
		if (par[i] != i) {
			child[inds[par[i]]].emplace_back(l, r);
		}
	}
	array<int, 2> carr[k];
	for (int i = 0; i < k; i++) {
		carr[i] = {arr[vals[i]][0], arr[vals[i]][1]};
	}
	for (int i = 0; i < k; i++) {
		int a = (i + k - 1) % k, b = (i + 1) % k;
		vector<pair<int, int>> cur = child[i];
		cur.emplace_back(carr[a][0], carr[a][1]);
		cur.emplace_back(carr[b][0], carr[b][1]);
		can[i] = covers(cur, carr[i][0], carr[i][1]);
		dbg(i, can[i], carr[a][0], carr[a][1], carr[b][0], carr[b][1], carr[i][0], carr[i][1]);
	}
	int cnt = 0;
	for (auto& a : can) {
		cnt += a;
	}
	for (int i = 0; i < k; i++) {
		int a = i, b = (i + 1) % k, c = (i + 2) % k, d = (i + 3) % k;
		int ccnt = cnt - can[b] - can[c];
		if (ccnt != k - 2) {
			continue;
		}
		vector<pair<int, int>> cur;
		cur.insert(cur.end(), begin(child[b]), end(child[b]));
		cur.insert(cur.end(), begin(child[c]), end(child[c]));
		cur.emplace_back(carr[a][0], carr[a][1]);
		cur.emplace_back(carr[d][0], carr[d][1]);
		if (!covers(cur, carr[b][0], carr[c][1])) {
			continue;
		}
		dbg(i, carr[b][0], carr[c][1]);
		vector<bool> distr(k);
		bool cb = true;
		while (true) {
			distr[c] = cb;
			cb ^= true;
			if (c == b) {
				break;
			}
			c = c + 1 == k ? 0 : c + 1;
		}
		check(distr);
//		assert(false);
	}
	cout << "impossible" << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}