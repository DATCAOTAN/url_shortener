use url::Url;

pub fn validate_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

pub fn validate_password(password: &str) -> bool {
    password.len() >= 8 && password.len() <= 128
}

pub fn validate_username(username: &str) -> bool {
    let len = username.len();
    len >= 3 && len <= 50
}

pub fn validate_url(input: &str) -> bool {
    match Url::parse(input) {
        Ok(url) => matches!(url.scheme(), "http" | "https"),
        Err(_) => false,
    }
}

pub fn validate_title(title: &str) -> bool {
    !title.is_empty() && title.len() <= 255
}
