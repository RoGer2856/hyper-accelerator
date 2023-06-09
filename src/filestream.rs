#![allow(clippy::uninit_vec)]

use std::path::Path;

use tokio::io::AsyncReadExt;

use crate::{error::Error, response_body::AsyncStream};

pub struct FileStream {
    file: tokio::fs::File,
}

impl FileStream {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let file = tokio::fs::File::open(path).await?;
        Ok(Self { file })
    }
}

impl AsyncStream<Vec<u8>> for FileStream {
    fn next<'a>(
        &'a mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>, Error>> + Send + Sync + 'a>,
    > {
        Box::pin(async move {
            const BUF_SIZE: usize = 8192;
            let mut buffer = Vec::with_capacity(BUF_SIZE);
            unsafe {
                buffer.set_len(BUF_SIZE);
            }

            let size = self.file.read(&mut buffer).await?;

            if size != 0 {
                unsafe {
                    buffer.set_len(size);
                }

                Ok(Some(buffer))
            } else {
                Ok(None)
            }
        })
    }
}
