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
        let strings = match ranker.strings() {
            None => break,
            Some(s) => s,
        };
        let choice: Arc<OnceLock::<i32>> = Arc::new(OnceLock::new());
        let mut dlg = Dialog::around(TextView::new("Which of these tasks is the most urgent?"));

        macro_rules! add_button {
            (label: $label:expr, index: $idx:expr) => {
                dlg.add_button(
                    $label,
                    {
                        let choice = choice.clone();
                        move |s| {
                            debug!("chose {}", $idx);
                            match choice.set($idx) {
                                Ok(_) => {},
                                Err(_) => s.set_user_data(Error::ChoiceAlreadySet),
                            }
                            s.quit();
                        }
                    }
                );
            };
        }


        add_button!(label: strings.0, index: 0);
        add_button!(label: strings.1, index: 1);
        if let Some(right) = strings.2 {
            add_button!(label: right, index: 2);
        }
        let mut siv = themed_siv();
        siv.add_layer(Layer::new(dlg));
        siv.run();

        if let Some(err) = siv.take_user_data::<Error>() {
            return Err(err);
        }

        let choice = match choice.get() {
            Some(c) => c,
            None => return Err(Error::NoChoiceMade),
        };
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
            let name = match s.call_on_name("secret", |view: &mut EditView| view.get_content()) {
                Some(n) => n,
                None => {
                    s.set_user_data(Error::NoSecretEntered);
                    s.quit();
                    return;
                }
            };
            match set_secret.set(name.to_string().clone()) {
                Ok(_) => {},
                Err(_) => s.set_user_data(Error::ChoiceAlreadySet),
            }
            s.quit();
        }),
    );
    siv.run();
    if let Some(err) = siv.take_user_data::<Error>() {
        return Err(err);
    }
    let secret = secret.get();
    if secret.is_none() {
        return Err(Error::NoSecretEntered);
    }
    debug!("got secret");
    match secret {
        Some(s) => Ok(s.clone()),
        None => Err(Error::NoSecretEntered),
    }
}
