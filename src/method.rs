use self::Inner::*;

/// The Request Method
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Method(Inner);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Inner {
    Get,
    Post,
    Head,
    Put,
    Patch,
    Delete,
    Options,
    Connect,
    Trace,
}

impl Method {
    /// GET
    pub const GET: Method = Method(Get);

    /// POST
    pub const POST: Method = Method(Post);

    /// HEAD
    pub const HEAD: Method = Method(Head);

    /// PUT
    pub const PUT: Method = Method(Put);

    /// PATCH
    pub const PATCH: Method = Method(Patch);

    /// DELETE
    pub const DELETE: Method = Method(Delete);

    /// OPTIONS
    pub const OPTIONS: Method = Method(Options);

    /// CONNECT
    pub const CONNECT: Method = Method(Connect);

    /// TRACE
    pub const TRACE: Method = Method(Trace);

    /// Return a &str representation of the HTTP method
    pub fn as_str(&self) -> &str {
        match self.0 {
            Get => "GET",
            Post => "POST",
            Head => "HEAD",
            Put => "PUT",
            Patch => "PATCH",
            Delete => "DELETE",
            Options => "OPTIONS",
            Connect => "CONNECT",
            Trace => "TRACE",
        }
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "HEAD" => Method::HEAD,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            "OPTIONS" => Method::OPTIONS,
            "CONNECT" => Method::CONNECT,
            "TRACE" => Method::TRACE,
            _ => Method::default(),
        }
    }
}

impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}
