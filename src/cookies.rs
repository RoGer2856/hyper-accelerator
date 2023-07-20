use cookie::{ParseError, SplitCookies};
use hyper::header::ToStrError;

#[derive(Debug)]
pub enum CookieParseError {
    ToStr(ToStrError),
    CookieParseError(ParseError),
}

pub struct CookieIter<'a> {
    iter: hyper::header::Iter<'a, hyper::header::HeaderValue>,
    split_cookies: Option<SplitCookies<'a>>,
}

impl<'a> Iterator for CookieIter<'a> {
    type Item = Result<cookie::Cookie<'a>, CookieParseError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.split_cookies {
                Some(split_cookies) => {
                    if let Some(cookie) = split_cookies.next() {
                        break Some(cookie.map_err(CookieParseError::CookieParseError));
                    } else {
                        self.split_cookies = None;
                    }
                }
                None => {
                    for header in self.iter.by_ref() {
                        let header_name: &hyper::header::HeaderName = header.0;
                        if header_name.as_str().to_lowercase() == "cookie" {
                            let header_value: &hyper::header::HeaderValue = header.1;
                            let header_value = match header_value.to_str() {
                                Ok(header_value) => header_value,
                                Err(e) => return Some(Err(CookieParseError::ToStr(e))),
                            };

                            self.split_cookies =
                                Some(cookie::Cookie::split_parse_encoded(header_value));
                            break;
                        }
                    }

                    if self.split_cookies.is_none() {
                        break None;
                    }
                }
            }
        }
    }
}

pub fn cookies_iter(headers: &hyper::HeaderMap) -> CookieIter<'_> {
    CookieIter {
        iter: headers.iter(),
        split_cookies: None,
    }
}

pub struct SetCookieIter<'a> {
    iter: hyper::header::Iter<'a, hyper::header::HeaderValue>,
}

impl<'a> Iterator for SetCookieIter<'a> {
    type Item = Result<cookie::Cookie<'a>, CookieParseError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for header in self.iter.by_ref() {
            let header_name: &hyper::header::HeaderName = header.0;
            if header_name.as_str().to_lowercase() == "set-cookie" {
                let header_value: &hyper::header::HeaderValue = header.1;
                let header_value = match header_value.to_str() {
                    Ok(header_value) => header_value,
                    Err(e) => return Some(Err(CookieParseError::ToStr(e))),
                };

                return Some(
                    cookie::Cookie::parse_encoded(header_value)
                        .map_err(CookieParseError::CookieParseError),
                );
            }
        }

        None
    }
}

pub fn set_cookies_iter(headers: &hyper::HeaderMap) -> SetCookieIter<'_> {
    SetCookieIter {
        iter: headers.iter(),
    }
}

pub fn is_cookie_expired_by_date(cookie: &cookie::Cookie<'_>) -> bool {
    if let Some(date_time) = cookie.expires_datetime() {
        let now = std::time::SystemTime::now();
        return date_time < now;
    }

    false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_over_cookies() {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            "beginning",
            hyper::header::HeaderValue::from_str("name=value").unwrap(),
        );
        headers.append(
            "Cookie",
            hyper::header::HeaderValue::from_str("name0=value0; name1=value1; name0=value2")
                .unwrap(),
        );
        headers.append(
            "middle",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );
        headers.append(
            "middle",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );
        headers.append(
            "cOOKIE",
            hyper::header::HeaderValue::from_str("name2=value3; name0=value4").unwrap(),
        );
        headers.append(
            "middle",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );
        headers.append(
            "middle",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );
        headers.append(
            "cookie",
            hyper::header::HeaderValue::from_str("name3=value5").unwrap(),
        );
        headers.append(
            "end",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );

        let mut iter = cookies_iter(&headers);

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name0");
        assert_eq!(cookie.value(), "value0");

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name1");
        assert_eq!(cookie.value(), "value1");

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name0");
        assert_eq!(cookie.value(), "value2");

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name2");
        assert_eq!(cookie.value(), "value3");

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name0");
        assert_eq!(cookie.value(), "value4");

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name3");
        assert_eq!(cookie.value(), "value5");

        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_over_set_cookies() {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            "beginning",
            hyper::header::HeaderValue::from_str("name=value").unwrap(),
        );
        headers.append(
            "Set-Cookie",
            hyper::header::HeaderValue::from_str("name0=value0").unwrap(),
        );
        headers.append(
            "middle",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );
        headers.append(
            "middle",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );
        headers.append(
            "set-cOOKIE",
            hyper::header::HeaderValue::from_str("name1=value1").unwrap(),
        );
        headers.append(
            "middle",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );
        headers.append(
            "middle",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );
        headers.append(
            "set-cookie",
            hyper::header::HeaderValue::from_str("name2=value2").unwrap(),
        );
        headers.append(
            "end",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );

        let mut iter = set_cookies_iter(&headers);

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name0");
        assert_eq!(cookie.value(), "value0");

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name1");
        assert_eq!(cookie.value(), "value1");

        let cookie = iter.next().unwrap().unwrap();
        assert_eq!(cookie.name(), "name2");
        assert_eq!(cookie.value(), "value2");

        assert!(iter.next().is_none());
    }
}
