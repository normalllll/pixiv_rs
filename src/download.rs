use async_stream::try_stream;
use futures_core::{Stream, TryStream};
use futures_util::TryStreamExt;
use reqwest::{Client, Proxy, Response, header};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_util::sync::CancellationToken;

use crate::error::{PixivError, PixivErrorKind};

#[derive(Debug, Clone)]
pub enum DownloadEvent<T> {
    Progress { received: usize, total: usize },
    Done { output: T },
}

pub type MemoryDownloadStream = DownloadStream<Vec<u8>>;
pub type FileDownloadStream = DownloadStream<PathBuf>;

pub struct DownloadStream<T> {
    cancel_token: CancellationToken,
    inner: Pin<Box<dyn Stream<Item = Result<DownloadEvent<T>, PixivError>> + Send>>,
}

impl<T> DownloadStream<T> {
    fn new(
        cancel_token: CancellationToken,
        inner: Pin<Box<dyn Stream<Item = Result<DownloadEvent<T>, PixivError>> + Send>>,
    ) -> Self {
        Self {
            cancel_token,
            inner,
        }
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}

impl<T> Stream for DownloadStream<T> {
    type Item = Result<DownloadEvent<T>, PixivError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

#[derive(Debug)]
struct ProgressTracker {
    received: usize,
    total: usize,
    part_offset: usize,
    next_threshold: usize,
}

impl ProgressTracker {
    fn new(total: usize) -> Self {
        let part_offset = (total / 50).max(1);

        Self {
            received: 0,
            total,
            part_offset,
            next_threshold: part_offset,
        }
    }

    fn record(&mut self, size: usize) -> bool {
        self.received += size;

        if self.received >= self.next_threshold || self.received == self.total {
            while self.next_threshold <= self.received {
                self.next_threshold += self.part_offset;
            }

            true
        } else {
            false
        }
    }

    fn received(&self) -> usize {
        self.received
    }

    fn total(&self) -> usize {
        self.total
    }
}

fn cancelled_error() -> PixivError {
    PixivError::new(PixivErrorKind::HttpClient, "Download cancelled".to_owned())
}

fn content_length_error() -> PixivError {
    PixivError::new(PixivErrorKind::HttpClient, "No content length".to_owned())
}

fn content_length_overflow_error() -> PixivError {
    PixivError::new(
        PixivErrorKind::HttpClient,
        "Content length is too large".to_owned(),
    )
}

fn incomplete_download_error(received: usize, total: usize) -> PixivError {
    PixivError::new(
        PixivErrorKind::HttpClient,
        format!("Incomplete download: received {received}, expected {total}"),
    )
}

fn io_error(context: &str, error: std::io::Error) -> PixivError {
    PixivError::new(PixivErrorKind::HttpClient, format!("{context}: {error}"))
}

async fn send_request(
    client: &Client,
    url: &str,
    cancel_token: &CancellationToken,
) -> Result<Response, PixivError> {
    tokio::select! {
        _ = cancel_token.cancelled() => {
            Err(cancelled_error())
        }
        response = client.get(url).header(header::REFERER, "https://www.pixiv.net/").send() => {
            response.map_err(Into::into)
        }
    }
}

fn download_client(proxy: Option<&str>) -> Result<Client, PixivError> {
    let mut builder = Client::builder();

    if let Some(proxy) = proxy {
        builder = builder.proxy(Proxy::all(proxy)?);
    }

    Ok(builder.build()?)
}

async fn check_response(response: Response) -> Result<(Response, usize), PixivError> {
    let status = response.status();

    if !status.is_success() {
        let response_url = response.url().to_string();
        let body = response.text().await.unwrap_or_default();
        return Err(PixivError::http_status(status.as_u16(), body).with_url(response_url));
    }

    let total_size = response.content_length().ok_or_else(content_length_error)?;

    let total_size = usize::try_from(total_size).map_err(|_| content_length_overflow_error())?;

    Ok((response, total_size))
}

async fn next_chunk<S>(
    stream: &mut S,
    cancel_token: &CancellationToken,
) -> Result<Option<S::Ok>, PixivError>
where
    S: TryStream<Error = reqwest::Error> + Unpin,
{
    tokio::select! {
        _ = cancel_token.cancelled() => {
            Err(cancelled_error())
        }
        item = stream.try_next() => {
            item.map_err(Into::into)
        }
    }
}

async fn remove_file_if_exists(path: &Path) -> Result<(), PixivError> {
    match fs::remove_file(path).await {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("Remove file failed", error)),
    }
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut filename = path
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("download"));
    filename.push(".tmp");

    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.join(&filename))
        .unwrap_or_else(|| PathBuf::from(filename))
}

async fn move_file_with_fallback(from: &Path, to: &Path) -> Result<(), PixivError> {
    match fs::rename(from, to).await {
        Ok(_) => return Ok(()),
        Err(first_rename_error) => {
            remove_file_if_exists(to).await?;

            match fs::rename(from, to).await {
                Ok(_) => return Ok(()),
                Err(second_rename_error) => {
                    if let Err(copy_error) = fs::copy(from, to).await {
                        let _ = fs::remove_file(to).await;
                        return Err(PixivError::new(
                            PixivErrorKind::HttpClient,
                            format!(
                                "Move temp file failed: firstRename={first_rename_error}, secondRename={second_rename_error}, copy={copy_error}"
                            ),
                        ));
                    }

                    let _ = fs::remove_file(from).await;

                    Ok(())
                }
            }
        }
    }
}

pub fn download_to_memory(
    url: String,
    proxy: Option<String>,
) -> Result<MemoryDownloadStream, PixivError> {
    let cancel_token = CancellationToken::new();
    let stream_cancel_token = cancel_token.clone();

    let stream = try_stream! {
        let client = download_client(proxy.as_deref())?;

        let response = send_request(&client, &url, &stream_cancel_token).await?;
        let (response, total_size) = check_response(response).await?;

        let mut output = Vec::with_capacity(total_size);
        let mut progress = ProgressTracker::new(total_size);
        let mut bytes_stream = response.bytes_stream();

        loop {
            let chunk = match next_chunk(&mut bytes_stream, &stream_cancel_token).await? {
                Some(chunk) => chunk,
                None => break,
            };

            output.extend_from_slice(chunk.as_ref());

            if progress.record(chunk.as_ref().len()) {
                yield DownloadEvent::Progress {
                    received: progress.received(),
                    total: progress.total(),
                };
            }
        }

        if progress.received() != progress.total() {
            Err(incomplete_download_error(progress.received(), progress.total()))?;
        }

        yield DownloadEvent::Done { output };
    };

    Ok(DownloadStream::new(cancel_token, Box::pin(stream)))
}

pub fn download_to_file(
    url: String,
    path: PathBuf,
    proxy: Option<String>,
) -> Result<FileDownloadStream, PixivError> {
    let cancel_token = CancellationToken::new();
    let stream_cancel_token = cancel_token.clone();

    let stream = try_stream! {
        let client = download_client(proxy.as_deref())?;

        let response = send_request(&client, &url, &stream_cancel_token).await?;
        let (response, total_size) = check_response(response).await?;

        let tmp_path = temp_path_for(&path);

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|error| io_error("Create parent dir failed", error))?;
            }
        }

        remove_file_if_exists(&tmp_path).await?;

        let file = File::create(&tmp_path)
            .await
            .map_err(|error| io_error("Create temp file failed", error))?;

        // 1MB buffer
        let mut writer = BufWriter::with_capacity(1024 * 1024, file);

        let mut progress = ProgressTracker::new(total_size);
        let mut bytes_stream = response.bytes_stream();

        loop {
            let chunk = match next_chunk(&mut bytes_stream, &stream_cancel_token).await {
                Ok(Some(chunk)) => chunk,
                Ok(None) => break,
                Err(error) => {
                    let _ = fs::remove_file(&tmp_path).await;
                    Err(error)?
                }
            };

            if let Err(error) = writer.write_all(chunk.as_ref()).await {
                let _ = fs::remove_file(&tmp_path).await;
                Err(io_error("Write temp file failed", error))?;
            }

            if progress.record(chunk.as_ref().len()) {
                yield DownloadEvent::Progress {
                    received: progress.received(),
                    total: progress.total(),
                };
            }
        }

        if progress.received() != progress.total() {
            let _ = fs::remove_file(&tmp_path).await;
            Err(incomplete_download_error(progress.received(), progress.total()))?;
        }

        if let Err(error) = writer.flush().await {
            let _ = fs::remove_file(&tmp_path).await;
            Err(io_error("Flush temp file failed", error))?;
        }

        drop(writer);

        move_file_with_fallback(&tmp_path, &path).await?;

        yield DownloadEvent::Done { output: path };
    };

    Ok(DownloadStream::new(cancel_token, Box::pin(stream)))
}
