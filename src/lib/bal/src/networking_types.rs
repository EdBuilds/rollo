pub enum Method {
    Get,
    Put,
    Post,
    Delete,
}

pub enum Status {
    Ok,
    InternalServerError,
    BadRequest,
}

pub struct Response {
    pub status: Status,
    pub body: String,
}

pub struct Request {
    pub method: Method,
    pub url: String,
    pub body: String,
}
