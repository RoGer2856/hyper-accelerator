use std::{future::Future, pin::Pin};

use crate::error::Error;

pub trait AsyncStream<ItemType>: 'static + Unpin + Send + Sync {
    fn next<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<ItemType>, Error>> + Send + Sync + 'a>>;
}

pub enum Body {
    None,
    Str(Option<&'static str>),
    String(Option<String>),
    Bytes(Option<Vec<u8>>),
    AsyncBytesStream(Box<dyn AsyncStream<Vec<u8>>>),
}

impl Default for Body {
    fn default() -> Self {
        Self::None
    }
}

impl Body {
    pub async fn read_all(&mut self) -> Result<Vec<u8>, Error> {
        let mut ret = Vec::new();
        while let Some(mut chunk) = self.read_next_chunk().await? {
            ret.append(&mut chunk);
        }
        Ok(ret)
    }

    pub async fn read_next_chunk(&mut self) -> Result<Option<Vec<u8>>, Error> {
        match self {
            Body::None => Ok(Some(Vec::new())),
            Body::Str(data) => Ok(data.take().map(|data| data.into())),
            Body::String(data) => Ok(data.take().map(|data| data.as_str().into())),
            Body::Bytes(data) => Ok(data.take()),
            Body::AsyncBytesStream(data) => data.next().await,
        }
    }
}

impl hyper::body::Body for Body {
    type Data = hyper::body::Bytes;
    type Error = Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            Body::None => std::task::Poll::Ready(None),
            Body::Str(data) => std::task::Poll::Ready(
                data.take()
                    .map(|data| Ok(hyper::body::Frame::data(hyper::body::Bytes::from(data)))),
            ),
            Body::String(data) => std::task::Poll::Ready(
                data.take()
                    .map(|data| Ok(hyper::body::Frame::data(hyper::body::Bytes::from(data)))),
            ),
            Body::Bytes(data) => std::task::Poll::Ready(
                data.take()
                    .map(|data| Ok(hyper::body::Frame::data(hyper::body::Bytes::from(data)))),
            ),
            Body::AsyncBytesStream(data) => {
                let mut data = data.next();
                let next_chunk_fut = Pin::new(&mut data);
                Pin::poll(next_chunk_fut, cx).map(|data| {
                    data.transpose().map(|data| {
                        data.map(|data| hyper::body::Frame::data(hyper::body::Bytes::from(data)))
                    })
                })
            }
        }
    }

    fn size_hint(&self) -> hyper::body::SizeHint {
        match self {
            Body::None => hyper::body::SizeHint::with_exact(0),
            Body::Str(data) => data.map_or_else(hyper::body::SizeHint::default, |data| {
                hyper::body::SizeHint::with_exact(data.len() as u64)
            }),
            Body::String(data) => data
                .as_ref()
                .map_or_else(hyper::body::SizeHint::default, |data| {
                    hyper::body::SizeHint::with_exact(data.len() as u64)
                }),
            Body::Bytes(data) => data
                .as_ref()
                .map_or_else(hyper::body::SizeHint::default, |data| {
                    hyper::body::SizeHint::with_exact(data.len() as u64)
                }),
            Body::AsyncBytesStream(_data) => hyper::body::SizeHint::default(),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Body::None => true,
            Body::Str(data) => data.is_none(),
            Body::String(data) => data.is_none(),
            Body::Bytes(data) => data.is_none(),
            Body::AsyncBytesStream(_data) => false,
        }
    }
}

impl From<&'static str> for Body {
    fn from(value: &'static str) -> Self {
        Self::Str(Some(value))
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Self::String(Some(value))
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(Some(value))
    }
}

impl<T: AsyncStream<Vec<u8>>> From<T> for Body {
    fn from(value: T) -> Self {
        Self::AsyncBytesStream(Box::new(value))
    }
}
