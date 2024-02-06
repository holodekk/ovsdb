use serde_json::Value;
use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::Decoder,
};

pub enum BufferTag {
    Obj,
    Str,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
    #[error("JSON protocol Error")]
    Serde(#[from] serde_json::Error),
}

#[derive(Default)]
pub struct JsonCodec {
    data: Vec<u8>,
    tags: Vec<BufferTag>,
}

impl JsonCodec {
    pub fn new() -> Self {
        Self::default()
    }

    fn try_decode_object(
        &mut self,
        src: &[u8],
    ) -> Result<(Option<Value>, usize), serde_json::Error> {
        let mut offset = 0;

        while offset < src.len() {
            match self.tags.last() {
                Some(BufferTag::Str) => {
                    if let Some(n) = &src[offset..].iter().position(|&c| c == b'"') {
                        offset += n + 1;
                        self.tags.pop();
                        continue;
                    } else {
                        break;
                    }
                }
                Some(BufferTag::Obj) => {
                    if let Some(n) = &src[offset..]
                        .iter()
                        .position(|&c| [b'"', b'{', b'}'].contains(&c))
                    {
                        offset += n;
                        let char = src[offset];
                        offset += 1;
                        match &char {
                            b'"' => self.tags.push(BufferTag::Str),
                            b'{' => self.tags.push(BufferTag::Obj),
                            b'}' => {
                                self.tags.pop();
                                if self.tags.is_empty() {
                                    // We have a full object
                                    self.data.extend_from_slice(src);
                                    let obj = serde_json::from_slice(&self.data.to_vec())?;
                                    self.data.clear();
                                    return Ok((Some(obj), offset));
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        break;
                    }
                }
                None => {
                    if let Some(n) = &src[offset..].iter().position(|&c| c == b'{') {
                        offset += n + 1;
                        self.tags.push(BufferTag::Obj);
                    } else {
                        panic!("Datastream corrupted.  No openening tag.");
                    }
                }
            }
        }

        self.data.extend_from_slice(src);
        Ok((None, src.len()))
    }
}

impl Decoder for JsonCodec {
    type Item = Value;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let (res, consume) = self.try_decode_object(src.chunk())?;
        src.advance(consume);
        Ok(res)
    }
}
