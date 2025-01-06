use std::{collections::HashMap, env, net::Ipv6Addr, str::FromStr};

use axum::{
	body::Body,
	extract::{Path, Query},
	http::{HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
	middleware::{self, Next},
	response::{Html, Response},
	routing::get,
	Extension, Json, Router,
};
use chrono::{Datelike, SecondsFormat, Timelike, Utc};
use cookie::{time::OffsetDateTime, Cookie, SameSite};
use files::StaticResponse;
use icu::{
	calendar::DateTime,
	datetime::{
		time_zone::TimeZoneFormatterOptions, DateTimeFormatterOptions, ZonedDateTimeFormatter,
	},
	locid::Locale,
	timezone::CustomTimeZone,
};
use locales::Locales;
use tokio::net::TcpListener;
use tracing::{info, warn};

mod files;
mod locales;
mod pages;
mod types;
mod intl {
	include!(concat!(env!("OUT_DIR"), "/time_units.rs"));
}

use pages::{countdown_html, new_html};
use tracing_subscriber::filter::LevelFilter;
use types::{ReqId, Timestamp, Title};

async fn new(
	Extension(locale): Extension<Locales>,
	Extension(id): Extension<ReqId>,
) -> Html<String> {
	Html(new_html(locale, id))
}

async fn now(Extension(locales): Extension<Locales>) -> String {
	let now = Utc::now();
	let rfc = now.to_rfc3339_opts(SecondsFormat::Micros, true);
	let unix = now.timestamp();
	let unix_subsec = now.timestamp_subsec_micros();
	let (formatter, locale) = locales
		.list()
		.iter()
		.filter_map(|(l, _)| {
			Some((
				ZonedDateTimeFormatter::try_new(
					&l.into(),
					DateTimeFormatterOptions::default(),
					TimeZoneFormatterOptions::default(),
				)
				.ok()?,
				l.clone(),
			))
		})
		.next()
		.expect("at least one locale must have the required data");
	let local = formatter
		.format_to_string(
			&DateTime::try_new_iso_datetime(
				now.year(),
				now.month() as u8,
				now.day() as u8,
				now.hour() as u8,
				now.minute() as u8,
				now.second() as u8,
			)
			.expect("the date is correct")
			.to_any(),
			&CustomTimeZone::utc(),
		)
		.expect("the calender is correct");
	let locales = locales
		.list()
		.iter()
		.map(|(l, p)| format!("{l}@{p}"))
		.collect::<Vec<_>>()
		.join(", ");

	format!("{rfc}\n{unix}.{unix_subsec:06}\n{locale} | [{locales}]\n{local}")
}

async fn countdown(
	timestamp: Timestamp,
	title: Option<Title>,
	_options: HashMap<String, String>,
	locale: Locales,
	id: ReqId,
) -> Html<String> {
	Html(countdown_html(
		title.unwrap_or_default(),
		timestamp,
		locale,
		id,
	))
}

async fn headers(req: Request<Body>, next: Next) -> Response {
	let id = req.extensions().get::<ReqId>().cloned();

	let mut res = next.run(req).await;

	res.headers_mut().insert(
		HeaderName::from_static("access-control-allow-origin"),
		HeaderValue::from_static("*"),
	);

	res.headers_mut().insert(
		HeaderName::from_static("access-control-allow-methods"),
		HeaderValue::from_static("GET"),
	);

	res.headers_mut().insert(
		HeaderName::from_static("cross-origin-resource-policy"),
		HeaderValue::from_static("cross-origin"),
	);

	res.headers_mut().insert(
		HeaderName::from_static("cross-origin-embedder-policy"),
		HeaderValue::from_static("require-corp"),
	);

	res.headers_mut().insert(
		HeaderName::from_static("cross-origin-opener-policy"),
		HeaderValue::from_static("same-origin"),
	);

	res.headers_mut().insert(
		HeaderName::from_static("x-content-type-options"),
		HeaderValue::from_static("nosniff"),
	);

	if res.extensions().get::<StaticResponse>().is_some() {
		// Enable caching
		res.headers_mut().insert(
			HeaderName::from_static("cache-control"),
			HeaderValue::from_static("max-age=86400, stale-while-revalidate=86400"),
		);
	} else {
		// Vary the response based on non-headers (the current time in this case)
		res.headers_mut().insert(
			HeaderName::from_static("vary"),
			HeaderValue::from_static("*"),
		);

		// Disable caching
		res.headers_mut().insert(
			HeaderName::from_static("cache-control"),
			HeaderValue::from_static("no-store"),
		);
	}

	let Some(id) = id else {
		warn!("Request ID missing");
		return res;
	};

	let csp = format!(
		"default-src 'none'; script-src 'nonce-{id}'; style-src 'nonce-{id}'; font-src 'self'; \
		 base-uri 'none'; connect-src 'self'; img-src 'self'; manifest-src 'self'; sandbox \
		 allow-same-origin allow-scripts allow-top-navigation"
	);
	res.headers_mut().insert(
		HeaderName::from_static("content-security-policy"),
		HeaderValue::from_str(&csp).expect("invalid csp header value"),
	);

	res
}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_max_level(
			env::var("FINAL_COUNTDOWN_LOG_LEVEL")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(if cfg!(debug_assertions) {
					LevelFilter::DEBUG
				} else {
					LevelFilter::INFO
				}),
		)
		.init();

	let get_cd = get(
		|Path(timestamp), Query(options), Extension(locale), Extension(id)| {
			countdown(timestamp, None, options, locale, id)
		},
	);
	let get_cd_titled = get(
		|Path((timestamp, title)), Query(options), Extension(locale), Extension(id)| {
			countdown(timestamp, title, options, locale, id)
		},
	);

	let app = Router::new()
		.route("/", get(new))
		.route("/now", get(now))
		.route(
			"/site.webmanifest",
			serve!("site.webmanifest" as "application/manifest"),
		)
		.route(
			"/font-numbers.woff2",
			serve!("font-numbers.woff2" as "font/woff2"),
		)
		.route(
			"/font-text.woff2",
			serve!("font-text.woff2" as "font/woff2"),
		)
		.route("/icon.ico", serve!("icon.ico" as "image/x-icon"))
		.route("/icon.png", serve!("icon.png" as "image/png"))
		.route("/icon.svg", serve!("icon.svg" as "image/svg+xml"))
		.route("/maskable.png", serve!("maskable.png" as "image/png"))
		.route("/maskable.svg", serve!("maskable.svg" as "image/svg+xml"))
		.route("/monochrome.png", serve!("monochrome.png" as "image/png"))
		.route(
			"/monochrome.svg",
			serve!("monochrome.svg" as "image/svg+xml"),
		)
		.route("/{timestamp}", get_cd.clone())
		.route("/{timestamp}/", get_cd)
		.route("/{timestamp}/{title}", get_cd_titled.clone())
		.route("/{timestamp}/{title}/", get_cd_titled)
		.layer(middleware::from_fn(Locales::middleware))
		.layer(middleware::from_fn(headers))
		.layer(middleware::from_fn(ReqId::middleware));

	let app = if cfg!(debug_assertions) {
		app.route(
			"/set-locale/{locale}",
			get(|Path(locale): Path<String>| async move {
				(
					StatusCode::OK,
					HeaderMap::from_iter([(
						HeaderName::from_static("set-cookie"),
						HeaderValue::from_str(
							&Cookie::build(("locale", &locale))
								.permanent()
								.path("/")
								.same_site(SameSite::Lax)
								.to_string(),
						)
						.expect("the cookie is valid"),
					)]),
					format!("`locale` Cookie successfully set to {locale}"),
				)
			}),
		)
		.route(
			"/clear-locale",
			get(|| async {
				(
					StatusCode::OK,
					HeaderMap::from_iter([(
						HeaderName::from_static("set-cookie"),
						HeaderValue::from_str(
							&Cookie::build("locale")
								.expires(OffsetDateTime::UNIX_EPOCH)
								.path("/")
								.same_site(SameSite::Lax)
								.to_string(),
						)
						.expect("the cookie is valid"),
					)]),
					"`locale` Cookie successfully unset",
				)
			}),
		)
		.route(
			"/locale/{locid}",
			get(|Path(locid): Path<String>| async move {
				intl::UNITS
					.get(
						&Locale::from_str(&locid)
							.map_err(|_| StatusCode::BAD_REQUEST)?
							.to_string(),
					)
					.ok_or(StatusCode::NOT_FOUND)
					.cloned()
					.map(Json)
			}),
		)
	} else {
		app
	};

	info!("Final Countdown server starting");

	axum::serve(
		TcpListener::bind((Ipv6Addr::UNSPECIFIED, 8080))
			.await
			.unwrap(),
		app.into_make_service(),
	)
	.await
	.unwrap();
}
