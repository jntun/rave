//
// Created by Justin Tunheim on 7/12/24
//

pub enum Value<T> {
    None,
    Default(T),
    User(T),
}

pub enum Scope {
    All,
    Region,
}

pub enum Command {
    List(Scope),
}

pub struct Configuration {
    pub command:     Value<crate::Command>,
    pub save_root:   Value<String>,
}

impl<T> Value<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            Value::None         => None,
            Value::User(val)    => Some(&val),
            Value::Default(val) => Some(&val),
        }
    }
}
