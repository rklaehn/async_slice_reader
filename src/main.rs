use tokio::io::{AsyncRead, AsyncSeek, AsyncReadExt, AsyncSeekExt};
use futures::{Future, FutureExt};
use std::io;
use std::io::{SeekFrom, Cursor};
use std::pin::Pin;

/// A reader that can read a slice at a specified offset
///
/// For a file, this will be implemented by seeking to the offset and then reading the data.
/// For other types of storage, seeking is not necessary. E.g. a Bytes or a memory mapped
/// slice already allows random access.
///
/// For external storage such as S3/R2, this might be implemented in terms of async http requests.
///
/// This is similar to the io interface of sqlite.
/// See xRead, xFileSize in https://www.sqlite.org/c3ref/io_methods.html
#[allow(clippy::len_without_is_empty)]
pub trait AsyncSliceReader {
    type ReadResult<'r>: Future<Output = io::Result<()>> + 'r where Self: 'r;
    fn read<'a, 'b, 'r>(
        &'a mut self,
        offset: u64,
        buf: &'b mut [u8],
    ) -> Self::ReadResult<'r>
    where
        Self: 'r,
        'a: 'r,
        'b: 'r;
    fn len<'a, 'r>(&'a mut self) -> Pin<Box<dyn Future<Output = io::Result<u64>> + Send + 'r>>
    where
        Self: 'r,
        'a: 'r;
}

impl<R: AsyncRead + AsyncSeek + Unpin + Send + Sync> AsyncSliceReader for R {
    type ReadResult<'r> = Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'r>> where Self: 'r;
    fn read<'a, 'b, 'r>(
        &'a mut self,
        offset: u64,
        buf: &'b mut [u8],
    ) -> Self::ReadResult<'r>
    where
        Self: 'r,
        'a: 'r,
        'b: 'r,
    {
        async move {
            self.seek(SeekFrom::Start(offset)).await?;
            self.read_exact(buf).await?;
            Ok(())
        }
        .boxed()
    }

    fn len<'a, 'r>(&'a mut self) -> Pin<Box<dyn Future<Output = io::Result<u64>> + Send + 'r>>
    where
        Self: 'r,
        'a: 'r,
    {
        async move { self.seek(SeekFrom::End(0)).await }.boxed()
    }
}

async fn test(mut read: impl AsyncSliceReader) -> io::Result<()> {
    let mut buffer = [0u8; 1000];
    for i in 0..100 {
        read.read(i, &mut buffer).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let data = Vec::new();
    tokio::task::spawn(async move {
        test(&mut Cursor::new(data)).await.unwrap();
    });
    
    println!("hi");
    Ok(())
}