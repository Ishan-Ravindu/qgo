use dialoguer::{theme::ColorfulTheme, Confirm};

pub fn confirm(message: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap_or(false)
}
