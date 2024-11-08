mod streaming_wav;

pub use reqwest::Client; //Need to re-export since it's used in [`new_http_wav`]'s api.
pub use streaming_wav::*;

use reqwest::Url;
use stream_download::{
    http::{HttpStream, HttpStreamError},
    storage::memory::MemoryStorageProvider,
    StreamDownload, StreamInitializationError,
};
use thiserror::Error;

/// A nameable alias for the [`StreamingWav`] type outputted by [`new_http_wav`].
#[allow(type_alias_bounds)]
pub type HttpStreamingWav<S: WavSample> = StreamingWav<S, StreamDownload<MemoryStorageProvider>>;

pub async fn get_wav_stream<S: WavSample>(
    client: Client,
    url: Url,
) -> Result<HttpStreamingWav<S>, NewHttpWavError> {
    let reader = StreamDownload::from_stream(
        HttpStream::new(client, url).await?,
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

    #[error("Error in making the request/getting a response.")]
    DownloadError(#[from] HttpStreamError<Client>),

    #[error("Error initializing the stream.")]
    StreamInitializationError(#[from] StreamInitializationError<HttpStream<Client>>),
}
