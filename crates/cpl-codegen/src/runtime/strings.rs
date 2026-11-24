//! Strings: conversion, borrowed views and cutting.

use super::Piece;

pub(super) const PIECES: &[Piece] = &[
    Piece {
        name: "str_conv",
        needs: &[],
        code: "\
inline std::vector<u8> str_to_list(const std::string& s) { return std::vector<u8>(s.begin(), s.end()); }
inline std::string list_to_str(const std::vector<u8>& v) { return std::string(v.begin(), v.end()); }
inline ll parse_int(const std::string& s) { return std::stoll(s); }
",
    },
    Piece {
        name: "str_view",
        needs: &["fail"],
        code: "\
// O(1) read-only slice/reverse view into a `Str` (`std::string_view` + reverse bit).
struct StrView {
    using value_type = u8;
    std::string_view sv;
    bool rev = false;
    ll k0 = 0;
    StrView() = default;
    StrView(const std::string& s, bool r = false) noexcept : sv(s), rev(r) {}
    StrView(std::string_view v, bool r = false, ll k = 0) noexcept : sv(v), rev(r), k0(k) {}
    void cpl_str_view_tag() const {}
    size_t size() const noexcept { return sv.size(); }
    bool empty() const noexcept { return sv.empty(); }
    u8 operator[](size_t i) const noexcept { return (u8)(rev ? sv[sv.size() - 1 - i] : sv[i]); }
    u8 front() const noexcept { return (*this)[0]; }
    u8 back() const noexcept { return (*this)[sv.size() - 1]; }
    StrView slice(ll lo, ll hi) const {
        if (lo < 0 || hi < lo || (size_t)hi > sv.size()) fail(\"slice outside the string\");
        size_t n = (size_t)(hi - lo), off = rev ? (sv.size() - (size_t)hi) : (size_t)lo;
        return StrView(sv.substr(off, n), rev, k0 + (ll)off);
    }
    StrView reversed() const noexcept { return StrView(sv, !rev, k0); }
    std::string to_str() const { return !rev ? std::string(sv) : std::string(sv.rbegin(), sv.rend()); }
    struct Iter {
        const char* d; size_t i, n; bool rev;
        u8 operator*() const noexcept { return (u8)(rev ? d[n - 1 - i] : d[i]); }
        Iter& operator++() noexcept { ++i; return *this; }
        bool operator!=(const Iter& o) const noexcept { return i != o.i; }
        bool operator==(const Iter& o) const noexcept { return i == o.i; }
    };
    Iter begin() const noexcept { return {sv.data(), 0, sv.size(), rev}; }
    Iter end() const noexcept { return {sv.data(), sv.size(), sv.size(), rev}; }
    Iter rbegin() const noexcept { return reversed().begin(); }
    Iter rend() const noexcept { return reversed().end(); }
    int cmp(StrView o) const noexcept {
        if (!rev && !o.rev) return sv.compare(o.sv);
        size_t n = sv.size() < o.sv.size() ? sv.size() : o.sv.size();
        for (size_t i = 0; i < n; i++) { u8 a = (*this)[i], b = o[i]; if (a != b) return a < b ? -1 : 1; }
        return sv.size() < o.sv.size() ? -1 : (sv.size() > o.sv.size() ? 1 : 0);
    }
    bool contains_char(u8 c) const noexcept { return sv.find((char)c) != std::string_view::npos; }
    bool contains_str(StrView p) const { return p.empty() || ((!rev && !p.rev) ? sv.find(p.sv) != std::string_view::npos : to_str().find(p.to_str()) != std::string::npos); }
    friend bool operator==(StrView a, StrView b) noexcept { return a.size() == b.size() && a.cmp(b) == 0; }
    friend bool operator!=(StrView a, StrView b) noexcept { return !(a == b); }
    friend bool operator<(StrView a, StrView b) noexcept { return a.cmp(b) < 0; }
    friend bool operator<=(StrView a, StrView b) noexcept { return a.cmp(b) <= 0; }
    friend bool operator>(StrView a, StrView b) noexcept { return a.cmp(b) > 0; }
    friend bool operator>=(StrView a, StrView b) noexcept { return a.cmp(b) >= 0; }
};
inline StrView str_slice(StrView s, ll lo, ll hi) { return s.slice(lo, hi); }
inline StrView str_rev(StrView s) noexcept { return s.reversed(); }
inline ll key_lo(StrView s) noexcept { return s.k0; }
inline ll key_pos(StrView s, ll k) { ll i = k - s.k0, n = (ll)s.size(); if (i < 0 || i >= n) key_fail(k, s.k0, n); return s.rev ? n - 1 - i : i; }
inline ll pos_key(StrView s, ll p) noexcept { return s.k0 + (s.rev ? (ll)s.size() - 1 - p : p); }
inline StrView key_slice(StrView s, ll lo, ll hi) { if (lo < s.k0 || hi < lo || hi > s.k0 + (ll)s.size()) fail(\"slice outside the string's keys\"); return StrView(s.sv.substr((size_t)(lo - s.k0), (size_t)(hi - lo)), s.rev, lo); }
inline std::string to_str(StrView s) { return s.to_str(); }
inline void append_str_view(std::string& o, StrView v) { if (!v.rev) o.append(v.sv.data(), v.sv.size()); else for (size_t i = v.sv.size(); i-- > 0;) o.push_back(v.sv[i]); }
inline std::string str_concat(StrView a, StrView b) { std::string o; o.reserve(a.size() + b.size()); append_str_view(o, a); append_str_view(o, b); return o; }
inline u8 at(StrView s, ll i) { if ((unsigned long long)i >= s.size()) fail(\"index outside the string\"); return s[(size_t)i]; }
inline std::optional<u8> first(StrView s) { return s.empty() ? std::optional<u8>() : std::optional<u8>(s.front()); }
inline std::optional<u8> last(StrView s) { return s.empty() ? std::optional<u8>() : std::optional<u8>(s.back()); }
inline std::vector<u8> str_to_list(StrView s) { std::vector<u8> v; v.reserve(s.size()); for (u8 c : s) v.push_back(c); return v; }
inline ll parse_int(StrView s) { return std::stoll(s.to_str()); }
",
    },
    Piece {
        name: "put_str_view",
        needs: &["out", "str_view"],
        code: "\
inline void put_str(StrView s) { append_str_view(out, s); }
",
    },
    Piece {
        name: "cut_beside",
        needs: &["fail"],
        code: "\
// The cut after (or before) key `k` in walk order (R07.4.14).
inline ll cut_beside(bool rev, ll k0, ll n, ll k, bool after) { if (k < k0 || k >= k0 + n) key_fail(k, k0, n); return after != rev ? k + 1 : k; }
",
    },
];
