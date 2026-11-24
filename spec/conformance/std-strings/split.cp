// expect: ok, output "YES"
use std.strings.{split_once, split_whitespace}
let e: List<Str> = []
print(yes([Str(p) for p in "a,b,,c".split(',')] == ["a", "b", "", "c"] and [Str(p) for p in "a,".split(',')] == ["a", ""] and [Str(p) for p in "".split(',')] == [""] and "key=val=x".split_once('=') == Some(("key", "val=x")) and "abc".split_once('=') == None and " a  b\tc ".split_whitespace() == ["a", "b", "c"] and "   ".split_whitespace() == e))
