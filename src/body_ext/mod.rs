use self::collect::{Collect, CollectFuture};

pub mod collect;

pub trait BodyExt<FrameDataType: hyper::body::Buf + Unpin>:
    hyper::body::Body<Data = FrameDataType> + Sized + Unpin
{
    fn collect(self) -> CollectFuture<Self, FrameDataType> {
        CollectFuture {
            body: self,
            collect: Collect {
                received_frames: Vec::new(),
            },
        }
    }
}

impl<
        FrameDataType: hyper::body::Buf + Unpin,
        BodyType: hyper::body::Body<Data = FrameDataType> + Unpin,
    > BodyExt<FrameDataType> for BodyType
{
}
