use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, Method, Response, StatusCode},
    routing::get,
    Router,
};
use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, TcpListener as StdTcpListener},
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, SeekFrom},
    net::TcpListener,
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[derive(Clone)]
pub struct VideoServer {
    addr: SocketAddr,
    files: Arc<RwLock<HashMap<String, PathBuf>>>,
}

impl VideoServer {
    /// Bind to a random available loopback port and spawn the server.
    pub fn start() -> io::Result<Self> {
        let listener = StdTcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let addr = listener.local_addr()?;

        let server = Self {
            addr,
            files: Arc::new(RwLock::new(HashMap::new())),
        };

        let server_for_task = server.clone();

        tauri::async_runtime::spawn(async move {
            let listener = match TcpListener::from_std(listener) {
                Ok(listener) => listener,
                Err(error) => {
                    eprintln!("Video server listener error: {error}");
                    return;
                }
            };

            let app = Router::new().route("/video/{id}", get(serve_video).head(serve_video)).with_state(server_for_task);

            if let Err(error) = axum::serve(listener, app).await {
                eprintln!("Video server error: {error}");
            }
        });

        Ok(server)
    }

    /// Register a file and return its URL.
    pub async fn register(&self, path: PathBuf) -> Result<String, String> {
        let path = tokio::fs::canonicalize(path).await.map_err(|e| format!("Cannot resolve video path: {e}"))?;

        let metadata = tokio::fs::metadata(&path).await.map_err(|e| format!("Cannot read video metadata: {e}"))?;

        if !metadata.is_file() {
            return Err("The selected path is not a file".into());
        }

        let id = Uuid::new_v4().simple().to_string();

        self.files.write().map_err(|_| "Video registry lock poisoned".to_string())?.insert(id.clone(), path);

        Ok(format!("http://{}/video/{id}", self.addr))
    }
}

fn response(status: StatusCode, body: Body) -> Response<Body> {
    Response::builder().status(status).body(body).unwrap()
}

fn parse_range(value: &str, len: u64) -> Result<(u64, u64), ()> {
    if len == 0 {
        return Err(());
    }

    let spec = value.strip_prefix("bytes=").ok_or(())?;

    // This implementation deliberately supports one range only.
    if spec.contains(',') {
        return Err(());
    }

    let (start, end) = spec.split_once('-').ok_or(())?;

    if start.is_empty() {
        let suffix_len: u64 = end.parse().map_err(|_| ())?;
        if suffix_len == 0 {
            return Err(());
        }

        return Ok((len.saturating_sub(suffix_len), len - 1));
    }

    let start: u64 = start.parse().map_err(|_| ())?;
    if start >= len {
        return Err(());
    }

    let end = if end.is_empty() {
        len - 1
    } else {
        let end: u64 = end.parse().map_err(|_| ())?;
        if end < start {
            return Err(());
        }
        end.min(len - 1)
    };

    Ok((start, end))
}

async fn serve_video(State(server): State<VideoServer>, Path(id): Path<String>, method: Method, headers: HeaderMap) -> Response<Body> {
    if method != Method::GET && method != Method::HEAD {
        return response(StatusCode::METHOD_NOT_ALLOWED, Body::empty());
    }

    let path = match server.files.read() {
        Ok(files) => match files.get(&id) {
            Some(path) => path.clone(),
            None => return response(StatusCode::NOT_FOUND, Body::empty()),
        },
        Err(_) => {
            return response(StatusCode::INTERNAL_SERVER_ERROR, Body::empty());
        }
    };

    let mut file = match File::open(&path).await {
        Ok(file) => file,
        Err(_) => return response(StatusCode::NOT_FOUND, Body::empty()),
    };

    let metadata = match file.metadata().await {
        Ok(metadata) => metadata,
        Err(_) => {
            return response(StatusCode::INTERNAL_SERVER_ERROR, Body::empty());
        }
    };

    let len = metadata.len();
    let mime = mime_guess::from_path(&path).first_or_octet_stream().to_string();

    let range_header = headers.get(header::RANGE);
    let (status, start, count, content_range) = match range_header {
        None => (StatusCode::OK, 0, len, None),
        Some(value) => {
            let parsed = value.to_str().ok().and_then(|s| parse_range(s, len).ok());

            match parsed {
                Some((start, end)) => (StatusCode::PARTIAL_CONTENT, start, end - start + 1, Some(format!("bytes {start}-{end}/{len}"))),
                None => {
                    return Response::builder().status(StatusCode::RANGE_NOT_SATISFIABLE).header(header::CONTENT_RANGE, format!("bytes */{len}")).body(Body::empty()).unwrap();
                }
            }
        }
    };

    if start > 0 && file.seek(SeekFrom::Start(start)).await.is_err() {
        return response(StatusCode::INTERNAL_SERVER_ERROR, Body::empty());
    }

    let mut builder = Response::builder().status(status).header(header::CONTENT_TYPE, mime).header(header::CONTENT_LENGTH, count.to_string()).header(header::ACCEPT_RANGES, "bytes");

    if let Some(content_range) = content_range {
        builder = builder.header(header::CONTENT_RANGE, content_range);
    }

    let body = if method == Method::HEAD || count == 0 {
        Body::empty()
    } else {
        Body::from_stream(ReaderStream::new(file.take(count)))
    };

    builder.body(body).unwrap()
}
