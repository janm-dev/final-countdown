//! Helper types for the HTTP path and query

use std::{
	borrow::Cow,
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	str::FromStr,
};

use axum::{
	body::Body,
	http::Request,
	middleware::Next,
	response::{IntoResponse, Response},
};
use chrono::{DateTime, FixedOffset, SecondsFormat};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ReqId(pub Uuid);

impl ReqId {
	pub fn random() -> Self {
		Self(Uuid::new_v4())
	}

	pub async fn middleware(mut request: Request<Body>, next: Next) -> Response {
		let id = Self::random();
		debug!(id = %id, "New request");

		request.extensions_mut().insert(id);
		next.run(request).await
	}
}

impl Display for ReqId {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_fmt(format_args!("{}", self.0))
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(try_from = "SerdeTimestamp", into = "SerdeTimestamp")]
pub struct Timestamp(pub DateTime<FixedOffset>);

impl Timestamp {
	pub fn from_unix(timestamp: i64) -> Result<Self, TimestampError> {
		DateTime::from_timestamp(timestamp, 0)
			.map(|dt| dt.fixed_offset())
			.map(Timestamp)
			.ok_or(TimestampError)
	}

	pub fn from_rfc3339(datetime: &str) -> Result<Self, TimestampError> {
		DateTime::parse_from_rfc3339(datetime)
			.map_err(|_| TimestampError)
			.map(Timestamp)
	}
}

impl FromStr for Timestamp {
	type Err = TimestampError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		i64::from_str(s)
			.map_err(|_| TimestampError)
			.and_then(Timestamp::from_unix)
			.or_else(|_| Timestamp::from_rfc3339(s))
	}
}

impl Display for Timestamp {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0.to_rfc3339_opts(SecondsFormat::Secs, true))
	}
}

impl<'a> TryFrom<SerdeTimestamp<'a>> for Timestamp {
	type Error = TimestampError;

	fn try_from(value: SerdeTimestamp) -> Result<Self, Self::Error> {
		match value {
			SerdeTimestamp::Int(i) => Self::from_unix(i),
			SerdeTimestamp::Str(s) => Self::from_str(&s),
		}
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum SerdeTimestamp<'a> {
	Int(i64),
	Str(Cow<'a, str>),
}

impl From<Timestamp> for SerdeTimestamp<'static> {
	fn from(value: Timestamp) -> Self {
		Self::Str(
			value
				.0
				.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
				.into(),
		)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct TimestampError;

impl Display for TimestampError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("invalid timestamp")
	}
}

impl Error for TimestampError {}

impl IntoResponse for TimestampError {
	fn into_response(self) -> Response {
		Response::builder()
			.status(404)
			.body(Body::default())
			.unwrap()
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(into = "SerdeTitle", from = "SerdeTitle")]
pub struct Title(pub String);

impl Title {
	/// The character that represents a space in the title (in addition to a
	/// space itself)
	const SPACE: char = '_';

	fn new(s: &str) -> Self {
		Title(s.replace(Self::SPACE, " "))
	}
}

impl Display for Title {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl<'a> From<SerdeTitle<'a>> for Title {
	fn from(value: SerdeTitle) -> Self {
		Self::new(&value.0)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct SerdeTitle<'a>(Cow<'a, str>);

impl From<Title> for SerdeTitle<'static> {
	fn from(value: Title) -> Self {
		Self(value.0.into())
	}
}
