#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

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

struct cint {
	long real, omega;

	constexpr cint() : cint(0) {}
	constexpr cint(int x) : cint(long(x)) {}
	constexpr cint(long x) : cint(x, 0) {}
	constexpr cint(long _real, long _omega) : real(_real), omega(_omega) {}

	constexpr bool operator==(const cint& x) const {
		return real == x.real && omega == x.omega;
	}

	constexpr cint& operator+=(const cint& x) {
		real += x.real;
		omega += x.omega;
		return *this;
	}

	constexpr cint& operator-=(const cint& x) {
		real -= x.real;
		omega -= x.omega;
		return *this;
	}

	constexpr cint& operator*=(const cint& x) {
		long nreal = real * x.real - omega * x.omega;
		long nomega = real * x.omega + omega * x.real - omega * x.omega;
		real = nreal;
		omega = nomega;
		return *this;
	}

	constexpr cint& operator/=(const cint& x) {
		if (x.omega == 0) {
			real /= x.real;
			omega /= x.real;
		} else {
			long nreal = (2 * omega - real) / 3;
			long nomega = (omega - 2 * real) / 3;
			real = nreal;
			omega = nomega;
		}
		return *this;
	}

	constexpr friend cint operator+(cint a, const cint& b) {
		return a += b;
	}

	constexpr friend cint operator-(cint a, const cint& b) {
		return a -= b;
	}

	constexpr friend cint operator*(cint a, const cint& b) {
		return a *= b;
	}

	constexpr friend cint operator/(cint a, const cint& b) {
		return a /= b;
	}
};

constexpr cint omega(0, 1), omegasq = omega * omega, osqsm = omega + omegasq,
							osqdi = omega - omegasq, const1 = 2 - osqsm,
							const2 = 1 - omegasq;

void conv(int n, cint* x, cint* out) {
	static cint buf[531441 * 12 * 2];
	static cint* ptr = buf;
	if (n == 1) {
		out[0] = x[0] * x[0];
		return;
	}
	int nn = n / 3;
	cint *ax = ptr;
	cint *bx = ptr += nn;
	cint *cx = ptr += nn;
	cint *a = ptr += nn;
	cint *b = ptr += nn;
	cint *c = ptr += nn;
	ptr += nn;
	for (int i = 0; i < nn; i++) {
		ax[i] = x[i] + x[i + nn] + x[i + nn + nn];
	}
	conv(nn, ax, a);
	for (int i = 0; i < nn; i++) {
		bx[i] = x[i] + x[i + nn] * omega + x[i + nn + nn] * omegasq;
	}
	conv(nn, bx, b);
	for (int i = 0; i < nn; i++) {
		cx[i] = x[i] + x[i + nn] * omegasq + x[i + nn + nn] * omega;
	}
	conv(nn, cx, c);
	for (int i = 0; i < nn; i++) {
		cint e = ((b[i] + c[i]) - osqsm * a[i]) / const1;
		cint f = (b[i] - omegasq * a[i] - const2 * e) / osqdi;
		cint g = (c[i] - omegasq * a[i] - const2 * e) / osqdi;
		out[i] = e;
		out[i + nn] = f;
		out[i + nn + nn] = g;
	}
	ptr -= nn * 6;
}

void solve() {
	int n, k;
	IO::read_int(n, k);
	int m = 1;
	for (int i = 0; i < k; i++) {
		m *= 3;
	}
	vector<int> arr(n);
	for (auto& a : arr) {
		IO::next_char();
		IO::next_char();
		int cur = 0;
		for (int i = 0; i < k; i++) {
			char c = IO::next_char();
			cur = cur * 3 + (c - '1');
		}
		a = cur;
	}
	vector<cint> v(m);
	for (auto& a : arr) {
		v[a] += 1;
	}
	vector<cint> convd(m);
	conv(m, v.data(), convd.data());
	long ans = 0;
	for (auto& a : arr) {
		int cur = 0, cpow = 1;
		while (a) {
			cur += (3 - a % 3) % 3 * cpow;
			cpow *= 3;
			a /= 3;
		}
		ans += convd[cur].real - 1;
	}
	IO::write_int(ans / 6, '\n');
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	IO::init();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}