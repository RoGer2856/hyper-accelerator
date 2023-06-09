use crate::response_body::{AsyncStream, ResponseBody};

#[derive(Debug)]
pub enum SerializeToJsonBodyError {
    SerdeJson(serde_json::Error),
}

impl From<serde_json::Error> for SerializeToJsonBodyError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerdeJson(e)
    }
}

pub fn create_json_body<T: serde::Serialize>(
    data: &T,
) -> Result<ResponseBody, SerializeToJsonBodyError> {
    Ok(ResponseBody::from(serde_json::to_string(data)?))
}

pub fn create_static_str_body(text: &'static str) -> ResponseBody {
    ResponseBody::from(text)
}

pub fn create_string_body(text: impl ToString) -> ResponseBody {
    ResponseBody::from(text.to_string())
}

pub fn create_bytes_body(bytes: &[u8]) -> ResponseBody {
    ResponseBody::from(Vec::from(bytes))
}

pub fn create_stream_body(stream: impl AsyncStream<Vec<u8>>) -> ResponseBody {
    ResponseBody::from(stream)
}
