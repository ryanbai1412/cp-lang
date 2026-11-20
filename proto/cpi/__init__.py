"""cpi: the reference interpreter for library code (stdlib/ and local libraries).

The compiler in ../cpc turns a single solution file into C++. This package
runs the part of the language that libraries are written in (modules,
generic structs, operation values, laws, generators, projections) directly,
so that `@test fn`s and laws in a library can be run today.
"""
