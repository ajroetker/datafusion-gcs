// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![warn(missing_docs)]

//! [DataFusion-ObjectStore-GCS](https://github.com/datafusion-contrib/datafusion-objectstore-s3)
//! provides a `TableProvider` interface for using `Datafusion` to query data in GCS.  This includes GCS
//! and services such as MinIO that implement the GCS API.
//!
//! ## Examples
//!
//! Load credentials from default GCS credential provider (such as environment)
//!
//! ```rust
//! # use std::sync::Arc;
//! # use datafusion::error::Result;
//! # use datafusion_objectstore_gcs::object_store::gcs::GCSFileSystem;
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! let gcs_file_system = Arc::new(GCSFileSystem::default().await);
//! # Ok(())
//! # }
//! ```
//!
//! `GCSFileSystem::default()` is a convenience wrapper for `GCSFileSystem::new(None, None, None, None, None, None)`.
//!
//! ```rust
//! use datafusion_objectstore_gcs::object_store::gcs::GCSFileSystem;
//!
//! # #[tokio::main]
//! # async fn main() {
//!
//! let gcs_file_system = GCSFileSystem::new().await;
//! # }
//! ```
//!
//! Using DataFusion's `ListingOtions` and `ListingTable` we register a table into a DataFusion `ExecutionContext` so that it can be queried.
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use datafusion::datasource::listing::*;
//! use datafusion::datasource::TableProvider;
//! use datafusion::prelude::SessionContext;
//! use datafusion::datasource::file_format::parquet::ParquetFormat;
//! use datafusion::error::Result;
//!
//! use datafusion_objectstore_gcs::object_store::gcs::GCSFileSystem;
//!
//! use http::Uri;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! let filename = "gcs://data/alltypes_plain.snappy.parquet";
//!
//! # let gcs_file_system = Arc::new(GCSFileSystem::new().await);
//!
//! let config = ListingTableConfig::new(gcs_file_system, filename).infer().await?;
//!
//! let table = ListingTable::try_new(config)?;
//!
//! let mut ctx = SessionContext::new();
//!
//! ctx.register_table("tbl", Arc::new(table))?;
//!
//! let df = ctx.sql("SELECT * FROM tbl").await?;
//! df.show();
//! # Ok(())
//! # }
//! ```
//!
//! We can also register the `GCSFileSystem` directly as an `ObjectStore` on an `ExecutionContext`. This provides an idiomatic way of creating `TableProviders` that can be queried.
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use datafusion::datasource::listing::*;
//! use datafusion::error::Result;
//!
//! use datafusion_objectstore_gcs::object_store::gcs::GCSFileSystem;
//!
//! use datafusion::prelude::SessionContext;
//! use http::Uri;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let gcs_file_system = Arc::new(
//!         GCSFileSystem::new().await,
//!     );
//!
//!     let ctx = SessionContext::new();
//!
//!     let uri = "gsc://data/alltypes_plain.snappy.parquet";
//!
//!     let config = ListingTableConfig::new(gcs_file_system, uri)
//!         .infer()
//!         .await?;
//!
//!     let table = ListingTable::try_new(config)?;
//!
//!     ctx.register_table("tbl", Arc::new(table))?;
//!
//!     let df = ctx.sql("SELECT * FROM tbl").await?;
//!     df.show().await?;
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod object_store;
