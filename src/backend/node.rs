use hyper::Uri;

#[derive(Debug)]
pub struct Node {
    uri: Uri,
}

impl Node {
    pub fn new(uri: Uri) -> Self {
        Node{uri}
    }

    pub fn from_str(uri: &str) -> Result<Self, hyper::http::uri::InvalidUri> {
        let uri = uri.parse::<Uri>()?;
        Ok(Self { uri })
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }
}