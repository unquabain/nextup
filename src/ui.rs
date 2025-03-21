use crate::list::ListRanker;
use crate::error::Error;
use cursive::{Cursive, CursiveExt, view::{Nameable,Resizable}, views::{Dialog,TextView,Layer, EditView}};
use std::sync::{OnceLock,Arc};
use log::debug;

fn themed_siv() -> Cursive {
    let mut siv = Cursive::default();
    let mut theme = siv.current_theme().clone();
    theme.palette = cursive::theme::Palette::terminal_default();
    siv.set_theme(theme);
    siv
}

pub fn questionnaire(ranker: &mut impl ListRanker) -> Result<(),Error> {
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
            .button(strings.0, move |s| { debug!("chose 0"); choice_zero.set(0).unwrap(); s.quit(); })
            .button(strings.1, move |s| { debug!("chose 1"); choice_one.set(1).unwrap(); s.quit(); });
        if let Some(right) = strings.2 {
            dlg.add_button(right, move |s| { debug!("chose 2"); choice_two.set(2).unwrap(); s.quit(); });
        }
        let mut siv = themed_siv();
        siv.add_layer(Layer::new(dlg));
        siv.run();

        let choice = choice.get().unwrap();
        if ! ranker.choose(*choice)? {
            break;
        }
    }

    Ok(())
}

pub fn get_secret(name: &str) -> Result<String, Error> {
    let secret: Arc<OnceLock<String>> = Arc::new(OnceLock::new());
    let set_secret = secret.clone();
    let mut siv = themed_siv();
    siv.add_layer(
        Dialog::new()
        .title(format!("Enter secret: {}", name))
        .padding_lrtb(1, 1, 1, 0)
        .content(
            EditView::new()
            .secret()
            .with_name("secret")
            .fixed_width(20),
        )
        .button("Ok", move |s| {
            let name = s
                .call_on_name("secret", |view: &mut EditView| view.get_content())
                .unwrap();
            set_secret.set(name.to_string().clone()).unwrap();
            s.quit();
        }),
    );
    siv.run();
    let secret = secret.get();
    if secret.is_none() {
        return Err(Error::new("No secret entered"));
    }
    debug!("got secret");
    Ok(secret.unwrap().clone())
}
