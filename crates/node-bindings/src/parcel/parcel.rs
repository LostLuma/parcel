use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use napi::Env;
use napi::JsFunction;
use napi::JsObject;
use napi::JsUnknown;
use napi_derive::napi;

use parcel::file_system::FileSystemRef;
use parcel::rpc::nodejs::NodejsWorker;
use parcel::rpc::nodejs::RpcHostNodejs;
use parcel::rpc::RpcHostRef;
use parcel::Parcel;
use parcel_core::types::ParcelOptions;
use parcel_napi_helpers::JsTransferable;
use parcel_package_manager::PackageManagerRef;

use crate::file_system::FileSystemNapi;

use super::package_manager_napi::PackageManagerNapi;
use super::tracer::Tracer;
use super::tracer::TracerMode;

#[napi(object)]
pub struct ParcelNapiBuildOptions {
  pub register_worker: JsFunction,
}

#[napi(object)]
pub struct ParcelNapiBuildResult {}

#[napi(object)]
pub struct ParcelNapiOptions {
  pub fs: Option<JsObject>,
  pub node_workers: Option<u32>,
  pub options: JsObject,
  pub package_manager: Option<JsObject>,
  pub threads: Option<u32>,
  pub tracer_options: Option<JsObject>,
}

#[napi]
pub struct ParcelNapi {
  pub node_worker_count: u32,
  fs: Option<FileSystemRef>,
  options: ParcelOptions,
  package_manager: Option<PackageManagerRef>,
  rpc: Option<RpcHostRef>,
  tracer: Tracer,
  tx_worker: Sender<NodejsWorker>,
}

#[napi]
impl ParcelNapi {
  #[napi(constructor)]
  pub fn new(napi_options: ParcelNapiOptions, env: Env) -> napi::Result<Self> {
    // Debugging Instrumentation
    let tracer_mode = TracerMode::from_js_value(&env, napi_options.tracer_options)?;
    // Tracer can fail to initialize when Parcel is constructed multiple times, ignore subsequent failures
    let tracer = Tracer::new(tracer_mode).unwrap_or_else(|_| Tracer::dummy());

    let thread_id = std::thread::current().id();
    tracing::trace!(?thread_id, "parcel-napi initialize");

    // Wrap the JavaScript-supplied FileSystem
    let fs: Option<FileSystemRef> = if let Some(fs) = napi_options.fs {
      Some(Arc::new(FileSystemNapi::new(&env, &fs)?))
    } else {
      None
    };

    let package_manager: Option<PackageManagerRef> = if let Some(pm) = napi_options.package_manager
    {
      Some(Arc::new(PackageManagerNapi::new(&env, &pm)?))
    } else {
      None
    };

    // Assign Rust thread count from JavaScript
    let threads = napi_options
      .threads
      .map(|t| t as usize)
      .unwrap_or_else(|| num_cpus::get());

    // Set up Nodejs plugin bindings
    let node_worker_count = napi_options
      .node_workers
      .map(|w| w as usize)
      .unwrap_or_else(|| threads);

    let (tx_worker, rx_worker) = channel::<NodejsWorker>();
    let rpc_host_nodejs = RpcHostNodejs::new(node_worker_count, rx_worker)?;
    let rpc = Some::<RpcHostRef>(Arc::new(rpc_host_nodejs));

    Ok(Self {
      fs,
      node_worker_count: node_worker_count as u32,
      options: env.from_js_value(napi_options.options)?,
      package_manager,
      rpc,
      tracer,
      tx_worker,
    })
  }

  #[napi]
  pub fn build(&self, env: Env, options: ParcelNapiBuildOptions) -> napi::Result<JsObject> {
    let (deferred, promise) = env.create_deferred()?;

    for _ in 0..self.node_worker_count {
      let transferable = JsTransferable::new(self.tx_worker.clone());
      options
        .register_worker
        .call1::<JsTransferable<Sender<NodejsWorker>>, JsUnknown>(transferable)?;
    }

    // Both the parcel initialization and build must be run a dedicated system thread so that
    // the napi threadsafe functions do not panic
    thread::spawn({
      let fs = self.fs.clone();
      let options = self.options.clone();
      let package_manager = self.package_manager.clone();
      let rpc = self.rpc.clone();

      move || {
        let parcel = Parcel::new(fs, options, package_manager, rpc);
        let to_napi_error = |error| napi::Error::from_reason(format!("{:?}", error));

        match parcel {
          Err(error) => deferred.reject(to_napi_error(error)),
          Ok(parcel) => match parcel.build() {
            Ok(_result) => deferred.resolve(|_env| Ok(ParcelNapiBuildResult {})),
            Err(error) => deferred.reject(to_napi_error(error)),
          },
        }
      }
    });

    Ok(promise)
  }
}
