use std::{future::Future, pin::Pin};

use crate::error::Error;

pub trait AsyncStream<ItemType>: 'static + Unpin + Send + Sync {
    fn next<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<ItemType>, Error>> + Send + Sync + 'a>>;
}

pub enum ResponseBody {
    None,
    Str(Option<&'static str>),
    String(Option<String>),
    Bytes(Option<Vec<u8>>),
    AsyncBytesStream(Box<dyn AsyncStream<Vec<u8>>>),
}

impl Default for ResponseBody {
    fn default() -> Self {
        Self::None
    }
}

impl ResponseBody {
    pub async fn read_all(&mut self) -> Result<Vec<u8>, Error> {
        let mut ret = Vec::new();
        while let Some(mut chunk) = self.read_next_chunk().await? {
            ret.append(&mut chunk);
        }
        Ok(ret)
    }

    pub async fn read_next_chunk(&mut self) -> Result<Option<Vec<u8>>, Error> {
        match self {
            ResponseBody::None => Ok(Some(Vec::new())),
            ResponseBody::Str(data) => Ok(data.take().map(|data| data.into())),
            ResponseBody::String(data) => Ok(data.take().map(|data| data.as_str().into())),
            ResponseBody::Bytes(data) => Ok(data.take()),
            ResponseBody::AsyncBytesStream(data) => data.next().await,
        }
    }
}

impl hyper::body::Body for ResponseBody {
    type Data = hyper::body::Bytes;
    type Error = Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            ResponseBody::None => std::task::Poll::Ready(None),
            ResponseBody::Str(data) => std::task::Poll::Ready(
                data.take()
                    .map(|data| Ok(hyper::body::Frame::data(hyper::body::Bytes::from(data)))),
            ),
            ResponseBody::String(data) => std::task::Poll::Ready(
                data.take()
                    .map(|data| Ok(hyper::body::Frame::data(hyper::body::Bytes::from(data)))),
            ),
            ResponseBody::Bytes(data) => std::task::Poll::Ready(
                data.take()
                    .map(|data| Ok(hyper::body::Frame::data(hyper::body::Bytes::from(data)))),
            ),
            ResponseBody::AsyncBytesStream(data) => {
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
            ResponseBody::None => hyper::body::SizeHint::with_exact(0),
            ResponseBody::Str(data) => data.map_or_else(hyper::body::SizeHint::default, |data| {
                hyper::body::SizeHint::with_exact(data.len() as u64)
            }),
            ResponseBody::String(data) => data
                .as_ref()
                .map_or_else(hyper::body::SizeHint::default, |data| {
                    hyper::body::SizeHint::with_exact(data.len() as u64)
                }),
            ResponseBody::Bytes(data) => data
                .as_ref()
                .map_or_else(hyper::body::SizeHint::default, |data| {
                    hyper::body::SizeHint::with_exact(data.len() as u64)
                }),
            ResponseBody::AsyncBytesStream(_data) => hyper::body::SizeHint::default(),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            ResponseBody::None => true,
            ResponseBody::Str(data) => data.is_none(),
            ResponseBody::String(data) => data.is_none(),
            ResponseBody::Bytes(data) => data.is_none(),
            ResponseBody::AsyncBytesStream(_data) => false,
        }
    }
}

impl From<&'static str> for ResponseBody {
    fn from(value: &'static str) -> Self {
        Self::Str(Some(value))
    }
}

impl From<String> for ResponseBody {
    fn from(value: String) -> Self {
        Self::String(Some(value))
    }
}

impl From<Vec<u8>> for ResponseBody {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(Some(value))
    }
}

impl<T: AsyncStream<Vec<u8>>> From<T> for ResponseBody {
    fn from(value: T) -> Self {
        Self::AsyncBytesStream(Box::new(value))
    }
}
