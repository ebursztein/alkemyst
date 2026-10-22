use super::{Error, Result};
pub(super) fn text(value: &str, maximum: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > maximum {
        return Err(Error::Invalid(
            "text is empty or exceeds its UTF-8 byte limit",
        ));
    }
    Ok(())
}
pub(super) fn key(value: &str) -> Result<()> {
    text(value, 128)?;
    if value.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "idempotency key contains control characters",
        ));
    }
    Ok(())
}
pub(super) fn page(query: &super::ChatQuery, prefix: &str) -> Result<Option<i64>> {
    if !(1..=100).contains(&query.limit) {
        return Err(Error::Invalid("page limit must be 1..=100"));
    }
    query
        .cursor
        .as_ref()
        .map(|c| {
            if c.len() != 19
                || !c.starts_with(prefix)
                || !c[3..].bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(Error::Invalid("invalid chat cursor"));
            }
            i64::from_str_radix(&c[3..], 16)
                .ok()
                .filter(|n| *n > 0)
                .ok_or(Error::Invalid("invalid chat cursor"))
        })
        .transpose()
}
