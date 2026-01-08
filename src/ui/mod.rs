use crate::list::ListRanker;
use crate::error::Error;
use raccacoonie::Runner;
mod chooser;
mod input;

pub async fn questionnaire(ranker: &mut impl ListRanker) -> Result<(),Error> {
    loop {
        let mut chooser = match ranker.strings() {
            None => break,
            Some((c1, c2, c3)) => chooser::Chooser::new(c1, c2, c3),
        };
        chooser.run().await?;
        let choice = match chooser.chosen {
            Some(c) => c,
            None => return Err(Error::NoChoiceMade),
        };
        if ! ranker.choose(choice)? {
            break;
        }
    }

    Ok(())
}

pub async fn get_secret(name: &str) -> Result<String, Error> {
    let mut input = input::Input::new(name);
    input.run().await?;
    input.value.ok_or(Error::NoSecretEntered)
}
