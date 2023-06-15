
#[derive(Debug)]
pub struct Config {
    pub auto_comp: bool,
    pub only_sagest_when_typing: bool,
    pub scroll_amount: u16,
    pub mouse_scroll: bool
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_comp: true,
            only_sagest_when_typing: false,
            scroll_amount: 5,
            mouse_scroll: true,
        }
    }
}
