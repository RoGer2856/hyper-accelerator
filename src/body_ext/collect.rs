pub struct Collect<FrameDataType: hyper::body::Buf + Unpin>
where
    FrameDataType: Sized,
{
    pub(super) received_frames: Vec<hyper::body::Frame<FrameDataType>>,
}

impl<FrameDataType: hyper::body::Buf + Unpin> Collect<FrameDataType> {
    pub fn take(&mut self) -> Self {
        Self {
            received_frames: self.received_frames.drain(..).collect(),
        }
    }

    pub fn aggregate(self) -> (Option<Vec<u8>>, Option<hyper::HeaderMap>) {
        let mut aggregated_data = None;
        let mut aggregated_trailers = None;
        for frame in self.received_frames {
            match frame.into_data() {
                Ok(mut data) => {
                    let aggregated_data = aggregated_data.get_or_insert_with(Vec::new);
                    let old_len = aggregated_data.len();
                    aggregated_data.resize(old_len + data.remaining(), 0);
                    data.copy_to_slice(&mut aggregated_data.as_mut_slice()[old_len..]);
                }
                Err(frame) => {
                    if let Ok(trailers) = frame.into_trailers() {
                        aggregated_trailers
                            .get_or_insert_with(hyper::HeaderMap::new)
                            .extend(trailers.into_iter());
                    }
                }
            }
        }

        (aggregated_data, aggregated_trailers)
    }
}

pub struct CollectFuture<BodyType, FrameDataType>
where
    FrameDataType: hyper::body::Buf + Unpin,
    BodyType: hyper::body::Body<Data = FrameDataType> + Unpin,
{
    pub(super) body: BodyType,
    pub(super) collect: Collect<FrameDataType>,
}

impl<BodyType, FrameDataType> std::future::Future for CollectFuture<BodyType, FrameDataType>
where
    FrameDataType: hyper::body::Buf + Unpin,
    BodyType: hyper::body::Body<Data = FrameDataType> + Unpin,
{
    type Output = Collect<FrameDataType>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut self_mut = self.as_mut();

        loop {
            let mut body = std::pin::Pin::new(&mut self_mut.body);
            let frame = match body.as_mut().poll_frame(cx) {
                std::task::Poll::Pending => return std::task::Poll::Pending,
                std::task::Poll::Ready(frame) => frame,
            };

            if let Some(frame) = frame {
                if let Ok(frame) = frame {
                    self_mut.collect.received_frames.push(frame);
                } else {
                    unreachable!();
                }
            } else {
                return std::task::Poll::Ready(self_mut.collect.take());
            }
        }
    }
}
