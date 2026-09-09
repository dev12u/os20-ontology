//! JSON-LD is a *serialization* of named OS20 identities. It is not a second UID.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::engineering::graph::EngineeringGraph;
use crate::snapshot::OntologySnapshot;

/// JSON-LD document (`@context` + `@graph`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonLdDocument {
    /// Context object.
    #[serde(rename = "@context")]
    pub context: Value,
    /// Graph nodes.
    #[serde(rename = "@graph")]
    pub graph: Vec<Value>,
}

/// JSON-LD context for OS20 engineering v1. Identities remain `@scope/name#Local`.
pub fn context() -> Value {
    json!({
        "os20": "https://os20.org/ns/engineering/v1#",
        "prov": "http://www.w3.org/ns/prov#",
        "qudt": "http://qudt.org/schema/qudt/",
        "xsd": "http://www.w3.org/2001/XMLSchema#",
        "@vocab": "https://os20.org/ns/engineering/v1#",
        "instanceOf": { "@id": "os20:instanceOf", "@type": "@id" },
        "contains": { "@id": "https://os20.org/id/@os20/core#contains", "@type": "@id" },
        "hasGeometry": { "@id": "os20:hasGeometry", "@type": "@id" },
        "hasBehavior": { "@id": "os20:hasBehavior", "@type": "@id" },
        "hasPort": { "@id": "os20:hasPort", "@type": "@id" },
        "hasProperty": { "@id": "os20:hasProperty", "@type": "@id" },
        "madeOf": { "@id": "os20:madeOf", "@type": "@id" },
        "connects": { "@id": "os20:connects", "@type": "@id" },
        "allocatedTo": { "@id": "os20:allocatedTo", "@type": "@id" },
        "verifiedBy": { "@id": "os20:verifiedBy", "@type": "@id" },
        "simulates": { "@id": "os20:simulates", "@type": "@id" },
        "usesGeometry": { "@id": "os20:usesGeometry", "@type": "@id" },
        "usesMaterial": { "@id": "os20:usesMaterial", "@type": "@id" },
        "representedBy": { "@id": "os20:representedBy", "@type": "@id" },
        "providesEvidenceFor": { "@id": "os20:providesEvidenceFor", "@type": "@id" },
        "producesResult": { "@id": "os20:producesResult", "@type": "@id" },
        "generatedBy": { "@id": "os20:generatedBy", "@type": "@id" },
        "manufacturedBy": { "@id": "os20:manufacturedBy", "@type": "@id" },
        "producedBy": { "@id": "os20:producedBy", "@type": "@id" },
        "controls": { "@id": "os20:controls", "@type": "@id" },
        "executesOn": { "@id": "os20:executesOn", "@type": "@id" },
        "configures": { "@id": "os20:configures", "@type": "@id" },
        "selects": { "@id": "os20:selects", "@type": "@id" },
        "derivedFrom": { "@id": "prov:wasDerivedFrom", "@type": "@id" },
        "wasGeneratedBy": { "@id": "prov:wasGeneratedBy", "@type": "@id" },
        "quantityKind": { "@id": "qudt:hasQuantityKind", "@type": "@id" },
        "unit": { "@id": "qudt:unit", "@type": "@id" }
    })
}

/// Serialize a graph. `@id` is the frozen ElementId (UIDS-compatible in this checkout).
pub fn to_jsonld(_snapshot: &OntologySnapshot, graph: &EngineeringGraph) -> JsonLdDocument {
    let mut nodes: Vec<Value> = graph
        .nodes
        .values()
        .map(|n| {
            let mut map = Map::new();
            map.insert("@id".into(), json!(n.id.as_str()));
            map.insert("@type".into(), json!(n.ty.as_str()));
            if n.is_instance {
                if let Some(d) = &n.definition {
                    map.insert("instanceOf".into(), json!(d.as_str()));
                }
            }
            if let Some(d) = &n.direction {
                map.insert("direction".into(), json!(d));
            }
            if let Some(k) = &n.flow_kind {
                map.insert("flowKind".into(), json!(k));
            }
            if let Some(run) = &n.execution_run {
                map.insert("executionRun".into(), json!(run));
            }
            if let Some(a) = &n.artifact {
                map.insert(
                    "artifactDigest".into(),
                    json!({
                        "@id": a.digest,
                        "format": a.format
                    }),
                );
            }
            if let Some(p) = &n.providing_package {
                map.insert("providedByPackage".into(), json!(p));
            }
            for (k, v) in &n.properties {
                map.insert(k.clone(), property_json(v));
            }
            Value::Object(map)
        })
        .collect();

    for e in &graph.edges {
        let pred = local_name(e.predicate.as_str());
        if let Some(Value::Object(map)) = nodes
            .iter_mut()
            .find(|n| n.get("@id").and_then(Value::as_str) == Some(e.source.as_str()))
        {
            match map.get_mut(pred) {
                Some(Value::Array(arr)) => arr.push(json!({ "@id": e.target.as_str() })),
                Some(existing) => {
                    let prev = existing.take();
                    map.insert(pred.into(), json!([prev, { "@id": e.target.as_str() }]));
                }
                None => {
                    map.insert(pred.into(), json!({ "@id": e.target.as_str() }));
                }
            }
        }
    }

    nodes.sort_by(|a, b| {
        a.get("@id")
            .and_then(Value::as_str)
            .cmp(&b.get("@id").and_then(Value::as_str))
    });

    JsonLdDocument {
        context: context(),
        graph: nodes,
    }
}

fn local_name(id: &str) -> &str {
    id.rsplit('#').next().unwrap_or(id)
}

fn property_json(v: &crate::engineering::graph::EngineeringPropertyValue) -> Value {
    use crate::engineering::graph::EngineeringPropertyValue::*;
    match v {
        Quantity {
            quantity_kind,
            dimension,
            value,
            unit,
            origin,
        } => json!({
            "quantityKind": quantity_kind,
            "dimension": dimension,
            "value": value,
            "unit": unit,
            "origin": origin
        }),
        Boolean { value } => json!(value),
        Enum { value } | Text { value } => json!(value),
        Reference { target } => json!({ "@id": target }),
    }
}
