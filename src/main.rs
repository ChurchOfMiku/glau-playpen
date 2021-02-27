mod cookies;

use mlua::{Error, Lua, MultiValue, Value};

fn main() {
    let lua = Lua::new();
    let val = lua
        .load("a = {}; a['a'] = a; a[1] = a; return a")
        .eval::<Value>()
        .unwrap();
    println!("{:#?}", cookies::serialize::serialize(&lua, val));
}
