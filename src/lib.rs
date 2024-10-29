mod streaming_wav;

pub use streaming_wav::*;

use reqwest::{Client, Url};
use stream_download::{
    http::HttpStream, storage::memory::MemoryStorageProvider, StreamDownload,
    StreamInitializationError,
};
use thiserror::Error;

/// A nameable alias for the [`StreamingWav`] type outputted by [`new_http_wav`].
#[allow(type_alias_bounds)]
pub type HttpStreamingWav<S: WavSample> = StreamingWav<S, StreamDownload<MemoryStorageProvider>>;

pub async fn new_http_wav<S: WavSample>(url: Url) -> Result<HttpStreamingWav<S>, NewHttpWavError> {
    let reader = StreamDownload::new_http(
        url,
        MemoryStorageProvider,
        stream_download::Settings::default(),
    )
    .await?;

    Ok(StreamingWav::new(reader)?)
}

#[derive(Debug, Error)]
pub enum NewHttpWavError {
    #[error("Error decoding the streamed wav file.")]
    WavDecodingError(#[from] StreamingWavError),

    #[error("Error initializing the stream.")]
    StreamInitializationError(#[from] StreamInitializationError<HttpStream<Client>>),
}
