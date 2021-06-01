use utopia_common::frontend::CoreEvent;

use futures::{stream::{Stream, FusedStream}, task::{Context, Poll}};
use tokio::{net::UnixStream, io::{AsyncRead, ReadBuf, AsyncWrite}};
use serde_json;
use std::pin::Pin;
use std::error::Error;

pub struct SocketStream {
    inner: UnixStream,
    terminated: bool
}
impl SocketStream {
	pub async fn new<P>(path: P) -> std::io::Result<Self>
	where
		P: AsRef<std::path::Path>
	{
		Ok(Self {
			inner: UnixStream::connect(path).await?,
			terminated: false
		})
	}
	pub fn from_stream(stream: UnixStream) -> Self {
		Self {
			inner: stream,
			terminated: false
		}
	}
	pub async fn block_writeable(&self) -> std::io::Result<()> {
		self.inner.writable().await?;
		Ok(())
	}
	pub async fn write_s(&self, buf: &[u8]) -> std::io::Result<usize> {
		self.inner.writable().await?;
		self.inner.try_write(buf)
	}
}
impl Stream for SocketStream {
    type Item = Result<CoreEvent, Box<dyn Error>>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = [0; 0xFFFF];
        let mut reader = ReadBuf::new(&mut buf);
        let stream = Pin::new(&mut self.inner);
        match stream.poll_read(cx, &mut reader) {
            Poll::Ready(Ok(())) => {
                match reader.filled().len() {
                    0 => {
                        self.terminated = true;
                        Poll::Ready(None)
                    },
                    _ => {
                        match serde_json::from_slice(reader.filled()) {
                            Ok(action) => Poll::Ready(Some(Ok(action))),
                            Err(e) => Poll::Ready(Some(Err(Box::new(e))))
                        }
                    }
                }
            },
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(Box::new(e)))),
            Poll::Pending => Poll::Pending
        }
    }
}
impl FusedStream for SocketStream {
    fn is_terminated(&self) -> bool {
        self.terminated
    }
}
impl AsyncRead for SocketStream {
	fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<(), std::io::Error>> {
        let stream = Pin::new(&mut self.inner);
        stream.poll_read(cx, buf)
    }
}
impl AsyncWrite for SocketStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        let stream = Pin::new(&mut self.inner);
        stream.poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), std::io::Error>> {
        let stream = Pin::new(&mut self.inner);
        stream.poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), std::io::Error>> {
        let stream = Pin::new(&mut self.inner);
        stream.poll_shutdown(cx)
    }
}
