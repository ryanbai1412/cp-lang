//! Type definition, implementations, `use` links and pulled diagnostics.

use super::*;

const ROADS: &str = "\
struct Road:
    to: Int
trait Sized:
    fn size(self) -> Int
impl Sized for Road:
    fn size(self) -> Int = self.to
impl Road:
    fn twice(self) -> Int = self.to * 2
fn far(r: Road) -> Int = r.size()
print(far(Road(to: 3)))
";

fn at(server: &mut Server, method: &str, line: u32, character: u32) -> Value {
    let reply = server.handle(&request(
        7,
        method,
        obj([("textDocument", doc()), ("position", pos(line, character))]),
    ));
    reply[0].at(&["result"]).clone()
}

fn starts(locations: &Value) -> Vec<Value> {
    let locations = locations.as_array().unwrap_or(&[]);
    locations
        .iter()
        .map(|l| l.at(&["range", "start"]).clone())
        .collect()
}

#[test]
fn type_definition_jumps_from_a_value_to_its_struct() {
    let (mut server, diagnostics) = open(ROADS);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let def = at(&mut server, "textDocument/typeDefinition", 8, 25);
    assert_eq!(def.at(&["uri"]).as_str(), Some(URI));
    assert_eq!(def.at(&["range", "start"]), &pos(0, 7));
    assert_eq!(def.at(&["range", "end"]), &pos(0, 11));
    // A value of a built-in type has no declaration to jump to.
    let (mut server, _) = open("let n = 3\nprint(n)\n");
    assert_eq!(
        at(&mut server, "textDocument/typeDefinition", 1, 6),
        Value::Null
    );
}

#[test]
fn implementation_lists_every_impl_of_a_type_or_a_trait() {
    let (mut server, _) = open(ROADS);
    let road = at(&mut server, "textDocument/implementation", 0, 8);
    assert_eq!(starts(&road), [pos(4, 5), pos(6, 5)]);
    let sized = at(&mut server, "textDocument/implementation", 2, 8);
    assert_eq!(starts(&sized), [pos(4, 5)]);
}

#[test]
fn use_lines_link_to_the_module_file() {
    let (mut server, _) = open("use std.mod.{Mod}\nprint(Mod<7>(3).v)\n");
    let reply = server.handle(&request(
        7,
        "textDocument/documentLink",
        obj([("textDocument", doc())]),
    ));
    let links = reply[0].at(&["result"]).as_array().unwrap_or(&[]).to_vec();
    assert_eq!(links.len(), 1, "{links:?}");
    assert_eq!(links[0].at(&["range", "start"]), &pos(0, 4));
    assert_eq!(links[0].at(&["range", "end"]), &pos(0, 11));
    let target = links[0].at(&["target"]).as_str().unwrap_or("");
    assert!(target.ends_with("std/mod.cp"), "{target}");
}

#[test]
fn diagnostics_can_be_pulled_too() {
    let (mut server, pushed) = open("print(total)\n");
    let reply = server.handle(&request(
        7,
        "textDocument/diagnostic",
        obj([("textDocument", doc())]),
    ));
    assert_eq!(reply[0].at(&["result", "kind"]).as_str(), Some("full"));
    let pulled = reply[0].at(&["result", "items"]).as_array().unwrap_or(&[]);
    assert_eq!(pulled.len(), pushed.len());
    assert_eq!(pulled[0].at(&["code"]), pushed[0].at(&["code"]));
}
