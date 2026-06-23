use cookie::Cookie;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_foundation::{
    NSDate, NSHTTPCookie, NSHTTPCookieDomain, NSHTTPCookieExpires, NSHTTPCookieName,
    NSHTTPCookiePath, NSHTTPCookieSameSiteLax, NSHTTPCookieSameSiteStrict, NSHTTPCookieSecure,
    NSHTTPCookieValue, NSMutableDictionary, NSNumber, NSString,
};

use crate::{Error, Result, catch, from_nsstring};

pub fn cookie_from_ns(c: &NSHTTPCookie) -> Result<Cookie<'static>> {
    catch(|| {
        let name = c.name();
        let value = c.value();
        let domain = c.domain();
        let path = c.path();
        let secure = c.isSecure();
        let http_only = c.isHTTPOnly();
        let expires_date = c.expiresDate();
        let mut builder = Cookie::build((from_nsstring(&name), from_nsstring(&value)))
            .domain(from_nsstring(&domain))
            .path(from_nsstring(&path))
            .secure(secure)
            .http_only(http_only);
        if let Some(expires_date) = expires_date {
            let expires = expires_date.timeIntervalSince1970();
            builder = builder.expires(time::OffsetDateTime::from_unix_timestamp(expires as i64)?);
        }
        if c.isSessionOnly() {
            builder = builder.expires(cookie::Expiration::Session);
        }
        if let Some(s) = c.sameSitePolicy() {
            if s.isEqualToString(unsafe { NSHTTPCookieSameSiteLax }) {
                builder = builder.same_site(cookie::SameSite::Lax);
            } else if s.isEqualToString(unsafe { NSHTTPCookieSameSiteStrict }) {
                builder = builder.same_site(cookie::SameSite::Strict);
            }
        }
        Ok(builder.build())
    })
    .flatten()
}

pub fn cookie_to_ns(c: &Cookie<'_>) -> Result<Retained<NSHTTPCookie>> {
    catch(|| unsafe {
        let properties = NSMutableDictionary::<NSString>::new();
        properties.setObject_forKey(
            &NSString::from_str(c.name()),
            ProtocolObject::from_ref(NSHTTPCookieName),
        );
        properties.setObject_forKey(
            &NSString::from_str(c.value()),
            ProtocolObject::from_ref(NSHTTPCookieValue),
        );
        if let Some(domain) = c.domain() {
            properties.setObject_forKey(
                &NSString::from_str(domain),
                ProtocolObject::from_ref(NSHTTPCookieDomain),
            );
        }
        if let Some(path) = c.path() {
            properties.setObject_forKey(
                &NSString::from_str(path),
                ProtocolObject::from_ref(NSHTTPCookiePath),
            );
        }
        if let Some(cookie::Expiration::DateTime(expires)) = c.expires() {
            let expires_date =
                NSDate::dateWithTimeIntervalSince1970(expires.unix_timestamp() as f64);
            properties
                .setObject_forKey(&expires_date, ProtocolObject::from_ref(NSHTTPCookieExpires));
        }
        if let Some(secure) = c.secure() {
            properties.setObject_forKey(
                &NSNumber::numberWithBool(secure),
                ProtocolObject::from_ref(NSHTTPCookieSecure),
            );
        }
        if let Some(same_site) = c.same_site()
            && !matches!(same_site, cookie::SameSite::None)
        {
            let same_site_str = match same_site {
                cookie::SameSite::Lax => NSHTTPCookieSameSiteLax,
                cookie::SameSite::Strict => NSHTTPCookieSameSiteStrict,
                _ => unreachable!(),
            };
            properties.setObject_forKey(
                same_site_str,
                ProtocolObject::from_ref(NSHTTPCookieSameSiteLax),
            );
        }
        let cookie = NSHTTPCookie::cookieWithProperties(&properties);
        cookie.ok_or(Error::NullPointer)
    })
    .flatten()
}
