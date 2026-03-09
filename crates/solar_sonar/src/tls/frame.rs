use crate::*;

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum ClientToServer {
    Close,
}

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum ServerToClient {
    Close,
    Event(DataEvent),
}

pub(crate) struct BitcodeCodec<T> {
    codec: r::tokio::LengthDelimitedCodec,
    _marker: PhantomData<T>,
}

impl<T> BitcodeCodec<T> {
    pub(crate) fn new() -> Self {
        Self {
            codec: r::tokio::LengthDelimitedCodec::new(),
            _marker: PhantomData,
        }
    }
}

impl<T> r::tokio::Decoder for BitcodeCodec<T>
where
    T: for<'a> bitcode::Decode<'a>,
{
    type Item = T;
    type Error = io::Error;

    fn decode(&mut self, src: &mut r::tokio::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let frame = match self.codec.decode(src)? {
            Some(frame) => frame,
            None => return Ok(None),
        };

        bitcode::decode(&frame)
            .map(Some)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to decode bitcode frame"))
    }
}

impl<T> r::tokio::Encoder<T> for BitcodeCodec<T>
where
    T: bitcode::Encode,
{
    type Error = std::io::Error;

    fn encode(&mut self, item: T, dst: &mut r::tokio::BytesMut) -> Result<(), Self::Error> {
        let bytes = bitcode::encode(&item);
        self.codec.encode(bytes.into(), dst)
    }
}