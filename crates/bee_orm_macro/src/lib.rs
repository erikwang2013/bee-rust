// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! `#[derive(Model)]` for `bee_orm::Model`.
//!
//! Struct level: `#[bee(table = "name")]` overrides the table name (default is
//! the struct name lowercased plus `s`); `#[bee(hooks(before_insert, …))]`
//! forwards each listed lifecycle hook to a same-named inherent async method
//! the user writes (`async fn hook(&self) -> bee_orm::Result<()>`);
//! `#[bee(m2m(Target))]`, repeatable, declares a many-to-many join table
//! towards `Target`. Its defaults are the lowercased idents — table
//! `{local}_{target}`, columns `{local}_id` / `{target}_id` — overridden by
//! `table = "…"`, `local = "…"`, `foreign = "…"`; a target whose two column
//! defaults collide (the model itself, `Self`) needs the explicit columns.
//! Field level, combinable in one attribute (`#[bee(pk, auto)]`):
//!
//! - `column = "name"` — column override, must match `[A-Za-z_][A-Za-z0-9_]*`
//! - `pk` — primary key; falls back to the field named `id` when unmarked
//! - `auto` — database-assigned, skipped by `insert_values`
//! - `ignore` — not a table column; excluded from every read/write and built
//!   with `Default::default()` in `from_row`
//! - `auto_now_add` / `auto_now` — unix-seconds timestamp columns injected by
//!   the trait (insert only / insert and update); excluded from both value
//!   lists. The field type must decode an integer (`i64` / `Option<i64>`)
//! - `soft_delete` — the soft-delete flag column, stored as a `bool`; stays an
//!   ordinary writable column, and rows whose flag is NULL stay invisible to
//!   default queries
//! - `sql_type = "..."` — raw SQL type for `columns()`, bypassing the type
//!   spelling table; required for spellings without a mapping (`u64`, custom
//!   types) and for a non-integer `auto` primary key
//! - `fk = Target` — foreign key: `columns()` records the target model's table
//!   and primary-key column, and the target must derive `Model`

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod expand;
mod parse;
mod types;

use expand::expand;

/// Derive `bee_orm::Model` for a struct.
#[proc_macro_derive(Model, attributes(bee))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(expanded) => expanded.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[cfg(test)]
mod tests;
