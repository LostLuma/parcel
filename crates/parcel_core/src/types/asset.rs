use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

use super::bundle::BundleBehavior;
use super::environment::Environment;
use super::file_type::FileType;
use super::json::JSONObject;
use super::symbol::Symbol;

#[derive(PartialEq, Hash, Clone, Copy, Debug)]
pub struct AssetId(pub NonZeroU32);

/// The source code for an asset.
#[derive(PartialEq, Default, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", transparent)]
pub struct Code {
  inner: String,
}

impl Code {
  pub fn bytes(&self) -> &[u8] {
    self.inner.as_bytes()
  }

  pub fn size(&self) -> u32 {
    self.inner.len() as u32
  }
}

impl Display for Code {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.inner)
  }
}

impl From<String> for Code {
  fn from(value: String) -> Self {
    Self { inner: value }
  }
}

/// An asset is a file or part of a file that may represent any data type including source code, binary data, etc.
///
/// Note that assets may exist in the file system or virtually.
///
#[derive(Default, PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
  /// The file type of the asset, which may change during transformation
  #[serde(rename = "type")]
  pub asset_type: FileType,

  /// Controls which bundle the asset is placed into
  pub bundle_behavior: BundleBehavior,

  /// The environment of the asset
  pub env: Arc<Environment>,

  /// The file path to the asset
  pub file_path: PathBuf,

  /// The code of this asset, initially read from disk, then becoming the
  /// transformed output
  pub code: Arc<Code>,

  /// Plugin specific metadata for the asset
  pub meta: JSONObject,

  /// The pipeline defined in .parcelrc that the asset should be processed with
  pub pipeline: Option<String>,

  /// The transformer options for the asset from the dependency query string
  pub query: Option<String>,

  /// Statistics about the asset
  pub stats: AssetStats,

  /// The symbols that the asset exports
  pub symbols: Vec<Symbol>,

  /// A unique key that identifies an asset
  ///
  /// When a transformer returns multiple assets, it can give them unique keys to identify them.
  /// This can be used to find assets during packaging, or to create dependencies between multiple
  /// assets returned by a transformer by using the unique key as the dependency specifier.
  ///
  /// TODO: Make this non-nullable and disallow creating assets without it.
  pub unique_key: Option<String>,

  /// Whether this asset can be omitted if none of its exports are being used
  ///
  /// This is initially set by the resolver, but can be overridden by transformers.
  ///
  pub side_effects: bool,

  /// Indicates if the asset is used as a bundle entry
  ///
  /// This controls whether a bundle can be split into multiple, or whether all of the
  /// dependencies must be placed in a single bundle.
  ///
  pub is_bundle_splittable: bool,

  /// Whether this asset is part of the project, and not an external dependency
  ///
  /// This indicates that transformation using the project configuration should be applied.
  ///
  pub is_source: bool,

  /// True if the asset has CommonJS exports
  pub has_cjs_exports: bool,

  /// This is true unless the module is a CommonJS module that does non-static access of the
  /// `this`, `exports` or `module.exports` objects. For example if the module uses code like
  /// `module.exports[key] = 10`.
  pub static_exports: bool,

  /// TODO: MISSING DOCUMENTATION
  pub should_wrap: bool,

  /// TODO: MISSING DOCUMENTATION
  pub has_node_replacements: bool,

  /// True if this is a 'constant module', meaning it only exports constant assignment statements,
  /// on this case this module may be inlined on its usage depending on whether it is only used
  /// once and the parcel configuration.
  ///
  /// An example of a 'constant module' would be:
  ///
  /// ```skip
  /// export const MY_CONSTANT = 'some-value';
  /// ```
  pub is_constant_module: bool,

  /// True if `Asset::symbols` has been populated. This field is deprecated and should be phased
  /// out.
  pub has_symbols: bool,
}

impl Asset {
  pub fn id(&self) -> u64 {
    let mut hasher = crate::hash::IdentifierHasher::default();

    self.asset_type.hash(&mut hasher);
    self.env.hash(&mut hasher);
    self.file_path.hash(&mut hasher);
    self.pipeline.hash(&mut hasher);
    self.query.hash(&mut hasher);
    self.unique_key.hash(&mut hasher);

    hasher.finish()
  }

  pub fn set_interpreter(&mut self, shebang: impl Into<serde_json::Value>) {
    self.meta.insert("interpreter".into(), shebang.into());
  }

  pub fn set_has_cjs_exports(&mut self, value: bool) {
    self.meta.insert("hasCJSExports".into(), value.into());
    self.has_cjs_exports = value;
  }

  pub fn set_static_exports(&mut self, value: bool) {
    self.meta.insert("staticExports".into(), value.into());
    self.static_exports = value;
  }

  pub fn set_should_wrap(&mut self, value: bool) {
    self.meta.insert("shouldWrap".into(), value.into());
    self.should_wrap = value;
  }
  pub fn set_is_constant_module(&mut self, is_constant_module: bool) {
    self.is_constant_module = is_constant_module;
    if is_constant_module {
      self.meta.insert("isConstantModule".into(), true.into());
    }
  }

  pub fn set_has_node_replacements(&mut self, has_node_replacements: bool) {
    self.has_node_replacements = has_node_replacements;
    if has_node_replacements {
      self
        .meta
        // This is intentionally snake_case as that's what it was originally.
        .insert("has_node_replacements".into(), true.into());
    }
  }
}

/// Statistics that pertain to an asset
#[derive(PartialEq, Clone, Debug, Default, Deserialize, Serialize)]
pub struct AssetStats {
  pub size: u32,
  pub time: u32,
}
