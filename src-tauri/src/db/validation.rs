use crate::types::PasswordItem;

pub fn validate_password_item_fields(
    item: &PasswordItem,
) -> std::result::Result<(), validator::ValidationError> {
    if item.title.is_empty() {
        return Err(validator::ValidationError::new("title_empty"));
    }
    if item.title.len() > 255 {
        return Err(validator::ValidationError::new("title_too_long"));
    }

    if let Some(username) = &item.username {
        if username.is_empty() {
            return Err(validator::ValidationError::new("username_empty"));
        }
    }

    let is_placeholder_password =
        item.password.trim().is_empty() || item.password.as_str() == "N/A";
    if !is_placeholder_password && item.password.len() < 8 {
        return Err(validator::ValidationError::new("password_too_short"));
    }

    if let Some(url) = &item.url {
        if !url.is_empty() && !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(validator::ValidationError::new("invalid_url_format"));
        }
    }

    if let Some(totp_secret) = &item.totp_secret {
        if !totp_secret.is_empty() && totp_secret.len() < 16 {
            return Err(validator::ValidationError::new("totp_secret_too_short"));
        }
    }

    Ok(())
}
