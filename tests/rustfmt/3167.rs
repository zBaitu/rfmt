// http://tools.ietf.org/html/rfc6265#section-5.1.4
pub fn path_match(request_path: &str, cookie_path: &str) -> bool {
     // A request-path path-matches a given cookie-path if at least one of
     // the following conditions holds:
     // The cookie-path and the request-path are identical.
     request_path == cookie_path ||

     (request_path.starts_with(cookie_path) && (
         // The cookie-path is a prefix of the request-path, and the last
         // character of the cookie-path is %x2F ("/").
         cookie_path.ends_with("/") ||
         // The cookie-path is a prefix of the request-path, and the first
         // character of the request-path that is not included in the cookie-
         // path is a %x2F ("/") character.
         request_path[cookie_path.len()..].starts_with("/")
     ))
}
