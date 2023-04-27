use std::fs;

use hlua::{LuaTable, LuaFunction, function0};
use home::home_dir;
use crate::app::App;


pub fn call_internal<T: ToString>(app: &App, name: T) -> Option<Result<String, hlua::LuaError>> 
{
    let mut lua = app.lua.borrow_mut();
    let mut internal: LuaTable<_> = lua.get("_internal").unwrap();
    let mut on_tick: LuaFunction<_> = match internal.get(name.to_string()) {
        Some(v) => v,
        None => return None,
    };

    Some(on_tick.call())
}

pub fn setup_lua(app: &App) {
    let mut lua = app.lua.borrow_mut();
    lua.openlibs();
    lua.execute::<()>("gish = {}").unwrap();
    {
        let mut gish: LuaTable<_> = lua.get("gish").unwrap();
        gish.set("get_pwd", function0(|| {
            std::env::current_dir().unwrap().to_str().unwrap().to_string()
        }),);
        gish.set("home_dir", function0(|| {
            home_dir().unwrap().to_str().unwrap().to_string()
        }),);
    }
    let src = fs::read_to_string("./lua/on_tick.lua").unwrap();
    lua.execute::<()>(&src).unwrap();
}

