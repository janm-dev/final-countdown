//! Static assets served from `include`ed files

/// A response extension indicating that the response is a static file
#[derive(Debug, Clone, Copy)]
pub struct StaticResponse;

/// Serve a static file from the assets directory
///
/// # Example
/// ```rust,ignore
/// Router::new().route("/icon.png", serve!("icon.png" as "image/png"));
/// ```
#[macro_export]
macro_rules! serve {
	($path:literal as $content_type:literal) => {{
		use ::axum::{
			Extension,
			http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
			routing::get,
		};
		use ::core::{concat, include_bytes};
		use $crate::files::StaticResponse;

		async fn serve() -> (
			StatusCode,
			HeaderMap<HeaderValue>,
			Extension<StaticResponse>,
			&'static [u8],
		) {
			const CONTENT: &[u8] = include_bytes!(concat!("../assets/", $path));

			(
				StatusCode::OK,
				HeaderMap::from_iter([(
					HeaderName::from_static("content-type"),
					HeaderValue::from_static($content_type),
				)]),
				Extension(StaticResponse),
				CONTENT,
			)
		}

		get(serve)
	}};
}
