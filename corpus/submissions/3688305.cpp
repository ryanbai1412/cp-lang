#include "bits/extc++.h"

using namespace std;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) static_cast<int>((x).size())

struct ST {
	int n;
	vector<int> v;

	ST(int n): n(n), v(4 * n) {}

	void update(int o, int l, int r, int ind, int x) {
		if(l == r) {
			v[o] += x;
			return;
		}
		int mid = (l + r) / 2;
		if(ind <= mid) {
			update(o * 2, l, mid, ind, x);
		}else {
			update(o * 2 + 1, mid + 1, r, ind, x);
		}
		v[o] = v[o * 2] + v[o * 2 + 1];
	}

	inline void update(int ind, int x) {
		update(1, 0, n - 1, ind, x);
	}

	int query(int o, int l, int r, int ql, int qr) {
		if(ql <= l && r <= qr) {
			return v[o];
		}
		int mid = (l + r) / 2;
		int ans = 0;
		if(ql <= mid) {
			ans += query(o * 2, l, mid, ql, qr);
		}
		if(mid < qr) {
			ans += query(o * 2 + 1, mid + 1, r, ql, qr);
		}
		return ans;
	}

	inline int query(int l, int r) {
		if(l > r) {
			return 0;
		}
		return query(1, 0, n - 1, l, r);
	}
};

const int bsize = 400;

void solve() {
	int n, k, q;
	cin >> n >> k >> q;
	int arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
		arr[i]--;
	}

	long qans[q * 2] {};
	int addq[q], subq[q];
	map<pair<int, int>, int> prev;
	vector<pair<int, int>> queries[k];
	{
		int perm[k];
		iota(perm, perm + k, 0);
		int qind = 0;
		for(int i = 0; i < q; i++) {
			int ind;
			cin >> ind;
			ind--;
			int u = perm[ind], v = perm[ind + 1];
			addq[i] = qind;
			queries[u].emplace_back(v, qind++);
			subq[i] = qind;
			queries[v].emplace_back(u, qind++);
			swap(perm[ind], perm[ind + 1]);
		}
	}

	bool check[k] {};
	for(int i = 0; i < k; i++) {
		if(sz(queries[i]) <= bsize) {
			check[i] = true;
		}else {
			int mqind[k];
			memset(mqind, -1, sizeof(mqind));
			for(auto &a: queries[i]) {
				mqind[a.first] = a.second;
			}
			assert(mqind[i] == -1);
			int cnt = 0;
			for(int j = 0; j < n; j++) {
				if(arr[j] == i) {
					cnt++;
				}else if(mqind[arr[j]] != -1) {
					qans[mqind[arr[j]]] += cnt;
				}
			}
			for(auto &a: queries[i]) {
				if(mqind[a.first] != a.second) {
					qans[a.second] = qans[mqind[a.first]];
				}
			}
		}
	}

	int cnt[k] {};
	for(int i = n - 1; i >= 0; i--) {
		if(check[arr[i]]) {
			for(auto &a: queries[arr[i]]) {
				qans[a.second] += cnt[a.first];
			}
		}
		cnt[arr[i]]++;
	}

	ST st(k);
	long ans = 0;
	for(int i = n - 1; i >= 0; i--) {
		ans += st.query(0, arr[i] - 1);
		st.update(arr[i], 1);
	}
	for(auto &a: qans) {
		dbg(a);
	}
	for(int i = 0; i < q; i++) {
		ans += qans[addq[i]];
		ans -= qans[subq[i]];
		cout << ans << endl;
	}
}

int main() {
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}