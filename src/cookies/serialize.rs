use mlua::Lua;
use mlua::Value;

use std::collections::HashMap;

use crate::cookies;
use crate::cookies::{Cookie, CookieId};

struct State<'a> {
    cookies: Vec<Cookie>,
    cookies_idx: mlua::Table<'a>,
}

#[derive(Debug)]
pub enum Error<'a> {
    LuaError(mlua::Error),
    UnknownCookieId(Value<'a>),
    UnsupportedValue(Value<'a>),
    NotImplemented,
}

impl From<mlua::Error> for Error<'_> {
    fn from(error: mlua::Error) -> Self {
        Self::LuaError(error)
    }
}

pub fn serialize<'a>(lua: &'a Lua, val: Value<'a>) -> Result<cookies::Jar, Error<'a>> {
    let gc_was_running = lua.gc_is_running();

    if gc_was_running {
        lua.gc_stop();
    }

    let mut state = State::new(lua)?;
    let jar = state.create_jar(val)?;

    if gc_was_running {
        lua.gc_restart();
    }

    Ok(jar)
}

impl<'a> State<'a> {
    fn new(lua: &'a Lua) -> Result<Self, Error<'a>> {
        Ok(Self {
            cookies: vec![Cookie::Nil],
            cookies_idx: lua.create_table()?,
        })
    }

    fn create_jar(mut self, val: Value<'a>) -> Result<cookies::Jar, Error<'a>> {
        let root = self.serialize_value(val)?;

        Ok(cookies::Jar {
            cookies: self.cookies,
            root,
        })
    }

    fn create_cookie(&mut self, val: Value<'a>) -> Result<CookieId, Error<'a>> {
        // TODO: Error if too many cookies
        let id = self.cookies.len();
        self.cookies.push(Cookie::Nil);
        self.cookies_idx.raw_set(val.clone(), id as i64)?; // TODO: This clone should not be necessary

        let cookie = match val {
            Value::Nil => unreachable!(),
            Value::Boolean(v) => Cookie::Boolean(v),
            Value::Integer(v) => Cookie::Integer(v),
            Value::Number(v) => Cookie::Number(v),
            Value::String(v) => Cookie::String(v.as_bytes().to_vec()),

            Value::Table(tab) => {
                let mut keys = vec![];
                let mut values = vec![];

                let metatable = match tab.get_metatable() {
                    Some(metatable) => Value::Table(metatable),
                    None => Value::Nil,
                };

                let metatable = self.serialize_value(metatable)?;

                for pair in tab.pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    keys.push(self.serialize_value(key)?);
                    values.push(self.serialize_value(value)?);
                }

                Cookie::Table {
                    keys,
                    values,
                    metatable,
                }
            }

            Value::Function(func) => Cookie::Function {
                bytecode: func.dump(false)?,
            },

            other => return Err(Error::UnsupportedValue(other)),
        };

        self.cookies[id] = cookie;

        Ok(CookieId(id as i64))
    }

    fn serialize_value(&mut self, val: Value<'a>) -> Result<CookieId, Error<'a>> {
        if let Value::Nil = val {
            return Ok(CookieId(0));
        }

        match self.cookies_idx.raw_get::<_, Value>(val.clone())? {
            // TODO: This clone should not be necessary
            // We've already been written to the cookie list, so just return our cached id
            Value::Integer(id) => return Ok(CookieId(id)),

            // Not in the cookie list, add us!
            Value::Nil => {
                return self.create_cookie(val);
            }

            other => return Err(Error::UnknownCookieId(other)),
        }
    }
}
