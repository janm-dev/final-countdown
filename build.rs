use std::{
	env,
	fs::{self, File},
	io::Write,
	path::PathBuf,
	time::Duration,
};

use minify_html::Cfg;
use phf_codegen::Map;

const CLDR_ZIP_URL: &str =
	"https://github.com/unicode-org/cldr-json/releases/download/44.0.1/cldr-44.0.1-json-full.zip";

fn main() {
	println!("cargo:rerun-if-changed=cldr");
	println!("cargo:rerun-if-changed=cldr.zip");

	let (has_zip, has_cldr) = data_is_generated();

	if !(has_zip || has_cldr) {
		download_zip();
	}

	if !has_cldr {
		extract_cldr();
	}

	html();
	time_units();
}

fn html() {
	eprintln!("Compressing HTML ...");

	let mut assets =
		PathBuf::from(&env::var_os("CARGO_MANIFEST_DIR").expect("assets dir is unavailable"));
	assets.push("assets");
	let assets = assets;

	let mut file_cd = assets.clone();
	file_cd.push("countdown.html");
	let file_cd = file_cd;

	let mut file_new = assets.clone();
	file_new.push("new.html");
	let file_new = file_new;

	minify("countdown", file_cd);
	minify("new", file_new);

	eprintln!("HTML compressed and written");
}

/// Minify the html file in `path`. The resulting file will be output into the
/// `OUT_DIR` directory with the name `name.html`
fn minify(name: &str, path: PathBuf) {
	println!("cargo:rerun-if-changed={}", path.to_str().unwrap());

	let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap())
		.join(name)
		.with_extension("html");
	let html = fs::read_to_string(path).unwrap();

	let config = Cfg {
		do_not_minify_doctype: true,
		ensure_spec_compliant_unquoted_attribute_values: true,
		keep_closing_tags: false,
		keep_html_and_head_opening_tags: false,
		keep_spaces_between_attributes: true,
		keep_comments: false,
		minify_css: true,
		minify_js: true,
		remove_bangs: true,
		remove_processing_instructions: true,
		..Cfg::default()
	};

	let minified = minify_html::minify(html.as_bytes(), &config);

	fs::write(out_path, minified).unwrap();
}

fn data_is_generated() -> (bool, bool) {
	eprintln!("Checking available data ...");
	let source =
		PathBuf::from(&env::var_os("CARGO_MANIFEST_DIR").expect("source dir is unavailable"));
	let target = PathBuf::from(&env::var_os("OUT_DIR").expect("target dir is unavailable"));

	let mut source_zip = source.clone();
	source_zip.push("cldr.zip");
	let mut source_cldr = source.clone();
	source_cldr.push("cldr");

	let mut target_zip = target.clone();
	target_zip.push("cldr.zip");
	let mut target_cldr = target.clone();
	target_cldr.push("cldr");
	let mut target_countdown_html = target.clone();
	target_countdown_html.push("countdown.html");
	let mut target_new_html = target.clone();
	target_new_html.push("new.html");

	let has_zip = if fs::metadata(&target_zip)
		.map(|m| m.is_file())
		.unwrap_or(false)
	{
		eprintln!("Found cldr zip in target directory");
		true
	} else if fs::metadata(&source_zip)
		.map(|m| m.is_file())
		.unwrap_or(false)
	{
		eprintln!("Found cldr zip in project directory, copying to target directory ...");
		copy_dir::copy_dir(&source_zip, &target_zip).expect("can't copy to target directory");
		true
	} else {
		false
	};

	let has_cldr = if fs::read_dir(&target_cldr)
		.map(|d| d.count() > 20)
		.unwrap_or(false)
	{
		eprintln!("Found cldr directory in target directory");
		true
	} else if fs::read_dir(&source_cldr)
		.map(|d| d.count() > 20)
		.unwrap_or(false)
	{
		eprintln!("Found cldr directory in project directory");
		true
	} else {
		false
	};

	(has_zip, has_cldr)
}

fn download_zip() {
	eprintln!("Downloading CLDR data ...");
	let zip = reqwest::blocking::ClientBuilder::new()
		.timeout(Some(Duration::from_secs(1800)))
		.https_only(true)
		.build()
		.expect("http client building failed")
		.get(CLDR_ZIP_URL)
		.send()
		.expect("cldr zip was unable to be downloaded")
		.bytes()
		.expect("cldr zip was unable to be fully downloaded");

	eprintln!("CLDR data downloaded");
	let mut file = PathBuf::from(&env::var_os("OUT_DIR").expect("target dir is unavailable"));
	file.push("cldr.zip");
	let file = file;

	eprintln!("Writing CLDR data ...");
	fs::write(file, zip).expect("cldr zip file is unwritable");
	eprintln!("CLDR data written");
}

fn extract_cldr() {
	eprintln!("Extracting CLDR data ...");
	let mut icu = PathBuf::from(&env::var_os("OUT_DIR").expect("target dir is unavailable"));
	icu.push("cldr");
	let icu = icu;

	let mut zip_path = PathBuf::from(&env::var_os("OUT_DIR").expect("target dir is unavailable"));
	zip_path.push("cldr.zip");
	let zip_path = zip_path;

	zip_extract::extract(
		File::open(zip_path).expect("cldr zip is unreadable"),
		&icu,
		false,
	)
	.expect("cldr zip is un-decompressable");

	eprintln!("CLDR data extracted");
}

fn time_units() {
	eprintln!("Generating time unit data ...");
	let mut target = PathBuf::from(&env::var_os("OUT_DIR").expect("target dir is unavailable"));
	target.push("time_units.rs");
	let target = target;

	eprintln!("Reading time unit data locales ...");
	let mut source_a = PathBuf::from(&env::var_os("OUT_DIR").expect("target path is unavailable"));
	source_a.push("cldr");
	source_a.push("cldr-units-full");
	source_a.push("main");
	let source_a = source_a;

	let mut source_b =
		PathBuf::from(&env::var_os("CARGO_MANIFEST_DIR").expect("target path is unavailable"));
	source_b.push("cldr");
	source_b.push("cldr-units-full");
	source_b.push("main");
	let source_b = source_b;

	let locales = fs::read_dir(source_a)
		.or_else(|_| fs::read_dir(source_b))
		.expect("cldr directory is unaccessible")
		.collect::<Result<Vec<_>, _>>()
		.expect("cldr directory is unreadable")
		.into_iter()
		.map(|e| {
			(
				e.file_name()
					.to_str()
					.expect("locale name is invalid")
					.to_string(),
				{
					let mut path = e.path();
					path.push("units.json");
					fs::read_to_string(path).expect("units file is unreadable")
				},
			)
		})
		.collect::<Vec<_>>();

	#[cfg(feature = "few-time-units")]
	let locales = locales
		.into_iter()
		.filter(|(locale, _)| ["und", "en"].contains(&locale.as_str()))
		.collect::<Vec<_>>();

	let code = stringify! {
		use std::borrow::Cow;
		use serde::{Serialize, Deserialize};

		#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
		pub struct Units<'a> {
			#[serde(borrow)]
			pub days: Unit<'a>,
			#[serde(borrow)]
			pub hours: Unit<'a>,
			#[serde(borrow)]
			pub minutes: Unit<'a>,
			#[serde(borrow)]
			pub seconds: Unit<'a>,
		}

		#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
		pub struct Unit<'a> {
			#[serde(borrow)]
			pub zero: Cow<'a, str>,
			#[serde(borrow)]
			pub one: Cow<'a, str>,
			#[serde(borrow)]
			pub two: Cow<'a, str>,
			#[serde(borrow)]
			pub few: Cow<'a, str>,
			#[serde(borrow)]
			pub many: Cow<'a, str>,
			#[serde(borrow)]
			pub other: Cow<'a, str>,
		}

		impl<'a> Unit<'a> {
			/// This string will be used inside [`Unit`] when there's no data
			/// for a plural category. This avoids an up to 192-byte overhead on
			/// each [`Units`] compared to using `Option`s, without much of a
			/// semantic difference.
			#[allow(dead_code)]
			pub const NO_DATA: &'static str = "";

			/// Get the name of this unit for the given `PluralCategory`.
			/// Returns `Self::NO_DATA` if the name is not available for the
			/// given category.
			#[allow(dead_code)]
			pub fn get(&self, plural: icu::plurals::PluralCategory) -> &str {
				use icu::plurals::PluralCategory::*;

				match plural {
					Zero => &self.zero,
					One => &self.one,
					Two => &self.two,
					Few => &self.few,
					Many => &self.many,
					Other => &self.other,
				}
			}

			/// Get the name of this unit for the given `PluralCategory`.
			/// Prefer this method over `get` if you need the returned value to
			/// outlive `&self`, especially if the internal unit name strings
			/// are likely to be borrowed, e.g. if this unit is from the global
			/// constant units map. Returns `Self::NO_DATA` if the name is not
			/// available for the given category.
			#[allow(dead_code)]
			pub fn get_cow(&self, plural: icu::plurals::PluralCategory) -> Cow<'a, str> {
				use icu::plurals::PluralCategory::*;

				match plural {
					Zero => Cow::clone(&self.zero),
					One => Cow::clone(&self.one),
					Two => Cow::clone(&self.two),
					Few => Cow::clone(&self.few),
					Many => Cow::clone(&self.many),
					Other => Cow::clone(&self.other),
				}
			}
		}
	};

	const NO_DATA: &str = "";

	let mut map = Map::<&'static str>::new();

	eprintln!("Parsing time unit data ...");
	for (locale, units_json) in &locales {
		let units: serde_json::Value =
			serde_json::from_str(units_json).expect("units file is invalid");

		let [mut days, mut hours, mut minutes, mut seconds] =
			[Vec::new(), Vec::new(), Vec::new(), Vec::new()];

		for plural in ["zero", "one", "two", "few", "many", "other"] {
			let day = units
				.pointer(&format!(
					"/main/{locale}/units/long/duration-day/unitPattern-count-{plural}"
				))
				.map(|v| v.as_str().expect("unit value is not a string"));

			let hour = units
				.pointer(&format!(
					"/main/{locale}/units/long/duration-hour/unitPattern-count-{plural}"
				))
				.map(|v| v.as_str().expect("unit value is not a string"));

			let minute = units
				.pointer(&format!(
					"/main/{locale}/units/long/duration-minute/unitPattern-count-{plural}"
				))
				.map(|v| v.as_str().expect("unit value is not a string"));

			let second = units
				.pointer(&format!(
					"/main/{locale}/units/long/duration-second/unitPattern-count-{plural}"
				))
				.map(|v| v.as_str().expect("unit value is not a string"));

			days.push(
				day.unwrap_or(NO_DATA)
					.replace("{0}", "")
					.trim()
					.to_uppercase(),
			);
			hours.push(
				hour.unwrap_or(NO_DATA)
					.replace("{0}", "")
					.trim()
					.to_uppercase(),
			);
			minutes.push(
				minute
					.unwrap_or(NO_DATA)
					.replace("{0}", "")
					.trim()
					.to_uppercase(),
			);
			seconds.push(
				second
					.unwrap_or(NO_DATA)
					.replace("{0}", "")
					.trim()
					.to_uppercase(),
			);
		}

		let units = format!(
			r#"Units::<'static> {{
				days: Unit {{
					zero: Cow::Borrowed("{}"),
					one: Cow::Borrowed("{}"),
					two: Cow::Borrowed("{}"),
					few: Cow::Borrowed("{}"),
					many: Cow::Borrowed("{}"),
					other: Cow::Borrowed("{}"),
				}},
				hours: Unit {{
					zero: Cow::Borrowed("{}"),
					one: Cow::Borrowed("{}"),
					two: Cow::Borrowed("{}"),
					few: Cow::Borrowed("{}"),
					many: Cow::Borrowed("{}"),
					other: Cow::Borrowed("{}"),
				}},
				minutes: Unit {{
					zero: Cow::Borrowed("{}"),
					one: Cow::Borrowed("{}"),
					two: Cow::Borrowed("{}"),
					few: Cow::Borrowed("{}"),
					many: Cow::Borrowed("{}"),
					other: Cow::Borrowed("{}"),
				}},
				seconds: Unit {{
					zero: Cow::Borrowed("{}"),
					one: Cow::Borrowed("{}"),
					two: Cow::Borrowed("{}"),
					few: Cow::Borrowed("{}"),
					many: Cow::Borrowed("{}"),
					other: Cow::Borrowed("{}"),
				}}
			}}"#,
			days[0],
			days[1],
			days[2],
			days[3],
			days[4],
			days[5],
			hours[0],
			hours[1],
			hours[2],
			hours[3],
			hours[4],
			hours[5],
			minutes[0],
			minutes[1],
			minutes[2],
			minutes[3],
			minutes[4],
			minutes[5],
			seconds[0],
			seconds[1],
			seconds[2],
			seconds[3],
			seconds[4],
			seconds[5],
		);

		map.entry(locale, &units);
	}

	let map = map.build();

	eprintln!("Writing time unit data ...");

	let syntax_tree = syn::parse_file(&format!(
		"{code}\n\npub static UNITS: phf::Map<&'static str, Units> = {map};\n"
	))
	.unwrap();
	let formatted = prettyplease::unparse(&syntax_tree);

	File::create(target)
		.expect("can't open output file")
		.write_all(formatted.as_bytes())
		.expect("can't write output file");

	eprintln!("Time unit data generated");
}
