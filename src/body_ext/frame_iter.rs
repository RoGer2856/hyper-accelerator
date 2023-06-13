use std::{future::Future, pin::Pin};

pub struct FrameIter<BodyType, FrameDataType>
where
    FrameDataType: hyper::body::Buf,
    BodyType: hyper::body::Body<Data = FrameDataType> + Unpin,
{
    pub(super) body: BodyType,
}

impl<BodyType, FrameDataType> FrameIter<BodyType, FrameDataType>
where
    FrameDataType: hyper::body::Buf,
    BodyType: hyper::body::Body<Data = FrameDataType> + Unpin,
{
    pub fn next_frame(&mut self) -> FrameFuture<'_, BodyType, FrameDataType> {
        FrameFuture {
            body: Pin::new(&mut self.body),
        }
    }
}

pub struct FrameFuture<'a, BodyType, FrameDataType>
where
    FrameDataType: hyper::body::Buf,
    BodyType: hyper::body::Body<Data = FrameDataType> + Unpin,
{
    body: Pin<&'a mut BodyType>,
}

impl<'a, BodyType, FrameDataType, BodyErrorType> Future for FrameFuture<'a, BodyType, FrameDataType>
where
    FrameDataType: hyper::body::Buf,
    BodyType: hyper::body::Body<Data = FrameDataType, Error = BodyErrorType> + Unpin,
{
    type Output = Option<Result<hyper::body::Frame<FrameDataType>, BodyErrorType>>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut self_mut = self.as_mut();

        let frame = match self_mut.body.as_mut().poll_frame(cx) {
            std::task::Poll::Pending => return std::task::Poll::Pending,
            std::task::Poll::Ready(frame) => frame,
        };

        if let Some(frame) = frame {
            match frame {
                Ok(frame) => std::task::Poll::Ready(Some(Ok(frame))),
                Err(e) => std::task::Poll::Ready(Some(Err(e))),
            }
        } else {
            std::task::Poll::Ready(None)
        }
    }
}
