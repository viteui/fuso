pub mod ext;
pub mod r#macro;

use std::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

pub type BoxedFuture<'lifetime, T> = Pin<Box<dyn Future<Output = T> + 'lifetime>>;

#[cfg(feature = "fuso-rt-smol")]
mod io {
    pub use smol::io::{AsyncRead, AsyncWrite};
}

#[cfg(feature = "fuso-rt-custom")]
mod io {
    pub use futures::io::{AsyncRead, AsyncWrite};
}

#[cfg(feature = "fuso-rt-tokio")]
pub struct ReadBuf<'a> {
    buf: tokio::io::ReadBuf<'a>,
}

#[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
pub struct ReadBuf<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

pub trait AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<crate::Result<usize>>;
}

pub trait AsyncWrite {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<crate::Result<usize>>;

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<crate::Result<()>>;

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<crate::Result<()>>;
}

#[cfg(feature = "fuso-rt-tokio")]
impl<T> AsyncWrite for T
where
    T: tokio::io::AsyncWrite,
{
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<crate::Result<usize>> {
        tokio::io::AsyncWrite::poll_write(self, cx, buf).map_err(Into::into)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<crate::Result<()>> {
        tokio::io::AsyncWrite::poll_flush(self, cx).map_err(Into::into)
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<crate::Result<()>> {
        tokio::io::AsyncWrite::poll_shutdown(self, cx).map_err(Into::into)
    }
}

#[cfg(feature = "fuso-rt-tokio")]
impl<T> AsyncRead for T
where
    T: tokio::io::AsyncRead,
{
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<crate::Result<usize>> {
        match tokio::io::AsyncRead::poll_read(self, cx, &mut buf.buf) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(buf.filled().len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
impl<T> AsyncWrite for T
where
    T: io::AsyncWrite,
{
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<crate::Result<usize>> {
        futures::AsyncWrite::poll_write(self, cx, buf).map_err(Into::into)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<crate::Result<()>> {
        futures::AsyncWrite::poll_flush(self, cx).map_err(Into::into)
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<crate::Result<()>> {
        futures::AsyncWrite::poll_close(self, cx).map_err(Into::into)
    }
}

#[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
impl<T> AsyncRead for T
where
    T: io::AsyncRead,
{
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<crate::Result<usize>> {
        match futures::AsyncRead::poll_read(self, cx, &mut buf.buf[buf.offset..])? {
            Poll::Pending => Poll::Pending,
            Poll::Ready(n) => {
                buf.offset += n;
                Poll::Ready(Ok(n))
            }
        }
    }
}

impl<'a> ReadBuf<'a> {
    #[cfg(feature = "fuso-rt-tokio")]
    pub fn new(buf: tokio::io::ReadBuf<'a>) -> Self {
        Self { buf }
    }

    #[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    #[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
    pub fn remaining(&self) -> usize {
        self.len() - self.offset
    }

    #[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
    pub fn position(&self) -> usize {
        self.offset
    }

    #[cfg(feature = "fuso-rt-tokio")]
    pub fn position(&self) -> usize {
        self.buf.filled().len()
    }

    #[cfg(feature = "fuso-rt-tokio")]
    pub fn iter_mut(&mut self) -> &mut [u8] {
        self.buf.initialized_mut()
    }

    #[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
    pub fn iter_mut(&mut self) -> &mut [u8] {
        &mut self.buf
    }

    #[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
    pub fn advance(&mut self, n: usize) {
        assert!(self.offset + n <= self.buf.len());
        self.offset += n;
    }
}

#[cfg(any(feature = "fuso-rt-smol", feature = "fuso-rt-custom"))]
impl<'a> Deref for ReadBuf<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buf
    }
}

#[cfg(feature = "fuso-rt-tokio")]
impl<'a> Deref for ReadBuf<'a> {
    type Target = tokio::io::ReadBuf<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<'a> DerefMut for ReadBuf<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}