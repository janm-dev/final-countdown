# Final Countdown

A simple timer webapp with emphasis on internationalization.

## Link structure

`https://{domain}/{target_timestamp}/{title}?{options}` (title and options are optional)

### `domain`

Wherever this is hosted, doesn't change the behaviour.

### `target_timestamp`

The timestamp of the countdown target. Can be a number (unix timestamp) or an RFC 3339 timestamp.

### `title`

The optional title for the countdown. Shown on the page with `_` in the url replaced with `[space]` on the page.

### `options`

- `simple` - only show the above-the-fold content
- `tz={timezone}` - display the target time in this timezone (can be repeated)

## Layout

### Countdown page

```txt
+---------------------------+
|          {title}          |
|                           |
|        {time_left}        |
|      {units_of_time}      |
+---------------------------+ <- Fold
| {link_to_creating_new_cd} |
|  {target_local_timezone}  |
|  {target_in_tz_timezone}  |
|  {target_in_tz_timezone}  |
|           {...}           |
|   {target_in_other_tzs}   |
|   {target_in_other_tzs}   |
|   {target_in_other_tzs}   |
|   {target_in_other_tzs}   |
|                           |
|   {footer}       {info}   |
+---------------------------+
```

### Root page

```txt
+---------------------------+
|      {date_selector}      |
|       {title_input}       |
|      {share_buttons}      |
|  {copy_pastable_cd_link}  |
|                           |
|   {footer}       {info}   |
+---------------------------+
```

## Attribution

For Cargo dependencies, you can use [`cargo about generate -o ATTRIBUTION.html about.hbs`](https://github.com/EmbarkStudios/cargo-about) to generate an html file with information about dependencies and their licenses.

In addition to Cargo dependencies, the following additional resources are used in Final Countdown:

- The font used for the timer (`assets/font-numbers.woff2`) is a subset of [Rubik Mono One](https://fonts.google.com/specimen/Rubik+Mono+One), used and redistributed under the terms of the [Open Font License](https://scripts.sil.org/cms/scripts/page.php?site_id=nrsi&id=OFL).
- The font used for other text (`assets/font-text.woff2`) is adapted from [Noto Sans](https://fonts.google.com/noto/specimen/Noto+Sans), used and redistributed under the terms of the [Open Font License](https://scripts.sil.org/cms/scripts/page.php?site_id=nrsi&id=OFL).
- The Final Countdown icon (`assets/icon.ico`, `assets/icon.png`, and `assets/icon.svg`) is based on the [Acute icon from Material Symbols](https://fonts.google.com/icons?icon.query=Acute&selected=Material+Symbols+Outlined:acute:FILL@0;wght@400;GRAD@0;opsz@24), used under the terms of the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
- Other website icons (as part of `assets/new.html`) are from [Material Symbols](https://fonts.google.com/icons), used under the terms of the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).

## [License](./LICENSE)

With the exception of the files listed above in [Attribution](#attribution), this program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License (`AGPL-3.0-or-later`) as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
