// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci Export Engine (Phase 7):
//! FCPXML/native XML export, export queue, HDR export, project bundle export.

pub mod analytics;
pub mod error;
pub mod fcpxml;
pub mod hdr;
pub mod options;
pub mod project_bundle;
pub mod queue;
pub mod service;
pub mod xml;

pub use analytics::ExportTimelineAnalyticsSnapshot;
pub use error::{ExportError, ExportResult};
pub use fcpxml::FCPXMLExporter;
pub use hdr::{HdrConfig, HdrTransfer};
pub use options::{
    ExportFormat, ExportResolution, FCPXMLTarget, FCPXMLVersion, TimelineExportFormat, VideoCodec,
    VideoExportOptions,
};
pub use project_bundle::{MissingMedia, PalmierProjectExporter, ProjectBundleReport};
pub use queue::{
    ExportJob, ExportJobSource, ExportJobStatus, ExportQueue, ExportQueueSubmission,
};
pub use service::ExportService;
pub use xml::XMLExporter;
