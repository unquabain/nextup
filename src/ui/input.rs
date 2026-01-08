use raccacoonie::prelude::*;
use ratatui::widgets::Paragraph;


pub struct Input {
    name: String,
    input: InputControl,
    pub value: Option<String>,
}

impl Input {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            input: InputControl::default(),
            value: None,
        }
    }
}

impl Model for Input {
    fn help(&self) -> Option<String> {
        Some("Type your input and press Enter to confirm.".to_string())
    }
    fn update(&mut self, msg: Message) -> Message {
        use ratatui::crossterm::event::{KeyEvent, KeyCode};
        match msg {
            Message::KeyPress(KeyEvent{code: KeyCode::Enter, ..}) => {
                self.value = Some(self.input.value());
                Message::Quit
            }
            _ => Message::Noop,
        }.or_else(|| self.input.update(msg))
    }
    fn view(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
        Paragraph::default()
            .block(STYLES.blur.block.clone().title(format!("Enter value for '{}'", self.name))),
            area);
        let areas: [Rect; 4] = 
            Layout::vertical([
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]).areas(area.inner(Margin::new(2,2)));
        self.input.set_focus(FocusState::Focus);
        self.input.view(frame, areas[1])?;
        frame.render_widget(
            Paragraph::new(self.help().unwrap_or_default())
                .block(STYLES.blur.block.clone().title("Help")),
            areas[3]
        );
        Ok(())
    }
}

impl Runner for Input {}
