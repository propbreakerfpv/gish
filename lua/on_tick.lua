gish.new_app = {
    search_input = "",
    cmd_input = "",
    content = "",
    mode = "",
    prompt = "",
}
gish.content = ""
gish.register_event = function(event, f)
    if _internal.events[event] == nil then
        _internal.events[event] = {f}
    else
        _internal.events[event].insert(f)
    end
end

-- old_print = print
gish.print = function(s)
    gish.content = gish.content .. s
end

ghelper = {}
function ghelper.starts_with(s, v)
    for i, c in s do
        if i > v.len() then
            return true
        end
        if c ~= v[i] then
            return false
        end
    end
end

_internal = {
    events = {},

    on_tick = function()
        local ret = Content
        Content = ""
        return ret
    end,
    event_handler = function(event)
        if gish.vents[event] ~= nil then
            for e in gish.events[event] do
                e()
            end
        end
    end,
    render_status_bar = function()
        return gish.home_dir() .. gish.get_pwd() .. "hello from lua status bar"
    end,
}


_internal.run_event = function(event)
    -- local t = _internal.events[event]
    -- for k, v in pairs(_internal.events[event]) do
    --     -- old_print(v)
    --     v()
    -- end
    return _internal.events[event][#_internal.events[event]]()
end

function do_nothing(v)
    local _ = v
end

gish.register_event("prompt", function ()
    local home = gish.home_dir()
    local pwd = gish.get_pwd()
    if pwd:sub(1, #home) == home then
        return {"0d82946{", "029d5d8prop", "0d82946}{", "037288c~" .. pwd:sub(#home + 1, #pwd), "0d82946}", }
    end
    return {"0d82946{", "029d5d8prop", "0d82946}{", "037288c" .. pwd, "0d82946}", }
end)

gish.print("hello from lua")
