//! This module defines the [`StreamingWav`] struct.
//!
//! For a useful specification of the WAVE format, see [here](https://www.mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html).

use std::time::Duration;
use std::{io::Read, marker::PhantomData};

use rodio::{Sample, Source};
use thiserror::Error;

/// Wraps a [`Read`] into an audio [`Source`] that can be used with [`rodio`].
pub struct StreamingWav<S: WavSample, R: Read> {
    reader: R,
    sample_rate: u32,
    channels: u16,
    _sample_type: PhantomData<S>,
}

impl<S: WavSample, R: Read> StreamingWav<S, R> {
    pub fn new(mut reader: R) -> Result<Self, StreamingWavError> {
        // Read the RIFF header
        let mut riff_header = [0u8; 12];
        reader.read_exact(&mut riff_header)?;

        // Verify the RIFF and WAVE identifiers
        if &riff_header[0..4] != b"RIFF" {
            return Err(StreamingWavError::InvalidRiffFormat);
        }
        if &riff_header[8..12] != b"WAVE" {
            return Err(StreamingWavError::InvalidWaveFormat);
        }

        let mut sample_rate = 0u32;
        let mut channels = 0u16;

        // Read chunks until we find the 'fmt ' and 'data' chunks
        loop {
            // Read chunk header
            let mut chunk_header = [0u8; 8];
            reader.read_exact(&mut chunk_header)?;
            let chunk_id = &chunk_header[0..4];
            let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().unwrap());

            if chunk_id == b"fmt " {
                // Read 'fmt ' chunk
                let mut fmt_chunk = vec![0u8; chunk_size as usize];
                reader.read_exact(&mut fmt_chunk)?;

                let audio_format = u16::from_le_bytes(fmt_chunk[0..2].try_into().unwrap());
                if audio_format != S::FORMAT {
                    return Err(StreamingWavError::UnsupportedAudioFormat(audio_format));
                }

                channels = u16::from_le_bytes(fmt_chunk[2..4].try_into().unwrap());
                sample_rate = u32::from_le_bytes(fmt_chunk[4..8].try_into().unwrap());
                let bits_per_sample = u16::from_le_bytes(fmt_chunk[14..16].try_into().unwrap());

                if bits_per_sample != S::BITS {
                    return Err(StreamingWavError::UnsupportedBitsPerSample(bits_per_sample));
                }
            } else if chunk_id == b"data" {
                // Found the 'data' chunk; ready to read samples
                return Ok(Self {
                    reader,
                    sample_rate,
                    channels,
                    _sample_type: PhantomData,
                });
            } else {
                // Skip over non-'fmt ' and non-'data' chunks
                let mut skip = chunk_size as usize;
                if skip % 2 == 1 {
                    skip += 1; // Account for padding byte
                }
                std::io::copy(&mut reader.by_ref().take(skip as u64), &mut std::io::sink())?;
            }
        }
    }
}

impl<S: WavSample, R: Read> Source for StreamingWav<S, R> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None // Streamed, so duration is unknown ahead of time
    }
}

impl<S: WavSample, R: Read> Iterator for StreamingWav<S, R> {
    type Item = S::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        S::next(&mut self.reader)
    }
}

#[derive(Debug, Error)]
pub enum StreamingWavError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid RIFF marker")]
    InvalidRiffFormat,

    #[error("Invalid WAVE marker")]
    InvalidWaveFormat,

    #[error("Unsupported audio format: {0}")]
    UnsupportedAudioFormat(u16),

    #[error("Unsupported bits per sample: {0}")]
    UnsupportedBitsPerSample(u16),
}

pub trait WavSample {
    type Sample: Sample;
    const FORMAT: u16;
    const BITS: u16;
    const BYTES: usize = Self::BITS as usize / 8;

    fn next<R: Read>(reader: &mut R) -> Option<Self::Sample>;
}

impl WavSample for u8 {
    type Sample = i16;
    const FORMAT: u16 = 1;
    const BITS: u16 = 8;

    fn next<R: Read>(reader: &mut R) -> Option<Self::Sample> {
        let mut buf = [0u8; 1];
        match reader.read_exact(&mut buf) {
            Ok(_) => Some((buf[0] as i16 - 128) * 256),
            Err(_) => None,
        }
    }
}

impl WavSample for i16 {
    type Sample = i16;
    const FORMAT: u16 = 1;
    const BITS: u16 = 16;

    fn next<R: Read>(reader: &mut R) -> Option<Self::Sample> {
        let mut buf = [0u8; 2];
        match reader.read_exact(&mut buf) {
            Ok(_) => Some(i16::from_le_bytes(buf)),
            Err(_) => None,
        }
    }
}

impl WavSample for f32 {
    type Sample = f32;
    const FORMAT: u16 = 3;
    const BITS: u16 = 32;

    fn next<R: Read>(reader: &mut R) -> Option<Self::Sample> {
        let mut buf = [0u8; 4];
        match reader.read_exact(&mut buf) {
            Ok(_) => Some(f32::from_le_bytes(buf)),
            Err(_) => None,
        }
    }
}
