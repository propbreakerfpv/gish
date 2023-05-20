// resources
// https://en.wikipedia.org/wiki/ANSI_escape_code
// https://stackoverflow.com/questions/4842424/list-of-ansi-color-escape-sequences




use anyhow::{anyhow, Result};
use tui::{text::{Text, Span, Spans}, style::{self, Style, Modifier}};

use crate::app::App;

pub fn test<'a>(app: &mut App, code: String) -> Text<'a> {


    let parser = AnsiParser {
        content: code.into(),
        pos: 0,
    };
    // panic!("{:?}", construct_text(app, parser.parse()));
    

    let parsed = parser.parse();

    // panic!("{:?}", parsed);
    // println!("{:?}", parsed);

    comput_cursore_moves(app, parsed.clone());

    // panic!("{:?}", app.vstdout);
    // println!("{:?}", app.vstdout);

    let ret = new_construct_text(app);
    // println!("{:?}", ret);
    ret
}

fn write_char(app: &mut App, char: Char) {
    app.vstdout[app.vc.1 as usize][app.vc.0 as usize] = char;
    app.vc.0 += 1;
}

fn comput_cursore_moves<'a>(app: &mut App, input: Vec<Char>) {
    for char in input {
        match char {
            Char::Char(char) => {
                if char == '\n' {
                    if app.vc.1 == app.vstdout.len() as u16 - 1 {
                        // todo fix this. should probubly scroll everything up by one row?
                        // text.push(Vec::new());
                    } else {
                        app.vc.1 += 1;
                        app.vc.0 = 0;
                    }
                } else {
                    write_char(app, Char::Char(char))
                }
            }
            Char::Ansi(ansi) => {
                match ansi {
                    Ansi::CursorUp(amount) => {
                        let amount = amount.parse().unwrap();
                        if app.vc.1 >= amount {
                            app.vc.1 -= amount
                        } else {
                            app.vc.1 = 0;
                        }
                    }
                    // Ansi::CursorUp(v) => {}
                    // Ansi::CursonDown(v) => {}
                    // Ansi::CursorForward(v) => {}
                    Ansi::CursorBack(amount) => {
                        let amount = amount.parse().unwrap();
                        if app.vc.0 >= amount {
                            app.vc.0 -= amount
                        } else {
                            app.vc.0 = 0;
                        }
                    }
                    // Ansi::CursorNextLine(v) => {}
                    // Ansi::CursorPreviousLine(v) => {}
                    // Ansi::CursorHorizontalAbsolute(v) => {}
                    // Ansi::CursorPosition((n, m)) => {}
                    Ansi::EraseInDisplay(_) => {}
                    // Ansi::EraseInLine(v) => {}
                    // Ansi::ScrollUp(v) => {}
                    // Ansi::ScrollDown(v) => {}
                    // Ansi::HorizontalVerticalPosition((n, m)) => {}
                    Ansi::Sgr(sgr) => {
                        // text.last_mut().unwrap().push(Char::Ansi(Ansi::Sgr(sgr)))
                        write_char(app, Char::Ansi(Ansi::Sgr(sgr)));
                    }
                    _ => {}
                }
            }
            Char::Empty => {}
        }
    }
}

fn new_construct_text<'a>(app: &mut App) -> Text<'a> {
    let mut input = Vec::new();


    let mut found = 0;
    let mut idx = 0;
    for line in app.vstdout.clone() {
        for char in line {
            if char != Char::Empty {
                found = idx;
            }
            input.push(char);
        }
        input.push(Char::Char('\n'));
        idx += 1;
    }

    let mut spans = Vec::new();
    let mut span = Vec::new();
    let mut current_text = String::new();
    let mut style = Style::default();
    for char in input {
        match char {
            Char::Char(char) => {
                if char == '\n' && found >= 0 {
                    span.push(Span::styled(current_text.clone(), style));
                    spans.push(Spans::from(span.clone()));
                    span.clear();
                    current_text.clear();
                    found -= 1;
                }
                current_text.push(char);
            }
            Char::Ansi(ansi) => {
                match ansi {
                    // Ansi::CursorUp(v) => {}
                    // Ansi::CursorUp(v) => {}
                    // Ansi::CursonDown(v) => {}
                    // Ansi::CursorForward(v) => {}
                    // Ansi::CursorBack(v) => {}
                    // Ansi::CursorNextLine(v) => {}
                    // Ansi::CursorPreviousLine(v) => {}
                    // Ansi::CursorHorizontalAbsolute(v) => {}
                    // Ansi::CursorPosition((n, m)) => {}
                    Ansi::EraseInDisplay(v) => {
                        match v.as_str() {
                            // clear from cursor to end of screen
                            "0" => {
                            }
                            // clear entire screen
                            "2" => {
                                app.content = Text::from("");
                                app.scroll = (0, 0)
                            }
                            // clear entire screen and all lines saved in  scrollback buffer
                            "3" => {}
                            _ => {}
                        }
                    }
                    // Ansi::EraseInLine(v) => {}
                    // Ansi::ScrollUp(v) => {}
                    // Ansi::ScrollDown(v) => {}
                    // Ansi::HorizontalVerticalPosition((n, m)) => {}
                    Ansi::Sgr(sgr) => {
                        if !current_text.is_empty() {
                            span.push(Span::styled(current_text.clone(), style));
                        }
                        // text.extend(Text::styled(current_text.clone(), style));
                        current_text.clear();
                        parse_sgr(sgr, &mut style);
                    }
                    _ => {}
                }
            }
            Char::Empty => {}
        }
    }
    if !current_text.is_empty() {
        span.push(Span::styled(current_text.clone(), style));
        spans.push(Spans::from(span.clone()));
    }
    Text::from(spans)
}

fn construct_text<'a>(app: &mut App, input: Vec<Char>) -> Text<'a> {
    let mut spans = Vec::new();
    let mut span = Vec::new();
    let mut current_text = String::new();
    let mut style = Style::default();
    for char in input {
        match char {
            Char::Char(char) => {
                if char == '\n' {
                    span.push(Span::styled(current_text.clone(), style));
                    spans.push(Spans::from(span.clone()));
                    span.clear();
                    current_text.clear();
                }
                current_text.push(char);
            }
            Char::Ansi(ansi) => {
                match ansi {
                    // Ansi::CursorUp(v) => {}
                    // Ansi::CursorUp(v) => {}
                    // Ansi::CursonDown(v) => {}
                    // Ansi::CursorForward(v) => {}
                    // Ansi::CursorBack(v) => {}
                    // Ansi::CursorNextLine(v) => {}
                    // Ansi::CursorPreviousLine(v) => {}
                    // Ansi::CursorHorizontalAbsolute(v) => {}
                    // Ansi::CursorPosition((n, m)) => {}
                    Ansi::EraseInDisplay(v) => {
                        match v.as_str() {
                            // clear from cursor to end of screen
                            "0" => {
                            }
                            // clear entire screen
                            "2" => {
                                app.content = Text::from("");
                                app.scroll = (0, 0)
                            }
                            // clear entire screen and all lines saved in  scrollback buffer
                            "3" => {}
                            _ => {}
                        }
                    }
                    // Ansi::EraseInLine(v) => {}
                    // Ansi::ScrollUp(v) => {}
                    // Ansi::ScrollDown(v) => {}
                    // Ansi::HorizontalVerticalPosition((n, m)) => {}
                    Ansi::Sgr(sgr) => {
                        if !current_text.is_empty() {
                            span.push(Span::styled(current_text.clone(), style));
                        }
                        // text.extend(Text::styled(current_text.clone(), style));
                        current_text.clear();
                        parse_sgr(sgr, &mut style);
                    }
                    _ => {}
                }
            }
            Char::Empty => {}
        }
    }
    if !current_text.is_empty() {
        span.push(Span::styled(current_text.clone(), style));
        spans.push(Spans::from(span.clone()));
    }
    Text::from(spans)
}

fn parse_sgr(sgrs: Vec<Sgr>, style: &mut Style) -> Style {
    for sgr in sgrs {
        match sgr {
            Sgr::Reset => {
                *style = Style::default();
                // panic!("{:?}", style);
            }
            Sgr::Bold => {
                *style = style.patch(Style::default().add_modifier(Modifier::BOLD));
            }
            Sgr::Dim => {
                *style = style.patch(Style::default().add_modifier(Modifier::DIM));
            }
            Sgr::Italic => {
                *style = style.patch(Style::default().add_modifier(Modifier::ITALIC));
            }
            Sgr::Underline => {
                *style = style.patch(Style::default().add_modifier(Modifier::UNDERLINED));
            }
            Sgr::SlowBlink => {
                *style = style.patch(Style::default().add_modifier(Modifier::SLOW_BLINK));
            }
            Sgr::RapidBlink => {
                *style = style.patch(Style::default().add_modifier(Modifier::RAPID_BLINK));
            }
            Sgr::Invert => {
                *style = style.patch(Style::default().add_modifier(Modifier::REVERSED));
            }
            Sgr::Hide => {
                *style = style.patch(Style::default().add_modifier(Modifier::HIDDEN));
            }
            Sgr::Strike => {
                *style = style.patch(Style::default().add_modifier(Modifier::CROSSED_OUT));
            }
            Sgr::PrimaryFont => {
            }
            Sgr::AltFont(_u8) => {}
            Sgr::Gothic => {}
            Sgr::DoublyUnderlined => {}
            Sgr::NormalIntensity => {}
            Sgr::NotItalicOrBlackletter => {}
            Sgr::NotUnderlined => {
                *style = style.patch(Style::default().remove_modifier(Modifier::UNDERLINED));
            }
            Sgr::NotBlinking => {
                *style = style.patch(Style::default().remove_modifier(Modifier::RAPID_BLINK).remove_modifier(Modifier::SLOW_BLINK));
            }
            Sgr::ProportionalSpacing => {}
            Sgr::NotInvert => {
                *style = style.patch(Style::default().remove_modifier(Modifier::REVERSED));
            }
            Sgr::NotHidden => {
                *style = style.patch(Style::default().remove_modifier(Modifier::HIDDEN));
            }
            Sgr::NotStrike => {
                *style = style.patch(Style::default().remove_modifier(Modifier::CROSSED_OUT));
            }
            Sgr::SetForeground7(u8) => {
                *style = style.patch(Style::default().fg(get_color_idx(u8)));
            }
            Sgr::SetForeground(color) => {
                match color {
                    Color::N(_) => {
                        // panic!("color is N");
                    }
                    Color::Rgb(rgb) => {
                        // println!("{:?}", style::Color::Rgb(rgb.r, rgb.g, rgb.b));
                        *style = style.patch(Style::default().fg(style::Color::Rgb(rgb.r, rgb.g, rgb.b)));
                        // print!("{:?}", style);
                    }
                }
            }
            Sgr::DefaultForeground => {
                *style = style.patch(Style::default().fg(style::Color::Reset));
            }
            Sgr::SetBackground7(u8) => {
                *style = style.patch(Style::default().bg(get_color_idx(u8)));
            }
            Sgr::SetBackground(color) => {
                match color {
                    Color::N(_) => {}
                    Color::Rgb(rgb) => {
                        *style = style.patch(Style::default().bg(style::Color::Rgb(rgb.r, rgb.g, rgb.b)));
                    }
                }
            }
            Sgr::DefaultBackground => {
                *style = style.patch(Style::default().bg(style::Color::Reset));
            }
            Sgr::DisableProportionalSpacing => {}
            Sgr::Framed => {}
            Sgr::Encircled => {}
            Sgr::Overlined => {}
            Sgr::NotFramedOrEncircled => {}
            Sgr::NotOverlined => {}
            Sgr::SetUnderline(_color) => {}
            Sgr::DefaultUnderline => {}

            // todo bright forground and background are not actualy bright there just the
            // normal 7 color
            Sgr::SetBrightForground7(u8) => {
                *style = style.patch(Style::default().fg(get_color_idx(u8)));
            }
            Sgr::SetBrightBackground7(u8) => {
                *style = style.patch(Style::default().bg(get_color_idx(u8)));
            }
        }
    }
    Style::default()
}

fn get_color_idx(idx: u8) -> style::Color {
    match idx {
        1 => style::Color::Red,
        2 => style::Color::Green,
        3 => style::Color::Yellow,
        4 => style::Color::Blue,
        5 => style::Color::Magenta,
        6 => style::Color::Cyan,
        7 => style::Color::White,
        _ => style::Color::Reset,
    }
}

struct AnsiParser {
    content: Vec<u8>,
    pos: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Char {
    Ansi(Ansi),
    Char(char),
    Empty,
}

impl From<u8> for Char {
    fn from(value: u8) -> Self {
        Char::Char(value as char)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Ansi {
    CursorUp(String),
    CursonDown(String),
    CursorForward(String),
    CursorBack(String),
    CursorNextLine(String),
    CursorPreviousLine(String),
    CursorHorizontalAbsolute(String),
    CursorPosition((String, String)),
    EraseInDisplay(String),
    EraseInLine(String),
    ScrollUp(String),
    ScrollDown(String),
    HorizontalVerticalPosition((String, String)),
    Sgr(Vec<Sgr>),
}

/// ESC args for SGR m
/// [01;32m"some string"[0m
/// results in "some string" being green
#[derive(Debug, Clone, PartialEq)]
enum Sgr {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    SlowBlink,
    RapidBlink,
    Invert,
    Hide,
    Strike,
    PrimaryFont,
    AltFont(u8),
    Gothic,
    /// or not bold in some terminals
    DoublyUnderlined,
    NormalIntensity,
    NotItalicOrBlackletter,
    NotUnderlined,
    NotBlinking,
    ProportionalSpacing,
    NotInvert,
    NotHidden,
    NotStrike,
    SetForeground7(u8),
    SetForeground(Color),
    DefaultForeground,
    SetBackground7(u8),
    SetBackground(Color),
    DefaultBackground,
    DisableProportionalSpacing,
    /// emojis i think
    Framed, 
    /// emojis i think
    Encircled,
    /// maybe not supported?
    Overlined,
    NotFramedOrEncircled,
    NotOverlined,
    /// not in standard
    SetUnderline(Color), 
    /// also not in standard for obvious reasons
    DefaultUnderline,

    // theres some here with realy long names that are not widely supported

    /// not widely supported afaik
    SetBrightForground7(u8),
    /// not widely supported afaik
    SetBrightBackground7(u8),
}


#[derive(Debug, Clone, PartialEq)]
enum Color {
    N(u8),
    Rgb(RgbColor),
}

#[derive(Debug, Clone, PartialEq)]
struct RgbColor {
    r: u8,
    g: u8,
    b: u8,
}

impl TryFrom<Vec<u8>> for RgbColor {
    type Error = anyhow::Error;
    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        if value.len() < 3 {
            return Err(anyhow!("invalid vector"))
        }
        Ok(RgbColor {
            r: value[0],
            g: value[1],
            b: value[2],
        })
    }
}

enum ColorType {
    N,
    Rgb
}

enum ColorPushType {
    Forground,
    Background,
}

fn vec_to_string(v: Vec<u8>) -> String {
    v.iter().map(|&x| x as char).collect::<String>()
}

fn parse_one_arg(args: Vec<u8>) -> String {
    let mut ret = String::new();
    for char in args {
        if char as char == ';' {
            break;
        }
        ret.push(char as char);
    }
    ret
}
fn parse_two_arg(args: Vec<u8>) -> (String, String) {
    let mut one = String::new();
    let mut two = String::new();
    let mut found = false;
    for char in args {
        if char as char == ';' {
            found = true;
        }
        if found {
            two.push(char as char);
        } else {
            one.push(char as char);
        }
    }
    (one, two)
}


impl AnsiParser {
    /// should return a Vec<Span>?
    fn parse(mut self) -> Vec<Char> {
        let mut ret = Vec::new();
        while !self.eof() {
            ret.push(self.parse_next())
        }
        ret
    }

    // should also return something. probubly Span
    fn parse_next(&mut self) -> Char {
        let next = self.consume_next();
        if next == 27 {
            return self.parse_escap_sequences();
        }
        Char::from(next)
    }

    // should retrun something
    fn parse_escap_sequences(&mut self) -> Char {
        match self.consume_next() {
            // todo C0 control codes. se wikipedia
            91 => { // [
                self.parse_csi_sequences()
            }
            v => { Char::from(v) }
        }
    }

    //should return something
    fn parse_csi_sequences(&mut self) -> Char {
        let args = self.parse_csi_parameter_bytes();
        let next = self.consume_next();

        // Moves the cursor n (default 1) cells in the given direction. If the 
        // cursor is already at the edge of the screen, this has no effect.
        if next == b'A' { // cursor up
            Char::Ansi(Ansi::CursorUp(parse_one_arg(args)))
        } else if next == b'B' { // cursor down
            Char::Ansi(Ansi::CursonDown(parse_one_arg(args)))
        } else if next == b'C' { // cursor forward
            Char::Ansi(Ansi::CursorForward(parse_one_arg(args)))
        } else if next == b'D' { // cursor back
            Char::Ansi(Ansi::CursorBack(parse_one_arg(args)))
        }
        // Moves cursor to beginning of the line n (default 1) lines down.
        // (not ANSI.SYS)
        else if next == b'E' {
            Char::Ansi(Ansi::CursorNextLine(parse_one_arg(args)))
        }
        // Moves cursor to beginning of the line n (default 1) lines up. 
        // (not ANSI.SYS
        else if next == b'F' {
            Char::Ansi(Ansi::CursorPreviousLine(parse_one_arg(args)))
        }
        // Moves the cursor to column n (default 1). (not ANSI.SYS)
        else if next == b'G' {
            Char::Ansi(Ansi::CursorHorizontalAbsolute(parse_one_arg(args)))
        }
        // Moves the cursor to row n, column m. The values are 1-based, and 
        // default to 1 (top left corner) if omitted. A sequence such as CSI 
        // ;5H is a synonym for CSI 1;5H as well as CSI 17;H is the same as CSI
        // 17H and CSI 17;1H
        else if next == b'H' {
            Char::Ansi(Ansi::CursorPosition(parse_two_arg(args)))
        }
        // Clears part of the screen. If n is 0 (or missing), clear from cursor
        // to end of screen. If n is 1, clear from cursor to beginning of the 
        // screen. If n is 2, clear entire screen (and moves cursor to upper 
        // left on DOS ANSI.SYS). If n is 3, clear entire screen and delete all
        // lines saved in the scrollback buffer (this feature was added for 
        // xterm and is supported by other terminal applications).
        else if next == b'J' {
            Char::Ansi(Ansi::EraseInDisplay(parse_one_arg(args)))
        }
        // Erases part of the line. If n is 0 (or missing), clear from cursor 
        // to the end of the line. If n is 1, clear from cursor to beginning 
        // of the line. If n is 2, clear entire line. Cursor position does not
        // change.
        else if next == b'K' {
            Char::Ansi(Ansi::EraseInLine(parse_one_arg(args)))
        }
        // Scroll whole page up by n (default 1) lines. New lines are added at 
        // the bottom. (not ANSI.SYS)
        else if next == b'S' {
            Char::Ansi(Ansi::ScrollUp(parse_one_arg(args)))
        }
        // Scroll whole page down by n (default 1) lines. New lines are added 
        // at the top. (not ANSI.SYS)
        else if next == b'T' {
            Char::Ansi(Ansi::ScrollDown(parse_one_arg(args)))
        }
        // Same as cursor position, but counts as a format effector function (like CR
        // or LF) rather than an editor function (like CUD or CNL). This can 
        // lead to different handling in certain terminal modes
        else if next == b'f' {
            Char::Ansi(Ansi::HorizontalVerticalPosition(parse_two_arg(args)))
        }
        // Sets colors and style of the characters following this code
        else if next == b'm' {
            Char::Ansi(Ansi::Sgr(self.parse_sgr(args).unwrap()))
        } else {
            Char::Char(next as char)
        }
        // theres some more here but they are multi character and do stuff i dont know if i will
        // suport.
        // the only thing i might need to suport is device status report witch reports the cursor
        // position. i guess you just send then through stdin?
        // maybe something to experiment with
    }

    fn parse_sgr(&mut self, args: Vec<u8>) -> Result<Vec<Sgr>, anyhow::Error> {
        // The control sequence CSI n m, named Select Graphic Rendition (SGR), 
        // sets display attributes. Several attributes can be set in the same 
        // sequence, separated by semicolons. Each display attribute remains
        // in effect until a following occurrence of SGR resets it. If no 
        // codes are given, CSI m is treated as CSI 0 m (reset / normal)

        let mut ret = Vec::new();
        let mut in_color = false;
        let mut next_color_type_select = false;
        let mut color_type = ColorType::N;
        let mut color_push_type = ColorPushType::Forground;
        let mut rgb: Vec<u8> = Vec::new();

        for arg in vec_to_string(args.clone()).split(';') {

            // println!("rgb len {}", rgb.len());


            // todo find a better way to trim leading 0s without removing valid 0s
            let mut arg = arg.trim().trim_start_matches('0');
            if arg.is_empty() {
                arg = "0"
            }
            if next_color_type_select {
                in_color = true;
                next_color_type_select = false;
                if arg == "5" {
                    color_type = ColorType::N;
                } else if arg == "2" {
                    color_type = ColorType::Rgb;
                } else {
                    // todo maybe somekind of developer mode that would show errors for this kind
                    // of stuff? till this probubly shouldn't panic.
                    panic!("invlaid color type {}", arg)
                    // invalid error
                }
                continue;
            }
            if in_color {
                match color_type {
                    ColorType::N => {
                        let color = Color::N(arg.parse().unwrap());
                        match color_push_type {
                            ColorPushType::Forground => ret.push(Sgr::SetForeground(color)),
                            ColorPushType::Background => ret.push(Sgr::SetBackground(color))
                        }
                        in_color = false;
                    }
                    ColorType::Rgb => {
                        if rgb.len() < 2 {
                            rgb.push(arg.parse().unwrap());
                            if arg == "70" {
                                panic!("{:?} {:?}", vec_to_string(args).split(';').collect::<Vec<_>>(), rgb);
                            }
                        } else {
                            rgb.push(arg.parse().unwrap());
                            // println!("pushing a color {:?}", rgb);
                            let color = Color::Rgb(RgbColor::try_from(rgb.clone()).unwrap());
                            // panic!("color is now {:?}", color);
                            match color_push_type {
                                ColorPushType::Forground => ret.push(Sgr::SetForeground(color)),
                                ColorPushType::Background => ret.push(Sgr::SetBackground(color))
                            }
                            // println!("{:?}", ret.last());
                            in_color = false;
                            // rgb.clear();
                        }
                    }
                }
                continue;
            }
            match arg {
                "0" => {
                    ret.push(Sgr::Reset)
                }
                "1" => {
                    ret.push(Sgr::Bold)
                }
                "2" => {
                    ret.push(Sgr::Dim)
                }
                "3" => {
                    ret.push(Sgr::Italic)
                }
                "4" => {
                    ret.push(Sgr::Underline)
                }
                "5" => {
                    ret.push(Sgr::SlowBlink)
                }
                "6" => {
                    ret.push(Sgr::RapidBlink)
                }
                "7" => {
                    ret.push(Sgr::Invert)
                }
                "8" => {
                    ret.push(Sgr::Hide)
                }
                "9" => {
                    ret.push(Sgr::Strike)
                }
                "10" => {
                    ret.push(Sgr::PrimaryFont)
                }
                "11" => {
                    ret.push(Sgr::AltFont(1))
                }
                "12" => {
                    ret.push(Sgr::AltFont(2))
                }
                "13" => {
                    ret.push(Sgr::AltFont(3))
                }
                "14" => {
                    ret.push(Sgr::AltFont(4))
                }
                "15" => {
                    ret.push(Sgr::AltFont(5))
                }
                "16" => {
                    ret.push(Sgr::AltFont(6))
                }
                "17" => {
                    ret.push(Sgr::AltFont(7))
                }
                "18" => {
                    ret.push(Sgr::AltFont(8))
                }
                "19" => {
                    ret.push(Sgr::AltFont(9))
                }
                "20" => {
                    ret.push(Sgr::Gothic)
                }
                "21" => {
                    ret.push(Sgr::DoublyUnderlined)
                }
                "22" => {
                    ret.push(Sgr::NormalIntensity)
                }
                "23" => {
                    ret.push(Sgr::NotItalicOrBlackletter)
                }
                "24" => {
                    ret.push(Sgr::NotUnderlined)
                }
                "25" => {
                    ret.push(Sgr::NotBlinking)
                }
                "26" => {
                    ret.push(Sgr::ProportionalSpacing)
                }
                "27" => {
                    ret.push(Sgr::NotInvert)
                }
                "28" => {
                    ret.push(Sgr::NotHidden)
                }
                "29" => {
                    ret.push(Sgr::NotStrike)
                }
                "30"|"31"|"32"|"33"|"34"|"35"|"36"|"37" =>{
                    // 29 so its a range from 1 to 8, i think thats the best way to do it...
                    ret.push(Sgr::SetForeground7(arg.parse::<u8>().unwrap() - 30))
                }
                "38" => {
                    next_color_type_select = true;
                    color_push_type = ColorPushType::Forground;
                    // ret.push(SGR::SetForeground(Color::N(0)))
                }
                "39" => {
                    ret.push(Sgr::DefaultForeground)
                }
                "40"|"41"|"42"|"43"|"44"|"45"|"46"|"47" =>{
                    // println!("4x");
                    ret.push(Sgr::SetBackground7(arg.parse::<u8>().unwrap() - 40))
                }
                "48" => {
                    // println!("48");
                    next_color_type_select = true;
                    color_push_type = ColorPushType::Background;
                    // ret.push(SGR::SetBackground(Color::N(0)))
                }
                "49" => {
                    ret.push(Sgr::DefaultBackground)
                }
                "50" => {
                    ret.push(Sgr::DisableProportionalSpacing)
                }
                "51" => {
                    ret.push(Sgr::Framed)
                }
                "52" => {
                    ret.push(Sgr::Encircled)
                }
                "53" => {
                    ret.push(Sgr::Overlined)
                }
                "54" => {
                    ret.push(Sgr::NotFramedOrEncircled)
                }
                "55" => {
                    ret.push(Sgr::NotOverlined)
                }
                "56" => {
                    ret.push(Sgr::SetUnderline(Color::N(0)))
                }
                "57" => {
                    ret.push(Sgr::DefaultUnderline)
                }
                "90"|"91"|"92"|"93"|"94"|"95"|"96"|"97" => {
                    ret.push(Sgr::SetBrightForground7(arg.parse::<u8>().unwrap() - 90))
                }
                "100"|"101"|"102"|"103"|"104"|"105"|"106"|"107" => {
                    ret.push(Sgr::SetBrightBackground7(arg.parse::<u8>().unwrap() - 100))
                }
                _ => {
                    // error?
                    // return Err(anyhow!("no a valid sgr code `{}`", e))
                    // println!("ERROR `{}`", e);
                    todo!("handle errors in sgr parser");
                }
            }
        }
        // println!("{:?}", ret);
        Ok(ret)
    }

    //should return something
    fn parse_csi_parameter_bytes(&mut self) -> Vec<u8> {
        let mut ret = Vec::new();
        loop {
            if self.eof() {
                break;
            }
            match self.next() as char {
                '0'..='9' | ';' => {
                    ret.push(self.consume_next());
                }
                _ => {
                    break;
                }
            }
        }
        ret
    }

    fn next(&self) -> u8 {
        self.content[self.pos]
    }

    fn consume_next(&mut self) -> u8 {
        self.pos += 1;
        self.content[self.pos - 1]
    }

    fn eof(&self) -> bool {
        if self.pos >= self.content.len() {
            return true;
        }
        false
    }
}
