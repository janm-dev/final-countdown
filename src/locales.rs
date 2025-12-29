use std::cmp::Reverse;

use axum::{
	body::Body,
	http::{
		Request,
		header::{ACCEPT_LANGUAGE, COOKIE},
	},
	middleware::Next,
	response::Response,
};
use cookie::Cookie;
use icu::{
	locid::{Locale, locale},
	locid_transform::fallback::LocaleFallbacker,
};
use tracing::{debug, trace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locales {
	/// The list of locale-preference pairs. The preference is 0 for the default
	/// locale, `quality * 100` for locales from the `Accept-Language` header,
	/// and 255 for the locale from the `locale` cookie. Always ordered by
	/// preference from most to least preferred, so that the most-preferred
	/// locale is the first in the list. Always contains at least one locale:
	/// the default locale with priority 0, and therefore `locale.list[0].0`
	/// will never panic, and always return the most-preferred locale.
	list: Vec<(Locale, u8)>,
}

impl Locales {
	pub const MAX_PRIORITY: u8 = u8::MAX;

	pub async fn middleware(mut request: Request<Body>, next: Next) -> Response {
		let mut finder = Self::default();

		debug!("setting locale from the `Accept-Language` header");
		request
			.headers()
			.get_all(ACCEPT_LANGUAGE)
			.into_iter()
			.filter_map(|v| v.to_str().ok())
			.flat_map(|v| {
				accept_language::parse_with_quality(v)
					.into_iter()
					.map(|(l, q)| (l, (q * 100.0) as u8))
			})
			.for_each(|(l, q)| finder.update(&l, q));

		debug!("setting locale from the `locale` cookie");
		request
			.headers()
			.get_all(COOKIE)
			.into_iter()
			.filter_map(|v| v.to_str().ok())
			.flat_map(Cookie::split_parse)
			.filter_map(|r| r.ok())
			.filter(|c| c.name() == "locale")
			.for_each(|c| finder.update(c.value(), Self::MAX_PRIORITY));

		request.extensions_mut().insert(finder);

		next.run(request).await
	}

	/// Get the list of locale-preference pairs
	///
	/// The locales are always ordered by preference from most to least
	/// preferred, so that the most-preferred locale is the first in the list.
	/// Always contains at least one locale: the default locale with priority 0,
	/// and therefore `locale.list[0]` will never panic, and always be the
	/// most-preferred locale.
	pub fn list(&self) -> &[(Locale, u8)] {
		&self.list
	}

	pub fn update(&mut self, locale: &str, mut priority: u8) {
		let Ok(locale) = locale.parse::<Locale>() else {
			debug!(
				"locale '{}' with priority {} is invalid and was not added",
				locale, priority
			);
			return;
		};

		let new = (locale.clone(), priority);

		let fallbacker = LocaleFallbacker::new();
		let key_fallbacker = fallbacker.for_config(Default::default());
		let mut iterator = key_fallbacker.fallback_for(locale.into());

		let mut locale = iterator.get(); // Because a `LocaleFallbackIterator` is not an iterator
		while locale != &locale!("und").into() {
			self.insert(locale.clone().into_locale(), priority);
			priority /= 2;
			locale = iterator.step().get();
		}

		debug!(
			"added locale {}@{} and its fallbacks to locale list, current preference list is [{}]",
			new.0,
			new.1,
			self.list
				.iter()
				.map(|(l, p)| format!("{l}@{p}"))
				.collect::<Vec<_>>()
				.join(", ")
		);
	}

	/// Insert one locale-preference pair into the list while keeping it sorted
	fn insert(&mut self, locale: Locale, priority: u8) {
		let i = match self
			.list
			.binary_search_by_key(&Reverse(priority), |e| Reverse(e.1))
		{
			Ok(i) => i,
			Err(i) => i,
		};

		self.list.insert(i, (locale, priority));

		trace!(
			"added locale {}@{} to locale list, current preference list is [{}]",
			self.list[i].0,
			self.list[i].1,
			self.list
				.iter()
				.map(|(l, p)| format!("{l}@{p}"))
				.collect::<Vec<_>>()
				.join(", ")
		);
	}
}

impl Default for Locales {
	fn default() -> Self {
		Self {
			list: vec![(locale!("und"), 0)],
		}
	}
}
