//! # Fornjot Application
//!
//! This library is part of the [Fornjot] ecosystem. Fornjot is an open-source,
//! code-first CAD application; and collection of libraries that make up the CAD
//! application, but can be used independently.
//!
//! Together with the [`fj`] library, this application forms the part of Fornjot
//! that is relevant to end users. Please refer to the [Fornjot repository] for
//! usage examples.
//!
//! [Fornjot]: https://www.fornjot.app/
//! [`fj`]: https://crates.io/crates/fj
//! [Fornjot repository]: https://github.com/hannobraun/Fornjot

mod args;
mod config;

use std::path::PathBuf;

use anyhow::{anyhow, Context as _};
use fj_export::export;
use fj_host::{Model, Parameters};
use fj_interop::status_report::StatusReport;
use fj_operations::shape_processor::ShapeProcessor;
use fj_window::run::run;
use tracing_subscriber::fmt::format;
use tracing_subscriber::EnvFilter;

use crate::{args::Args, config::Config};

fn main() -> anyhow::Result<()> {
    let mut status = StatusReport::new();
    // Respect `RUST_LOG`. If that's not defined or erroneous, log warnings and
    // above.
    //
    // It would be better to fail, if `RUST_LOG` is erroneous, but I don't know
    // how to distinguish between that and the "not defined" case.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("WARN")),
        )
        .event_format(format().pretty())
        .init();

    let args = Args::parse();
    let config = Config::load()?;

    let path = config.default_path.unwrap_or_else(|| PathBuf::from(""));
    let parameters = args.parameters.unwrap_or_else(Parameters::empty);
    let shape_processor = ShapeProcessor {
        tolerance: args.tolerance,
    };

    let model = if let Some(model) = args.model.or(config.default_model) {
        let mut model_path = path;
        model_path.push(model);
        Some(Model::from_path(model_path.clone()).with_context(|| {
            format!("Failed to load model: {}", model_path.display())
        })?)
    } else {
        None
    };

    if let Some(export_path) = args.export {
        // export only mode. just load model, process, export and exit

        let model = model.ok_or_else(|| {
            anyhow!(
                "No model specified, and no default model configured.\n\
            Specify a model by passing `--model path/to/model`."
            )
        })?;

        let shape = model.load_once(&parameters, &mut status)?;
        let shape = shape_processor.process(&shape)?;

        export(&shape.mesh, &export_path)?;

        return Ok(());
    }

    if let Some(model) = model {
        let watcher = model.load_and_watch(parameters)?;
        run(Some(watcher), shape_processor, status)?;
    } else {
        run(None, shape_processor, status)?;
    }

    Ok(())
}
