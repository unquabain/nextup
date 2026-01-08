use raccacoonie::prelude::*;
use ratatui::widgets::Paragraph;

pub struct Chooser {
    choice_1: Button,
    choice_2: Button,
    choice_3: Option<Button>,
    tab: TabController,
    pub chosen: Option<i32>,
}

impl Chooser {
    pub fn new(choice_1: &str, choice_2: &str, choice_3: Option<&str>) -> Chooser {
        Chooser {
            choice_1: Button::new(choice_1, Message::Choice(0)),
            choice_2: Button::new(choice_2, Message::Choice(1)),
            choice_3: choice_3.map(|s| Button::new(s, Message::Choice(2))),
            tab: TabController::new( if choice_3.is_some() { 3 } else { 2 } ),
            chosen: None,
        }
    }
    fn num_buttons(&self) -> usize {
        if self.choice_3.is_some() {
            3
        } else {
            2
        }
    }
}

impl Model for Chooser {
    fn help(&self) -> Option<String> {
        match self.tab.get_current_index() {
            0 => self.choice_1.help(),
            1 => self.choice_2.help(),
            2 => self.choice_3.as_ref().and_then(|c| c.help()),
            _ => None,
        }
    }
    fn update(&mut self, msg: Message) -> Message {
        self.tab.update(&msg).or_else(|| {
            match msg {
                Message::Choice(idx) if idx < self.num_buttons() => {
                    self.chosen = Some(idx as i32);
                    Message::Quit
                }
                _ => Message::Noop,
            }
        })
        .or_else(|| {
            use ratatui::crossterm::event::{KeyEvent, KeyCode};
            match msg {
                Message::KeyPress(KeyEvent{code: KeyCode::Up, ..}) |
                Message::KeyPress(KeyEvent{code: KeyCode::Char('k'), ..}) |
                Message::KeyPress(KeyEvent{code: KeyCode::Char('w'), ..}) => {
                    self.tab.previous();
                    Message::Redraw
                }
                Message::KeyPress(KeyEvent{code: KeyCode::Down, ..}) |
                Message::KeyPress(KeyEvent{code: KeyCode::Char('j'), ..}) |
                Message::KeyPress(KeyEvent{code: KeyCode::Char('s'), ..}) => {
                    self.tab.next();
                    Message::Redraw
                }
                _ => Message::Noop,
            }
        })
        .or_else(|| {
            match self.tab.get_current_index() {
                0 => self.choice_1.update(msg),
                1 => self.choice_2.update(msg),
                2 => self.choice_3.as_mut().map_or(Message::Noop, |c| c.update(msg)),
                _ => Message::Noop,
            }
        })
    }
    fn view(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        f.render_widget(
        Paragraph::default()
            .block(STYLES.blur.block.clone().title("Which of these tasks is the most urgent?")),
            area);
        let inner = area.inner(Margin::new(2,2));
        let btn_height = 4 * self.num_buttons() as u16;
        let vpadding = (inner.height - btn_height) / 2;
        for (i, focused) in self.tab.iter() {
            let btn_area = Rect {
                x: inner.x,
                y: inner.y + vpadding + (i as u16) * 4,
                width: inner.width,
                height: 3,
            };
            macro_rules! view_btn {
                ($btn:expr) => {
                    {
                        $btn.set_focus(focused);
                        $btn.view(f, btn_area)?
                    }
                };
            }
            match i {
                0 => view_btn!(self.choice_1),
                1 => view_btn!(self.choice_2),
                2 => view_btn!(self.choice_3.as_mut().expect("Button 3 should exist")),
                _ => unreachable!(),
            };
        }
        let help_rect = Rect {
            x: inner.x,
            y: inner.y + inner.height - 3,
            width: inner.width,
            height: 3,
        };
        f.render_widget(
            Paragraph::new(self.help().unwrap_or_default())
                .block(STYLES.blur.block.clone().title("Help")),
            help_rect
        );

        Ok(())
    }
}

impl Runner for Chooser {}

