// me when I copy a template into a two-month-old submission...
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

// based on https://judge.yosupo.jp/submission/52300
template <bool emptySuffix = false,
		  bool genLcp = false,
		  bool genSuffixes = false>
struct SuffixArray {
	int n;
	string s;
	vector<int> arr, lcp;
	vector<string_view> suffixes;

	SuffixArray(const string& s) : SuffixArray(s, conv) {}

	template <typename Conv>
	SuffixArray(const string& s, const Conv& conv)
		: n(sz(s)), s(s), arr{}, lcp{}, suffixes{} {
		vector<int> tmp(n + 1);
		for (int i = 0; i < n; i++) {
			tmp[i] = conv(s[i]) + 1;
		}
		arr = SA_IS(tmp, *max_element(begin(tmp), end(tmp)) + 2);
		if (!emptySuffix) {
			arr.erase(arr.begin());
		}
		if (genLcp) {
			genLcpArray();
		}
		if (genSuffixes) {
			genSuffixArray();
		}
	}

	static int conv(char c) {
		if (islower(c)) {
			return c - 'a';
		} else if (isupper(c)) {
			return 26 + c - 'A';
		} else if (isdigit(c)) {
			return 52 + c - '0';
		}
		return 62 + c;
	}

	static void induced_sort(const vector<int>& vec,
							 int val_range,
							 vector<int>& SA,
							 const vector<bool>& sl,
							 const vector<int>& lms_idx) {
		vector<int> l(val_range, 0), r(val_range, 0);
		for (int c : vec) {
			if (c + 1 < val_range)
				++l[c + 1];
			++r[c];
		}
		partial_sum(l.begin(), l.end(), l.begin());
		partial_sum(r.begin(), r.end(), r.begin());
		fill(SA.begin(), SA.end(), -1);
		for (int i = (int)lms_idx.size() - 1; i >= 0; --i)
			SA[--r[vec[lms_idx[i]]]] = lms_idx[i];
		for (int i : SA)
			if (i >= 1 && sl[i - 1])
				SA[l[vec[i - 1]]++] = i - 1;
		fill(r.begin(), r.end(), 0);
		for (int c : vec)
			++r[c];
		partial_sum(r.begin(), r.end(), r.begin());
		for (int k = (int)SA.size() - 1, i = SA[k]; k >= 1; --k, i = SA[k])
			if (i >= 1 && !sl[i - 1]) {
				SA[--r[vec[i - 1]]] = i - 1;
			}
	}

	static vector<int> SA_IS(const vector<int>& vec, int val_range) {
		const int n = vec.size();
		vector<int> SA(n), lms_idx;
		vector<bool> sl(n);
		sl[n - 1] = false;
		for (int i = n - 2; i >= 0; --i) {
			sl[i] =
				(vec[i] > vec[i + 1] || (vec[i] == vec[i + 1] && sl[i + 1]));
			if (sl[i] && !sl[i + 1])
				lms_idx.push_back(i + 1);
		}
		reverse(lms_idx.begin(), lms_idx.end());
		induced_sort(vec, val_range, SA, sl, lms_idx);
		vector<int> new_lms_idx(lms_idx.size()), lms_vec(lms_idx.size());
		for (int i = 0, k = 0; i < n; ++i)
			if (!sl[SA[i]] && SA[i] >= 1 && sl[SA[i] - 1]) {
				new_lms_idx[k++] = SA[i];
			}
		int cur = 0;
		SA[n - 1] = cur;
		for (size_t k = 1; k < new_lms_idx.size(); ++k) {
			int i = new_lms_idx[k - 1], j = new_lms_idx[k];
			if (vec[i] != vec[j]) {
				SA[j] = ++cur;
				continue;
			}
			bool flag = false;
			for (int a = i + 1, b = j + 1;; ++a, ++b) {
				if (vec[a] != vec[b]) {
					flag = true;
					break;
				}
				if ((!sl[a] && sl[a - 1]) || (!sl[b] && sl[b - 1])) {
					flag = !((!sl[a] && sl[a - 1]) && (!sl[b] && sl[b - 1]));
					break;
				}
			}
			SA[j] = (flag ? ++cur : cur);
		}
		for (size_t i = 0; i < lms_idx.size(); ++i)
			lms_vec[i] = SA[lms_idx[i]];
		if (cur + 1 < (int)lms_idx.size()) {
			auto lms_SA = SA_IS(lms_vec, cur + 1);
			for (size_t i = 0; i < lms_idx.size(); ++i) {
				new_lms_idx[i] = lms_idx[lms_SA[i]];
			}
		}
		induced_sort(vec, val_range, SA, sl, new_lms_idx);
		return SA;
	}

	// Author: https://xxccd.glitch.me/blog/entry/12796?#comment-175287
	// Uses kasai's algorithm linear in time and space
	void genLcpArray() {
		int n = sz(arr), k = 0;
		lcp.resize(n);
		vector<int> rank(n + 1);
		for (int i = 0; i < n; i++)
			rank[arr[i]] = i;
		for (int i = 0; i < n; i++, k ? k-- : 0) {
			if (rank[i] == n - 1) {
				k = 0;
				continue;
			}
			int j = arr[rank[i] + 1];
			while (i + k < n && j + k < n && s[i + k] == s[j + k])
				k++;
			lcp[rank[i]] = k;
		}
		lcp[n - 1] = 0;
	}

	void genSuffixArray() {
		suffixes.resize(sz(arr));
		string_view x(s);
		for (int i = 0; i < sz(arr); i++) {
			suffixes[i] = x.substr(arr[i]);
		}
	}
};

struct Smallest {
	static constexpr long null = LONG_MAX;
	long a = null, b = null;

	void add(long x) {
		if (x < a) {
			b = a;
			a = x;
		} else if (x < b) {
			b = x;
		}
	}

	void merge(const Smallest& s) {
		add(s.a);
		add(s.b);
	}

	long get() {
		if (a == null || b == null) {
			return LONG_MIN;
		}
		return a * b;
	}
};

struct DS {
	long sz = 0, cnt = 0, best = LONG_MIN;
	Smallest pos, neg;

	void maintain() {
		cnt = sz * (sz - 1) / 2;
		best = max(pos.get(), neg.get());
	}

	void add(int x) {
		pos.add(-x);
		neg.add(x);
		sz++;
		maintain();
	}

	void merge(const DS& ds) {
		sz += ds.sz;
		pos.merge(ds.pos);
		neg.merge(ds.neg);
		maintain();
	}
};

struct DSU {
	static constexpr DS null{};

	vector<int> p;
	vector<DS> rep;
	long cnt, best;

	DSU(int n) : p(n, -1), rep(n, null), cnt(0), best(LONG_MIN) {}

	int find(int u) { return p[u] < 0 ? u : (p[u] = find(p[u])); }

	void merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return;
		}
		if (p[u] < p[v]) {
			swap(u, v);
		}
		cnt -= rep[u].cnt;
		cnt -= rep[v].cnt;
		p[v] += p[u];
		p[u] = v;
		rep[v].merge(rep[u]);
		rep[u] = null;
		cnt += rep[v].cnt;
		best = max(best, rep[v].best);
	}
};

void solve() {
	int n;
	string s;
	cin >> n >> s;
	long arr[n];
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	SuffixArray<false, true> sa(s);
	vector<int> change[n];
	for (int i = 0; i < n - 1; i++) {
		change[sa.lcp[i]].push_back(i);
	}
	DSU dsu(n);
	for (int i = 0; i < n; i++) {
		dsu.rep[i].add(arr[sa.arr[i]]);
	}
	long ans[n][2];
	for (int i = n - 1; i >= 0; i--) {
		for (auto& a : change[i]) {
			dsu.merge(a, a + 1);
		}
		ans[i][0] = dsu.cnt;
		ans[i][1] = dsu.best;
	}
	for (int i = 0; i < n; i++) {
		if (ans[i][0]) {
			cout << ans[i][0] << " " << ans[i][1] << endl;
		} else {
			cout << 0 << " " << 0 << endl;
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}