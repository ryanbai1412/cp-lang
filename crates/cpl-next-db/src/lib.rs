//! The new pipeline's query database (handbook/design/next/incremental.md): the database
//! itself, and the item queries (`parse`, `items`, `item`, `item_at`) every layer keys its
//! own queries on. It sits below the layers, so each defines its queries in its own crate.
pub mod db;
pub mod items;
