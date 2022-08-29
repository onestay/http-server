use crate::request::HttpRequest;

pub(crate) trait Middleware {
    fn run(self: &Self, request: &mut HttpRequest) -> bool;
}

pub(crate) struct MiddlewareManager {
    middlewares: Vec<Box<dyn Middleware>>
}

impl MiddlewareManager {
    pub(crate) fn new() -> Self {
        MiddlewareManager { middlewares: Vec::new() }
    }

    pub(crate) fn execute(&self, request: &mut HttpRequest) -> bool {
        for middleware in &self.middlewares {
            if !middleware.run(request) {
                return false;
            }
        }
        true
    }
}