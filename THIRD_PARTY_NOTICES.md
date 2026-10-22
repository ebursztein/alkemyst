# Third-party notices

Original Alkemyst code: Copyright 2026 Elie Bursztein, Apache-2.0.

## landlord

The maintained native engine under `src/landlord` derives from
[mtgoncurve/landlord](https://github.com/mtgoncurve/landlord), revision
`18e995af042e61a947c9347d1aefd5a2ec93f9e9` (upstream library 2025.4.1).
Copyright (c) 2020 Justin Shrake. MIT licensed; the complete original license is
preserved in `src/landlord/LICENSE`. See `src/landlord/UPSTREAM.md` for the adoption
map and intentional behavior changes. This derivative crate remains MIT licensed;
Alkemyst's original application code remains Apache-2.0.

Dependency licenses remain applicable.

## Runtime card metadata

Alkemyst can download public card metadata from Scryfall's bulk API. Scryfall
and Magic: The Gathering card names, rules and artwork retain their respective
owners' rights. This project's Apache-2.0 license covers Alkemyst's original code;
it does not relicense card data, images or installed Arena assets. Scryfall bulk
files and local game databases are not distributed in this repository.
