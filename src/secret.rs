use crate::ui::get_secret;
use crate::error::Error;
use keyring::Entry;

pub fn replace_secrets(template: &mut String) -> Result<(), Error> {
    let opens = template.match_indices("{{").collect::<Vec<_>>();
    if opens.is_empty() {
        return Ok(());
    }
    let closes = template.match_indices("}}").collect::<Vec<_>>();
    if closes.len() != opens.len() {
        return Err(Error::new("unmatched {{"));
    }
    let mut replacements = Vec::new();
    for (open, close) in opens.iter().zip(closes.iter()) {
        let open = open.0;
        let close = close.0;
        let secret_name = &template[open+2..close].trim();
        let entry = Entry::new("nextup", secret_name).map_err(Error::from_error)?;
        replacements.push((open, close + 2, entry.get_password().map_err(Error::from_error)?));
    }
    while let Some(last) = replacements.pop() {
        template.replace_range(last.0..last.1, &last.2);
    }


    Ok(())
}

pub fn add_secret(name: &str) -> Result<(), Error> {
    let secret = get_secret(name)?;
    let entry = Entry::new("nextup", name).map_err(Error::from_error)?;
    entry.set_password(&secret).map_err(Error::from_error)
}

pub fn delete_secret(name: &str) -> Result<(), Error> {
    let entry = Entry::new("nextup", name).map_err(Error::from_error)?;
    entry.delete_credential().map_err(Error::from_error)
}
