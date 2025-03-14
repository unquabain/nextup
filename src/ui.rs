use crate::list::ListRanker;
use cursive::{Cursive, CursiveExt, views::{Dialog,TextView,Layer}};
use std::sync::{OnceLock,Arc};

pub fn questionnaire(ranker: &mut impl ListRanker) -> Result<(),&'static str> {
    loop {
        let strings = ranker.strings();
        if strings.is_none() {
            break;
        }
        let strings = strings.unwrap();
        let choice: Arc<OnceLock::<i32>> = Arc::new(OnceLock::new());
        let choice_zero = choice.clone();
        let choice_one = choice.clone();
        let choice_two = choice.clone();

        let mut dlg = Dialog::around(TextView::new("Which of these tasks is the most urgent?"))
            .button(strings.0, move |s| { choice_zero.set(0).unwrap(); s.quit(); })
            .button(strings.1, move |s| { choice_one.set(1).unwrap(); s.quit(); });
        if let Some(right) = strings.2 {
            dlg.add_button(right, move |s| { choice_two.set(2).unwrap(); s.quit(); });
        }
        let mut siv = Cursive::default();
        let mut theme = siv.current_theme().clone();
        theme.palette = cursive::theme::Palette::terminal_default();
        siv.set_theme(theme);
        siv.add_layer(Layer::new(dlg));
        siv.run();

        let choice = choice.get().unwrap();
        if ! ranker.choose(*choice)? {
            break;
        }
    }

    Ok(())
}
