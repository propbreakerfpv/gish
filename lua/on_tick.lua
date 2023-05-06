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

gish.print = function(s)
    gish.content = gish.content .. s
end

gish.alais = function (name, value)
    _internal.refresh.alais = _internal.refresh.alais .. name .. ":" .. value .. "\n"
end


_internal = {
    events = {},
    refresh = {
        alais = ""
    },

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
}


_internal.run_event = function(event)
    return _internal.events[event][#_internal.events[event]]()
end


gish.register_event("prompt", function ()
    local home = gish.home_dir()
    local pwd = gish.get_pwd()
    if pwd:sub(1, #home) == home then
        return {"0d82946{", "029d5d8prop", "0d82946}{", "037288c~" .. pwd:sub(#home + 1, #pwd), "0d82946} ", }
    end
    return {"0d82946{", "029d5d8prop", "0d82946}{", "037288c" .. pwd, "0d82946} ", }
end)

gish.register_event("render_status_bar", function ()
    return {"2d82946e8e8e8hello", "029d5d8 world"}
end)

gish.register_event("refresh", function ()
    return _internal.refresh
end)

gish.alais("ls", "ls --color")

-- gish.print("hello from lua")
