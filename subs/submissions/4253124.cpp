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

namespace IO {
const int BUFFER_SIZE = 1 << 16;

char input_buffer[BUFFER_SIZE];
size_t input_pos = 0, input_len = 0;

char output_buffer[BUFFER_SIZE];
int output_pos = 0;

char number_buffer[100];
uint8_t lookup[100];

void _update_input_buffer() {
	input_len = fread(input_buffer, sizeof(char), BUFFER_SIZE, stdin);
	input_pos = 0;

	if (input_len == 0)
		input_buffer[0] = EOF;
}

inline char next_char(bool advance = true) {
	if (input_pos >= input_len)
		_update_input_buffer();

	return input_buffer[advance ? input_pos++ : input_pos];
}

template <typename T>
inline void read_int(T& number) {
	bool negative = false;
	number = 0;

	while (!isdigit(next_char(false)))
		if (next_char() == '-')
			negative = true;

	do {
		number = 10 * number + (next_char() - '0');
	} while (isdigit(next_char(false)));

	if (negative)
		number = -number;
}

template <typename T, typename... Args>
inline void read_int(T& number, Args&... args) {
	read_int(number);
	read_int(args...);
}

void _flush_output() {
	fwrite(output_buffer, sizeof(char), output_pos, stdout);
	output_pos = 0;
}

inline void write_char(char c) {
	if (output_pos == BUFFER_SIZE)
		_flush_output();

	output_buffer[output_pos++] = c;
}

template <typename T>
inline void write_int(T number, char after = '\0') {
	if (number < 0) {
		write_char('-');
		number = -number;
	}

	int length = 0;

	while (number >= 10) {
		uint8_t lookup_value = lookup[number % 100];
		number /= 100;
		number_buffer[length++] = char((lookup_value & 15) + '0');
		number_buffer[length++] = char((lookup_value >> 4) + '0');
	}

	if (number != 0 || length == 0)
		write_char(char(number + '0'));

	for (int i = length - 1; i >= 0; i--)
		write_char(number_buffer[i]);

	if (after)
		write_char(after);
}

void init() {
	// Make sure _flush_output() is called at the end of the program.
	bool exit_success = atexit(_flush_output) == 0;
	assert(exit_success);

	for (int i = 0; i < 100; i++)
		lookup[i] = uint8_t((i / 10 << 4) + i % 10);
}
}  // namespace IO

template <typename Node_T, typename Operation = plus<Node_T>, bool debug = true>
class SegTree {
  private:
	static constexpr Node_T noT {};

  public:
	const int n;
	const Operation operation;

	vector<Node_T> v, arr;

	explicit SegTree(const vector<Node_T>& _arr, const Operation& _operation = Operation())
		: n(sz(_arr)), operation(_operation), v(4 * n), arr(_arr) {
		if (n) {
			build(1, 0, n - 1);
		}
	}

  private:
	void build(int o, int l, int r) {
		if (l == r) {
			v[o] = arr[l];
		} else {
			int mid = (l + r) / 2;
			build(o * 2, l, mid);
			build(o * 2 + 1, mid + 1, r);
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	template <typename F>
	void set_func(int o, int l, int r, int ind, const F& f) {
		if (l == r) {
			f(arr[l]);
			v[o] = arr[l];
		} else {
			int mid = (l + r) / 2;
			if (ind <= mid) {
				set_func(o * 2, l, mid, ind, f);
			} else {
				set_func(o * 2 + 1, mid + 1, r, ind, f);
			}
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	[[nodiscard]] Node_T query(int o, int l, int r, int ql, int qr) const {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2;
			if (ql <= mid) {
				if (mid < qr) {
					return operation(query(o * 2, l, mid, ql, qr),
									 query(o * 2 + 1, mid + 1, r, ql, qr));
				}
				return query(o * 2, l, mid, ql, qr);
			}
			return query(o * 2 + 1, mid + 1, r, ql, qr);
		}
	}

  public:
	[[nodiscard]] Node_T query(int l, int r) const {
		if (debug) {
			assert(0 <= l);
			assert(l < n);
			assert(0 <= r);
			assert(r < n);
			assert(l <= r);
		}
		return query(1, 0, n - 1, l, r);
	}

	void set(int ind, const Node_T& x) {
		if (debug) {
			assert(0 <= ind);
			assert(ind < n);
		}
		set_func(1, 0, n - 1, ind, [&](Node_T& t) {
			t = x;
		});
	}

	/**
	 * applies the function f to arr[ind]
	 * f should be of signature (Node_T&) -> void
	 */
	template <typename F>
	void set_func(int ind, const F& f) {
		if (debug) {
			assert(0 <= ind);
			assert(ind < n);
		}
		set_func(1, 0, n - 1, ind, f);
	}
};

template <int maxc>
using Node = array<pair<int, int>, maxc>;

template <int maxc>
Node<maxc> nfrom(int x) {
	Node<maxc> ans {};
	ans[0] = {x, 1};
	return ans;
}

template <int maxc>
struct Op {
	Node<maxc> operator()(const Node<maxc>& a, const Node<maxc>& b) const {
		static pair<int, int> tmp[2 * maxc];
		int n = 0, j;
		for (j = 0; j < maxc && a[j].second; j++)
			;
		copy(a.begin(), a.begin() + j, tmp);
		n += j;
		for (j = 0; j < maxc && b[j].second; j++)
			;
		copy(b.begin(), b.begin() + j, tmp + n);
		n += j;

		sort(tmp, tmp + n, [&](const auto& a, const auto& b) -> bool {
			return a.first < b.first;
		});
		for (int i = 0; i + 1 < n; i++) {
			if (tmp[i].first == tmp[i + 1].first) {
				tmp[i].second += tmp[i + 1].second;
				tmp[i + 1].second = 0;
				i++;
			}
		}
		sort(tmp, tmp + n, [&](const auto& a, const auto& b) -> bool {
			return a.second > b.second;
		});
		if (maxc < n && tmp[maxc].second) {
			for (int i = 0; i < maxc; i++) {
				tmp[i].second -= tmp[maxc].second;
			}
		} else {
			for (; n && !tmp[n - 1].second; n--)
				;
		}
		Node<maxc> ans;
		copy(tmp, tmp + min(n, maxc), ans.begin());
		fill(ans.begin() + min(n, maxc), ans.end(), pair<int, int> {0, 0});
		return ans;
	}
};

void solve() {
	int n, m;
	IO::read_int(n, m);
	int arr[n];
	for (auto& a : arr) {
		IO::read_int(a);
	}
	vector<int> vals[m];
	for (int i = 0; i < n; i++) {
		vals[arr[i] - 1].push_back(i);
	}
	auto cnt = [&](int x, int l, int r) -> int {
		x--;
		return int(upper_bound(begin(vals[x]), end(vals[x]), r) -
				   lower_bound(begin(vals[x]), end(vals[x]), l));
	};
	vector<Node<3>> v;
	for (auto& a : arr) {
		v.push_back(nfrom<3>(a));
	}
	SegTree<Node<3>, Op<3>> st(v);
	int q;
	IO::read_int(q);
	while (q--) {
		int l, r;
		IO::read_int(l, r);
		l--;
		r--;
		int opt1 = 0, opt2 = 0;
		for (auto& [x, c] : st.query(l, r)) {
			dbg(x, c);
			if (c) {
				x = cnt(x, l, r);
				dbg(x);
				if (x > opt1) {
					opt1 = x;
				} else if (x > opt2) {
					opt2 = x;
				}
			}
		}
		dbg(opt1, opt2);
		if (opt1 >= int(ceil(2 * double(r - l + 1) / 3)) || opt2 >= (r - l + 1 + 2) / 3) {
			IO::write_char('Y');
			IO::write_char('E');
			IO::write_char('S');
		} else {
			IO::write_char('N');
			IO::write_char('O');
		}
		IO::write_char('\n');
	}
}

int main() {
	IO::init();
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