/*
#[derive(Clone)]
pub struct LayerAPI;

impl<S> Layer<S> for LayerAPI {
    type Service = MiddlewareAPI<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MiddlewareAPI { inner }
    }
}

#[derive(Clone)]
struct MiddlewareAPI<S> {
    inner: S,
}

impl<S> Service<Request> for MiddlewareAPI<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
*/
