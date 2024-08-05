//
// Created by Justin Tunheim on 7/12/24
//

use std::fmt::Debug;

pub enum Value<T> {
    None,
    Default(T),
    User(T),
}

pub enum Index {
    First,
    Value(usize),
    All,
}

pub enum Scope {
    All,
    Region,
}

pub enum Method {
    Name(String),
}

pub enum Command {
    List(Scope),
    Search(Method),
}

pub struct Configuration {
    pub command:   Value<crate::Command>,
    pub save_root: Value<String>,
    pub index:     Value<Index>,
}

impl<T> Value<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            Value::None             => None,
            Value::User(val)    => Some(&val),
            Value::Default(val) => Some(&val),
        }
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::List(scope) => {
                f.write_str("list: ")?;
                match scope {
                    Scope::All => f.write_str("all"),
                    Scope::Region => f.write_str("region"),
                }
            },
            Command::Search(method) => {
                f.write_str("search: ")?;
                match method {
                    Method::Name(name) => f.write_fmt(format_args!("[name]: \"{}\"", name)),
                }
            },
        }
    }
}

impl Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Index::First => f.write_str("first"),
            Index::Value(i) => f.write_fmt(format_args!("{}", i)),
            Index::All => f.write_str("all"),
        }
    }
}

impl<T> Debug for Value<T> 
where
    T: Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => f.write_str("None"),
            Value::Default(T) => f.write_str("Default"),
            Value::User(T) => f.write_str("User"),
        }
    }
}

impl Debug for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("command", &self.command.value().unwrap())
            .field("root", &self.save_root)
            .field("index", &self.index)
            .finish()
    }
}
