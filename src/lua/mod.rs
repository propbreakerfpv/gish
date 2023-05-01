use std::fs;

use hlua::{LuaTable, LuaFunction, function0};
use home::home_dir;
use tui::{text::Span, style::{Style, Color}};
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

pub fn run_ui_update<'a, T: ToString>(app: &App, event: T) -> Vec<Span<'a>> {
    let mut lua = app.lua.borrow_mut();
    let mut internal: LuaTable<_> = lua.get("_internal").unwrap();
    let run_event: Option<LuaFunction<_>> = internal.get("run_event");

    match run_event {
        Some(mut v) => {
            let mut ret = Vec::new();
            let mut table: LuaTable<_> = v.call_with_args(event.to_string()).unwrap();
            for (_, v) in table.iter::<i32, String>().filter_map(|e| e) {
                let ground: u8 = v[0..1].parse().unwrap();
                let r = u8::from_str_radix(&v[1..=2], 16).unwrap();
                let g = u8::from_str_radix(&v[3..=4], 16).unwrap();
                let b = u8::from_str_radix(&v[5..=6], 16).unwrap();
                if ground == 2 {
                    let br = u8::from_str_radix(&v[6..8], 16).unwrap();
                    let bg = u8::from_str_radix(&v[8..10], 16).unwrap();
                    let bb = u8::from_str_radix(&v[10..12], 16).unwrap();
                    let value = v[13..=v.len() - 1].to_string();
                    ret.push(Span::styled(value, Style::default().bg(Color::Rgb(r, g, b)).fg(Color::Rgb(br, bg, bb))))
                } else if ground == 1 {
                    let value = v[7..=v.len() - 1].to_string();
                    ret.push(Span::styled(value, Style::default().bg(Color::Rgb(r, g, b))))
                } else if ground == 0 {
                    let value = v[7..=v.len() - 1].to_string();
                    ret.push(Span::styled(value, Style::default().fg(Color::Rgb(r, g, b))))
                }
            }
            ret
        }
        None => {vec![Span::from("error in lua")]}
    }
}
