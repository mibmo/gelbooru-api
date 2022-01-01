use crate::Error;

#[derive(Clone, Copy, Debug)]
pub struct AuthDetails<'a> {
    user: usize,
    key: &'a str,
}

impl<'a> AuthDetails<'a> {
    pub fn from_query_string<'b: 'a>(qs: &'b str) -> Result<Self, Error> {
        let user_start = qs.find("&user_id=").ok_or(Error::AuthParse(None))?;
        let user_raw = &qs[user_start + 9..];
        let user = user_raw
            .parse()
            .map_err(|err| Error::AuthParse(Some(err)))?;
        let key = &qs[9..user_start];

        Ok(Self { user, key })
    }
}
