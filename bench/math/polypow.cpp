// Matched C++20 reference for bench/math/polypow.cp: ecnerwala's ps_pow_monic(c, num(-2 * (n - 1))) on the same data.
#include "ecnerwala_series.h"

int main() {
	using num = modnum<998244353>;
	using E = ecnerwala::fft::engines::ntt<num>;
	using ps = ecnerwala::series::trunc<E>;
	const long long n = 100001, reps = 5;
	long long s = 88172645;
	auto next = [&]() { s = (s * 1103515245 + 12345) % 2147483648LL; return num(s % 998244353); };
	ps f(n), g(n), c(n);
	for (auto& x : f) x = next();
	for (auto& x : g) x = next();
	for (auto& x : c) x = next();
	g[0] = 0; c[0] = 1;
	num acc = 0;
	for (long long r = 0; r < reps; r++) {
		f[0] += acc; c[1] += acc;
		ps x = ps(ps_pow_monic(c, num(-2 * (n - 1))));
		acc += x[r % n];
	}
	std::cout << acc << '\n';
}
