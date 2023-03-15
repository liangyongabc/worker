use bytes::Buf;
use futures_util::StreamExt;
use wasm_bindgen::JsCast;
use worker_sys::ext::{HeadersExt, ResponseExt, ResponseInitExt};

use crate::WebSocket;

use crate::body::Body;

pub fn from_wasm(res: web_sys::Response) -> http::Response<Body> {
    let mut builder = http::Response::builder().status(res.status());

    for header in res.headers().entries() {
        let header = header.unwrap().unchecked_into::<js_sys::Array>();
        builder = builder.header(
            header.get(0).as_string().unwrap(),
            header.get(1).as_string().unwrap(),
        );
    }

    if let Some(ws) = res.websocket() {
        builder = builder.extension(WebSocket::from(ws));
    }

    builder.body(Body::from(res)).unwrap()
}

pub fn into_wasm(mut res: http::Response<Body>) -> web_sys::Response {
    let status = res.status().as_u16();

    let headers = web_sys::Headers::new().unwrap();
    for (name, value) in res
        .headers()
        .into_iter()
        .filter_map(|(name, value)| value.to_str().map(|value| (name.as_str(), value)).ok())
    {
        headers.append(name, value).unwrap();
    }

    let mut init = web_sys::ResponseInit::new();
    init.status(status).headers(&headers);

    if let Some(ws) = res.extensions_mut().remove::<WebSocket>() {
        init.websocket(ws.as_ref());
    }

    let body = res.into_body();
    let body = if body.is_none() {
        None
    } else {
        let stream = wasm_streams::ReadableStream::from_stream(body.map(|chunk| {
            chunk
                .map(|buf| js_sys::Uint8Array::from(buf.chunk()).into())
                .map_err(|_| wasm_bindgen::JsValue::NULL)
        }));

        Some(stream.into_raw().unchecked_into())
    };

    web_sys::Response::new_with_opt_readable_stream_and_init(body.as_ref(), &init).unwrap()
}