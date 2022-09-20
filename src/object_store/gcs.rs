//! ObjectStore implementation for the Google Cloud Storage API

use std::io::{ErrorKind, Read};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Buf;
use futures::{stream, AsyncRead, StreamExt};

use datafusion_data_access::object_store::{
    FileMetaStream, ListEntryStream, ObjectReader, ObjectStore,
};
use datafusion_data_access::{FileMeta, Result, SizedFile};

use cloud_storage::client::Client;

use crate::error::GCSError;

async fn new_client() -> Client {
    Client::new()
}

/// `ObjectStore` implementation for the Google Cloud Storage API
#[derive(Debug)]
pub struct GCSFileSystem {
    client: Client,
}

impl GCSFileSystem {
    /// Create new `ObjectStore`
    pub async fn new() -> Self {
        Self {
            client: new_client().await,
        }
    }
}

#[async_trait]
impl ObjectStore for GCSFileSystem {
    async fn list_file(&self, uri: &str) -> Result<FileMetaStream> {
        let (_, prefix) = uri.split_once("gcs://").ok_or_else(|| {
            std::io::Error::new(ErrorKind::Other, GCSError::GCS("No s3 scheme found".into()))
        })?;
        let (bucket, prefix) = match prefix.split_once('/') {
            Some((bucket, prefix)) => (bucket.to_owned(), prefix),
            None => (prefix.to_owned(), ""),
        };

        let mut list_request = cloud_storage::object::ListRequest::default();
        list_request.prefix = Some(prefix.to_string());
        let objects = self
            .client
            .object()
            .list(&bucket, list_request)
            .await
            .map_err(|err| {
                std::io::Error::new(ErrorKind::Other, GCSError::GCS(format!("{:?}", err)))
            })?
            .flat_map(|r| {
                let object = r.unwrap_or_default();
                stream::iter(object.items.into_iter().map(|o| {
                    Ok::<FileMeta, std::io::Error>(FileMeta {
                        sized_file: SizedFile {
                            path: format!("{}/{}", &bucket, o.name),
                            size: o.size,
                        },
                        last_modified: Some(o.updated),
                    })
                }))
            })
            .collect::<Vec<Result<FileMeta>>>()
            .await;

        //Ok(Box::<impl Stream<Item = Result<FileMeta, std::io::Error>>>::pin(objects))
        Ok(Box::pin(stream::iter(objects)))
    }

    async fn list_dir(&self, _prefix: &str, _delimiter: Option<String>) -> Result<ListEntryStream> {
        todo!()
    }

    fn file_reader(&self, file: SizedFile) -> Result<Arc<dyn ObjectReader>> {
        Ok(Arc::new(GCSFileReader::new(file)?))
    }
}

#[allow(dead_code)]
impl GCSFileSystem {
    /// Convenience wrapper for creating a new `GCSFileSystem` using default configuration options.  Only works with AWS.
    pub async fn default() -> Self {
        GCSFileSystem::new().await
    }
}

struct GCSFileReader {
    file: SizedFile,
}

impl GCSFileReader {
    #[allow(clippy::too_many_arguments)]
    fn new(file: SizedFile) -> Result<Self> {
        Ok(Self { file })
    }
}

#[async_trait]
impl ObjectReader for GCSFileReader {
    async fn chunk_reader(&self, _start: u64, _length: usize) -> Result<Box<dyn AsyncRead>> {
        todo!("implement once async file readers are available (arrow-rs#78, arrow-rs#111)")
    }

    fn sync_chunk_reader(&self, start: u64, length: usize) -> Result<Box<dyn Read + Send + Sync>> {
        let file_path = self.file.path.clone();

        // once the async chunk file readers have been implemented this complexity can be removed
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                // aws_sdk_s3::Client appears bound to the runtime and will deadlock if cloned from the main runtime
                let client = new_client().await;

                let (bucket, key) = match file_path.split_once('/') {
                    Some((bucket, prefix)) => (bucket, prefix),
                    None => (file_path.as_str(), ""),
                };

                let resp = if length > 0 {
                    client
                        .object()
                        .download_range(bucket, key, start, length)
                        .await
                } else {
                    client.object().download(bucket, key).await
                };

                let bytes = match resp {
                    Ok(res) => Ok(bytes::Bytes::from(res)),
                    Err(err) => Err(std::io::Error::new(
                        ErrorKind::Other,
                        GCSError::GCS(format!("{:?}", err)),
                    )),
                };

                tx.send(bytes).unwrap();
            })
        });

        let bytes = rx.recv_timeout(Duration::from_secs(10)).map_err(|err| {
            std::io::Error::new(ErrorKind::TimedOut, GCSError::GCS(format!("{:?}", err)))
        })??;

        Ok(Box::new(bytes.reader()))
    }

    fn length(&self) -> u64 {
        self.file.size
    }
}
