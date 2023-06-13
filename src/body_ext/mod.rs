use self::{
    collect::{Collect, CollectFuture},
    frame_iter::FrameIter,
};

pub mod collect;
pub mod frame_iter;

pub trait BodyExt<FrameDataType: hyper::body::Buf + Unpin>:
    hyper::body::Body<Data = FrameDataType> + Sized + Unpin
{
    fn collect(self) -> CollectFuture<Self, FrameDataType> {
        CollectFuture {
            body: self,
            collect: Collect::new(),
        }
    }

    fn frame_iter(self) -> FrameIter<Self, FrameDataType> {
        FrameIter { body: self }
    }
}

impl<
        FrameDataType: hyper::body::Buf + Unpin,
        BodyType: hyper::body::Body<Data = FrameDataType> + Unpin,
    > BodyExt<FrameDataType> for BodyType
{
}
