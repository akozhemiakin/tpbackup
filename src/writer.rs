use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    sync::Mutex,
};

#[async_trait]
pub trait Writer: Debug + Unpin {
    async fn write(&mut self, bytes: &[u8]);
}

#[async_trait]
impl<T> Writer for T
where
    T: AsyncWrite + Unpin + Debug + Send,
{
    async fn write(&mut self, bytes: &[u8]) {
        self.write_all(bytes).await.unwrap();
    }
}

#[derive(Debug)]
pub struct SyncWriter<T>
where
    T: AsyncWrite + Unpin + Debug + Send,
{
    out: Arc<Mutex<T>>,
}

impl<T> SyncWriter<T>
where
    T: AsyncWrite + Unpin + Debug + Send,
{
    pub fn new(out: T) -> Self {
        SyncWriter {
            out: Arc::new(Mutex::new(out)),
        }
    }
}

#[async_trait]
impl<T> Writer for SyncWriter<T>
where
    T: AsyncWrite + Unpin + Debug + Send,
{
    async fn write(&mut self, bytes: &[u8]) {
        let mut out = self.out.lock().await;

        out.write_all(bytes).await.unwrap();
    }
}
