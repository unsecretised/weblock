use axum::http::HeaderMap;

pub fn extract_cookie(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    if !cookie_name.is_empty() {
        // First, try extracting from the Cookie header
        let from_cookie_header = headers
            .get("Cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookie_header| {
                cookie_header
                    .split(';')
                    .filter(|x| x.trim().starts_with(cookie_name) && x.trim().contains("="))
                    .map(|x| x.split_once('=').unwrap_or(("", "")).1.to_string())
                    .next()
            });

        // Fall back to trying the cookie_name directly as a header name
        if let Some(value) = from_cookie_header {
            Some(value)
        } else {
            headers
                .get(cookie_name)
                .and_then(|x| x.to_str().ok())
                .map(|s| s.to_string())
        }
    } else {
        panic!("Attempted to extract empty cookie header")
    }
}
