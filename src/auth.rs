use crate::Error;

#[derive(Clone, Debug)]
pub struct AuthDetails {
    pub user: usize,
    pub key: String,
}

impl AuthDetails {
    pub fn from_query_string(qs: &str) -> Result<Self, Error> {
        let user_start = qs.find("&user_id=").ok_or(Error::ParseAuth)?;
        let user_raw = &qs[user_start + 9..];
        let user = user_raw.parse().map_err(|err| Error::ParseUserId(err))?;
        let key = qs[9..user_start].to_string();

        Ok(Self { user, key })
    }
}
