// resources
// https://en.wikipedia.org/wiki/ANSI_escape_code
// https://stackoverflow.com/questions/4842424/list-of-ansi-color-escape-sequences


use anyhow::{anyhow, Result};
use tui::{text::{Text, Span, Spans}, style::{self, Style, Modifier}};

pub fn test<'a>(code: String) -> Text<'a> {
    let parser = ANSI_Parser {
        content: code.into(),
        pos: 0,
    };
    // panic!("{:?}", construct_text(parser.parse()));
    construct_text(parser.parse())
}

fn construct_text<'a>(input: Vec<Char>) -> Text<'a> {
    let mut spans = Vec::new();
    let mut span = Vec::new();
    let mut current_text = String::new();
    let mut style = Style::default();
    for char in input {
        match char {
            Char::Char(char) => {
                if char == '\n' {
                    span.push(Span::styled(current_text.clone(), style.clone()));
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
                    // Ansi::EraseInDisplay(v) => {}
                    // Ansi::EraseInLine(v) => {}
                    // Ansi::ScrollUp(v) => {}
                    // Ansi::ScrollDown(v) => {}
                    // Ansi::HorizontalVerticalPosition((n, m)) => {}
                    Ansi::SGR(sgr) => {
                        if current_text.len() != 0 {
                            span.push(Span::styled(current_text.clone(), style.clone()));
                        }
                        // text.extend(Text::styled(current_text.clone(), style.clone()));
                        current_text.clear();
                        parse_sgr(sgr, &mut style);
                    }
                    _ => {}
            }
            }
        }
    }
    if current_text.len() != 0 {
        span.push(Span::styled(current_text.clone(), style.clone()));
        spans.push(Spans::from(span.clone()));
    }
    Text::from(spans)
}

fn parse_sgr(sgrs: Vec<SGR>, style: &mut Style) -> Style {
    for sgr in sgrs {
        match sgr {
            SGR::Reset => {
                *style = Style::default();
                // panic!("{:?}", style);
            }
            SGR::Bold => {
                *style = style.patch(Style::default().add_modifier(Modifier::BOLD));
            }
            SGR::Dim => {
                *style = style.patch(Style::default().add_modifier(Modifier::DIM));
            }
            SGR::Italic => {
                *style = style.patch(Style::default().add_modifier(Modifier::ITALIC));
            }
            SGR::Underline => {
                *style = style.patch(Style::default().add_modifier(Modifier::UNDERLINED));
            }
            SGR::SlowBlink => {
                *style = style.patch(Style::default().add_modifier(Modifier::SLOW_BLINK));
            }
            SGR::RapidBlink => {
                *style = style.patch(Style::default().add_modifier(Modifier::RAPID_BLINK));
            }
            SGR::Invert => {
                *style = style.patch(Style::default().add_modifier(Modifier::REVERSED));
            }
            SGR::Hide => {
                *style = style.patch(Style::default().add_modifier(Modifier::HIDDEN));
            }
            SGR::Strike => {
                *style = style.patch(Style::default().add_modifier(Modifier::CROSSED_OUT));
            }
            SGR::PrimaryFont => {
            }
            SGR::AltFont(_u8) => {}
            SGR::Gothic => {}
            SGR::DoublyUnderlined => {}
            SGR::NormalIntensity => {}
            SGR::NotItalicOrBlackletter => {}
            SGR::NotUnderlined => {
                *style = style.patch(Style::default().remove_modifier(Modifier::UNDERLINED));
            }
            SGR::NotBlinking => {
                *style = style.patch(Style::default().remove_modifier(Modifier::RAPID_BLINK).remove_modifier(Modifier::SLOW_BLINK));
            }
            SGR::ProportionalSpacing => {}
            SGR::NotInvert => {
                *style = style.patch(Style::default().remove_modifier(Modifier::REVERSED));
            }
            SGR::NotHidden => {
                *style = style.patch(Style::default().remove_modifier(Modifier::HIDDEN));
            }
            SGR::NotStrike => {
                *style = style.patch(Style::default().remove_modifier(Modifier::CROSSED_OUT));
            }
            SGR::SetForeground7(u8) => {
                *style = style.patch(Style::default().fg(get_color_idx(u8)));
            }
            SGR::SetForeground(color) => {
                match color {
                    Color::N(_) => {}
                    Color::RGB(rgb) => {
                        style.patch(Style::default().fg(style::Color::Rgb(rgb.r, rgb.g, rgb.b)));
                    }
                }
            }
            SGR::DefaultForeground => {
                *style = style.patch(Style::default().fg(style::Color::Reset));
            }
            SGR::SetBackground7(u8) => {
                *style = style.patch(Style::default().bg(get_color_idx(u8)));
            }
            SGR::SetBackground(color) => {
                match color {
                    Color::N(_) => {}
                    Color::RGB(rgb) => {
                        *style = style.patch(Style::default().bg(style::Color::Rgb(rgb.r, rgb.g, rgb.b)));
                    }
                }
            }
            SGR::DefaultBackground => {
                *style = style.patch(Style::default().bg(style::Color::Reset));
            }
            SGR::DisableProportionalSpacing => {}
            SGR::Framed => {}
            SGR::Encircled => {}
            SGR::Overlined => {}
            SGR::NotFramedOrEncircled => {}
            SGR::NotOverlined => {}
            SGR::SetUnderline(_color) => {}
            SGR::DefaultUnderline => {}

            // todo bright forground and background are not actualy bright there just the
            // normal 7 color
            SGR::SetBrightForground7(u8) => {
                *style = style.patch(Style::default().fg(get_color_idx(u8)));
            }
            SGR::SetBrightBackground7(u8) => {
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

#[allow(non_camel_case_types)]
struct ANSI_Parser {
    content: Vec<u8>,
    pos: usize,
}

#[derive(Debug)]
enum Char {
    Ansi(Ansi),
    Char(char),
}

impl From<u8> for Char {
    fn from(value: u8) -> Self {
        Char::Char(value as char)
    }
}

#[derive(Debug)]
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
    SGR(Vec<SGR>),
}

/// ESC args for SGR m
/// [01;32m"some string"[0m
/// results in "some string" being green
#[derive(Debug)]
enum SGR {
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


#[derive(Debug)]
enum Color {
    N(u8),
    RGB(RgbColor),
}

#[derive(Debug)]
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


impl ANSI_Parser {
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
            91 => {
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
        if next == 'A' as u8 { // cursor up
            Char::Ansi(Ansi::CursorUp(parse_one_arg(args)))
        } else if next == 'B' as u8 { // cursor down
            Char::Ansi(Ansi::CursonDown(parse_one_arg(args)))
        } else if next == 'C' as u8 { // cursor forward
            Char::Ansi(Ansi::CursorForward(parse_one_arg(args)))
        } else if next == 'D' as u8 { // cursor back
            Char::Ansi(Ansi::CursorBack(parse_one_arg(args)))
        }
        // Moves cursor to beginning of the line n (default 1) lines down.
        // (not ANSI.SYS)
        else if next == 'E' as u8 {
            Char::Ansi(Ansi::CursorNextLine(parse_one_arg(args)))
        }
        // Moves cursor to beginning of the line n (default 1) lines up. 
        // (not ANSI.SYS
        else if next == 'F' as u8 {
            Char::Ansi(Ansi::CursorPreviousLine(parse_one_arg(args)))
        }
        // Moves the cursor to column n (default 1). (not ANSI.SYS)
        else if next == 'G' as u8 {
            Char::Ansi(Ansi::CursorHorizontalAbsolute(parse_one_arg(args)))
        }
        // Moves the cursor to row n, column m. The values are 1-based, and 
        // default to 1 (top left corner) if omitted. A sequence such as CSI 
        // ;5H is a synonym for CSI 1;5H as well as CSI 17;H is the same as CSI
        // 17H and CSI 17;1H
        else if next == 'H' as u8 {
            Char::Ansi(Ansi::CursorPosition(parse_two_arg(args)))
        }
        // Clears part of the screen. If n is 0 (or missing), clear from cursor
        // to end of screen. If n is 1, clear from cursor to beginning of the 
        // screen. If n is 2, clear entire screen (and moves cursor to upper 
        // left on DOS ANSI.SYS). If n is 3, clear entire screen and delete all
        // lines saved in the scrollback buffer (this feature was added for 
        // xterm and is supported by other terminal applications).
        else if next == 'J' as u8 {
            Char::Ansi(Ansi::EraseInDisplay(parse_one_arg(args)))
        }
        // Erases part of the line. If n is 0 (or missing), clear from cursor 
        // to the end of the line. If n is 1, clear from cursor to beginning 
        // of the line. If n is 2, clear entire line. Cursor position does not
        // change.
        else if next == 'K' as u8 {
            Char::Ansi(Ansi::EraseInLine(parse_one_arg(args)))
        }
        // Scroll whole page up by n (default 1) lines. New lines are added at 
        // the bottom. (not ANSI.SYS)
        else if next == 'S' as u8 {
            Char::Ansi(Ansi::ScrollUp(parse_one_arg(args)))
        }
        // Scroll whole page down by n (default 1) lines. New lines are added 
        // at the top. (not ANSI.SYS)
        else if next == 'T' as u8 {
            Char::Ansi(Ansi::ScrollDown(parse_one_arg(args)))
        }
        // Same as cursor position, but counts as a format effector function (like CR
        // or LF) rather than an editor function (like CUD or CNL). This can 
        // lead to different handling in certain terminal modes
        else if next == 'f' as u8 {
            Char::Ansi(Ansi::HorizontalVerticalPosition(parse_two_arg(args)))
        }
        // Sets colors and style of the characters following this code
        else if next == 'm' as u8 {
            Char::Ansi(Ansi::SGR(self.parse_sgr(args).unwrap()))
        } else {
            Char::Char(next as char)
        }
        // theres some more here but they are multi character and do stuff i dont know if i will
        // suport.
        // the only thing i might need to suport is device status report witch reports the cursor
        // position. i guess you just send then through stdin?
        // maybe something to experiment with
    }

    fn parse_sgr(&mut self, args: Vec<u8>) -> Result<Vec<SGR>, anyhow::Error> {
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



        for arg in vec_to_string(args).split(";") {


            // todo find a better way to trim leading 0s without removing valid 0s
            let mut arg = arg.trim().trim_start_matches("0");
            if arg.len() == 0 {
                arg = "0"
            }
            if next_color_type_select {
                in_color = true;
                next_color_type_select = false;
                if arg == "5" {
                    color_type = ColorType::Rgb;
                } else if arg == "2" {
                    color_type = ColorType::Rgb;
                } else {
                    // invalid error
                }
            }
            if in_color {
                match color_type {
                    ColorType::N => {
                        let color = Color::N(arg.parse().unwrap());
                        match color_push_type {
                            ColorPushType::Forground => ret.push(SGR::SetForeground(color)),
                            ColorPushType::Background => ret.push(SGR::SetBackground(color))
                        }
                        in_color = false;
                    }
                    ColorType::Rgb => {
                        if rgb.len() < 3 {
                            rgb.push(arg.parse().unwrap())
                        } else {
                            let color = Color::RGB(RgbColor::try_from(rgb.clone()).unwrap());
                            match color_push_type {
                                ColorPushType::Forground => ret.push(SGR::SetForeground(color)),
                                ColorPushType::Background => ret.push(SGR::SetBackground(color))
                            }
                            in_color = false;
                        }
                    }
                }
            }
            match arg {
                "0" => {
                    ret.push(SGR::Reset)
                }
                "1" => {
                    ret.push(SGR::Bold)
                }
                "2" => {
                    ret.push(SGR::Dim)
                }
                "3" => {
                    ret.push(SGR::Italic)
                }
                "4" => {
                    ret.push(SGR::Underline)
                }
                "5" => {
                    ret.push(SGR::SlowBlink)
                }
                "6" => {
                    ret.push(SGR::RapidBlink)
                }
                "7" => {
                    ret.push(SGR::Invert)
                }
                "8" => {
                    ret.push(SGR::Hide)
                }
                "9" => {
                    ret.push(SGR::Strike)
                }
                "10" => {
                    ret.push(SGR::PrimaryFont)
                }
                "11" => {
                    ret.push(SGR::AltFont(1))
                }
                "12" => {
                    ret.push(SGR::AltFont(2))
                }
                "13" => {
                    ret.push(SGR::AltFont(3))
                }
                "14" => {
                    ret.push(SGR::AltFont(4))
                }
                "15" => {
                    ret.push(SGR::AltFont(5))
                }
                "16" => {
                    ret.push(SGR::AltFont(6))
                }
                "17" => {
                    ret.push(SGR::AltFont(7))
                }
                "18" => {
                    ret.push(SGR::AltFont(8))
                }
                "19" => {
                    ret.push(SGR::AltFont(9))
                }
                "20" => {
                    ret.push(SGR::Gothic)
                }
                "21" => {
                    ret.push(SGR::DoublyUnderlined)
                }
                "22" => {
                    ret.push(SGR::NormalIntensity)
                }
                "23" => {
                    ret.push(SGR::NotItalicOrBlackletter)
                }
                "24" => {
                    ret.push(SGR::NotUnderlined)
                }
                "25" => {
                    ret.push(SGR::NotBlinking)
                }
                "26" => {
                    ret.push(SGR::ProportionalSpacing)
                }
                "27" => {
                    ret.push(SGR::NotInvert)
                }
                "28" => {
                    ret.push(SGR::NotHidden)
                }
                "29" => {
                    ret.push(SGR::NotStrike)
                }
                "30"|"31"|"32"|"33"|"34"|"35"|"36"|"37" =>
                /* v if v.parse::<u8>().unwrap() > 30 && v.parse::<u8>().unwrap() <= 37 => */ {
                    // 29 so its a range from 1 to 8, i think thats the best way to do it...
                    ret.push(SGR::SetForeground7(arg.parse::<u8>().unwrap() - 30))
                }
                "38" => {
                    next_color_type_select = true;
                    color_push_type = ColorPushType::Forground;
                    // ret.push(SGR::SetForeground(Color::N(0)))
                }
                "39" => {
                    ret.push(SGR::DefaultForeground)
                }
                "40"|"41"|"42"|"43"|"44"|"45"|"46"|"47" =>
                /* v if v.parse::<u8>().unwrap() > 40 && v.parse::<u8>().unwrap() <= 47 =>  */{
                    ret.push(SGR::SetBackground7(arg.parse::<u8>().unwrap() - 40))
                }
                "48" => {
                    next_color_type_select = true;
                    color_push_type = ColorPushType::Background;
                    // ret.push(SGR::SetBackground(Color::N(0)))
                }
                "49" => {
                    ret.push(SGR::DefaultBackground)
                }
                "50" => {
                    ret.push(SGR::DisableProportionalSpacing)
                }
                "51" => {
                    ret.push(SGR::Framed)
                }
                "52" => {
                    ret.push(SGR::Encircled)
                }
                "53" => {
                    ret.push(SGR::Overlined)
                }
                "54" => {
                    ret.push(SGR::NotFramedOrEncircled)
                }
                "55" => {
                    ret.push(SGR::NotOverlined)
                }
                "56" => {
                    ret.push(SGR::SetUnderline(Color::N(0)))
                }
                "57" => {
                    ret.push(SGR::DefaultUnderline)
                }
                "90"|"91"|"92"|"93"|"94"|"95"|"96"|"97" => {
                    ret.push(SGR::SetBrightForground7(arg.parse::<u8>().unwrap() - 90))
                }
                "100"|"101"|"102"|"103"|"104"|"105"|"106"|"107" => {
                    ret.push(SGR::SetBrightBackground7(arg.parse::<u8>().unwrap() - 100))
                }
                _ => {
                    // error?
                    // return Err(anyhow!("no a valid sgr code `{}`", e))
                    // println!("ERROR `{}`", e);
                }
            }
        }
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
        return self.content[self.pos - 1];
    }

    fn eof(&self) -> bool {
        if self.pos >= self.content.len() {
            return true;
        }
        return false;
    }
}
