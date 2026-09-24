//! Shared handling of the pass-through field every model carries.
//!
//! Request and result models both keep an `extra` map so a worker parameter or
//! a worker field this release does not declare stays reachable. Both therefore
//! need the same rule for what happens when a key in that map is one the model
//! already declares.

use std::collections::BTreeMap;

use serde::{Serialize, Serializer};
use serde_json::Value;

/// Serializes a model's declared fields, then adds the pass-through keys they
/// did not already occupy.
///
/// A declared field always wins. On a request that matters because the method a
/// caller reached for is what chose the parameter, and the task is billed
/// whether or not the value was the one they meant. On a result it matters
/// because `Solved` is documented as storable, and a round trip that swaps a
/// decoded token for a pass-through value corrupts the record silently.
///
/// A key the declared fields did not emit stays available: an unset optional is
/// skipped during serialization, never reaches the map, and the pass-through
/// value survives. The Go and Python SDKs resolve the same collision this way.
///
/// Rendering through `serde_json` is what lets the merge compare real keys
/// rather than a stream of events. Every endpoint this crate talks to speaks
/// JSON, so no other format is given up.
pub(crate) fn merge_extra<F, S>(
    declared: F,
    extra: &BTreeMap<String, Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    F: FnOnce(serde_json::value::Serializer) -> Result<Value, serde_json::Error>,
    S: Serializer,
{
    let rendered = declared(serde_json::value::Serializer).map_err(serde::ser::Error::custom)?;
    let Value::Object(mut map) = rendered else {
        return Err(serde::ser::Error::custom(
            "model must serialize as an object",
        ));
    };
    for (key, value) in extra {
        // or_insert rather than insert: the declared fields land first, and this
        // only fills the keys they left free.
        map.entry(key.as_str()).or_insert_with(|| value.clone());
    }
    map.serialize(serializer)
}
