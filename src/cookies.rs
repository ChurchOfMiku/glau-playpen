pub mod serialize;

#[derive(Debug)]
pub struct CookieId(i64);

#[derive(Debug)]
enum Cookie {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(Vec<u8>),
    Table {
        keys: Vec<CookieId>,
        values: Vec<CookieId>,
        metatable: CookieId,
    },
    Function {
        bytecode: Vec<u8>,
        // TODO: upvalues
    },
}

#[derive(Debug)]
pub struct Jar {
    cookies: Vec<Cookie>,
    root: CookieId,
}
