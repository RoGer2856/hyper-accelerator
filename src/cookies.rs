#[derive(Copy, Clone)]
pub enum CookieType {
    SetCookie,
    Cookie,
}

pub struct CookieIter<'a> {
    iter: hyper::header::Iter<'a, hyper::header::HeaderValue>,
    cookie_type: CookieType,
}

impl<'a> Iterator for CookieIter<'a> {
    type Item = cookie::Cookie<'a>;

    #[inline]
    fn next(&mut self) -> Option<cookie::Cookie<'a>> {
        for header in self.iter.by_ref() {
            let header_name: &hyper::header::HeaderName = header.0;
            let header_value: &hyper::header::HeaderValue = header.1;
            if let Some(cookie) =
                convert_header_to_cookie(self.cookie_type, header_name, header_value)
            {
                return Some(cookie);
            }
        }

        None
    }
}

pub fn cookies_iter(cookie_type: CookieType, headers: &hyper::HeaderMap) -> CookieIter<'_> {
    CookieIter {
        iter: headers.iter(),
        cookie_type,
    }
}

pub fn is_cookie_expired_by_date(cookie: &cookie::Cookie<'_>) -> bool {
    if let Some(date_time) = cookie.expires_datetime() {
        let now = std::time::SystemTime::now();
        return date_time < now;
    }

    false
}

pub fn remove_cookies_from_headers(
    cookie_type: CookieType,
    cookie_name: &str,
    headers: hyper::HeaderMap,
) -> hyper::HeaderMap {
    let mut ret = hyper::HeaderMap::new();

    for (header_name, header_value) in headers.iter() {
        let mut should_keep = true;
        if let Some(cookie) = convert_header_to_cookie(cookie_type, header_name, header_value) {
            if cookie.name().to_lowercase() == cookie_name {
                should_keep = false
            }
        }

        if should_keep {
            ret.append(header_name.clone(), header_value.clone());
        }
    }

    ret
}

fn convert_header_to_cookie<'a>(
    cookie_type: CookieType,
    header_name: &'a hyper::header::HeaderName,
    header_value: &'a hyper::header::HeaderValue,
) -> Option<cookie::Cookie<'a>> {
    let cookie_header_name = match cookie_type {
        CookieType::SetCookie => "set-cookie",
        CookieType::Cookie => "cookie",
    };

    if cookie_header_name == header_name.as_str().to_lowercase() {
        if let Ok(header_value) = header_value.to_str() {
            if let Ok(cookie) = cookie::Cookie::parse(header_value) {
                return Some(cookie);
            }
        }
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cookies_iter_only_cookie_header() {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            "Cookie",
            hyper::header::HeaderValue::from_str("name0=value0").unwrap(),
        );
        headers.append(
            "cOOKIE",
            hyper::header::HeaderValue::from_str("name1=value1").unwrap(),
        );

        let mut iter = cookies_iter(CookieType::Cookie, &headers);

        let cookie = iter.next().unwrap();
        assert_eq!(cookie.name(), "name0");
        assert_eq!(cookie.value(), "value0");

        let cookie = iter.next().unwrap();
        assert_eq!(cookie.name(), "name1");
        assert_eq!(cookie.value(), "value1");

        assert!(iter.next().is_none());
    }

    #[test]
    fn cookies_iter_mixed_header() {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            "beginning",
            hyper::header::HeaderValue::from_str("name=value").unwrap(),
        );
        headers.append(
            "COOKIE",
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
            "COOKIE",
            hyper::header::HeaderValue::from_str("name1=value1").unwrap(),
        );
        headers.append(
            "end",
            hyper::header::HeaderValue::from_str("foobar").unwrap(),
        );

        let mut iter = cookies_iter(CookieType::Cookie, &headers);

        let cookie = iter.next().unwrap();
        assert_eq!(cookie.name(), "name0");
        assert_eq!(cookie.value(), "value0");

        let cookie = iter.next().unwrap();
        assert_eq!(cookie.name(), "name1");
        assert_eq!(cookie.value(), "value1");

        assert!(iter.next().is_none());
    }

    #[test]
    fn removing_cookies() {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            "cookie",
            hyper::header::HeaderValue::from_str("name0=value0").unwrap(),
        );
        headers.append(
            "cookie",
            hyper::header::HeaderValue::from_str("name1=value2").unwrap(),
        );
        headers.append(
            "cookie",
            hyper::header::HeaderValue::from_str("name1=value3").unwrap(),
        );
        headers.append(
            "cookie",
            hyper::header::HeaderValue::from_str("name2=value4").unwrap(),
        );

        let headers = remove_cookies_from_headers(CookieType::Cookie, "name1", headers);

        assert_eq!(headers.len(), 2);
        assert!(cookies_iter(CookieType::Cookie, &headers)
            .find(|cookie| cookie.name() == "name0" && cookie.value() == "value0")
            .is_some());
        assert!(cookies_iter(CookieType::Cookie, &headers)
            .find(|cookie| cookie.name() == "name2" && cookie.value() == "value4")
            .is_some());
    }
}
