//! Bounded project-intelligence context-pack assembly.

use super::agent_query::{diagnostics, safe_fixes};
use super::context::{ContentQueryError, QueryContext};
use super::output::{
    ContextPackBoundsOutput, ContextPackOmissionOutput, ContextPackOutput,
    ContextPackRequestOutput, ContextPackTruncationOutput,
};
use super::{expand, missing_relations, search, show_instance};

const CONTEXT_PACK_SCHEMA: &str = "assura.project-intelligence.context-pack.v1";

pub(super) struct ContextPackRequest<'a> {
    pub(super) collection: Option<&'a String>,
    pub(super) id: Option<&'a String>,
    pub(super) text: Option<&'a String>,
    pub(super) limit: usize,
}

pub(super) fn context_pack(
    context: &QueryContext,
    request: ContextPackRequest<'_>,
) -> Result<ContextPackOutput, ContentQueryError> {
    let mode = request_mode(&request)?;
    let mut bounds = BoundsBuilder::new(request.limit);
    let mut diagnostics = diagnostics(context).diagnostics;
    bounds.truncate("diagnostics", &mut diagnostics);

    let mut missing_relations = missing_relations(context).missing_relations;
    bounds.truncate("missing_relations", &mut missing_relations);

    let mut safe_fixes = safe_fixes(context).safe_fixes;
    bounds.truncate("safe_fixes", &mut safe_fixes);

    let instance = match (request.collection, request.id) {
        (Some(collection), Some(id)) => {
            let mut instance = show_instance(context, collection, id)?;
            bounds.truncate(
                "instance.outgoing_relations",
                &mut instance.outgoing_relations,
            );
            bounds.truncate(
                "instance.incoming_relations",
                &mut instance.incoming_relations,
            );
            bounds.truncate("instance.diagnostics", &mut instance.diagnostics);
            bounds.truncate("instance.sections", &mut instance.sections);
            Some(instance)
        }
        (None, None) => {
            bounds.omit(
                "instance",
                "provide --collection and --id for object-oriented context",
            );
            None
        }
        _ => {
            return Err(ContentQueryError::configuration(
                "context-pack requires --collection and --id together",
            ));
        }
    };

    let related = match (request.collection, request.id) {
        (Some(collection), Some(id)) => {
            let mut related = expand(context, collection, id, usize::MAX)?;
            bounds.truncate("related.related", &mut related.related);
            Some(related)
        }
        _ => {
            bounds.omit(
                "related",
                "provide --collection and --id for graph expansion",
            );
            None
        }
    };

    let search = match request.text {
        Some(text) => {
            let mut search = search(context, text);
            bounds.truncate("search.matches", &mut search.matches);
            Some(search)
        }
        None => {
            bounds.omit("search", "provide --text for keyword search context");
            None
        }
    };

    Ok(ContextPackOutput {
        schema: CONTEXT_PACK_SCHEMA,
        request: ContextPackRequestOutput {
            mode,
            cli: "assura content context-pack",
            project_root: context.project_root.clone(),
            config_path: context.config_path.clone(),
            collection: request.collection.cloned(),
            id: request.id.cloned(),
            text: request.text.cloned(),
            limit: request.limit,
        },
        bounds: bounds.finish(),
        diagnostics,
        instance,
        related,
        search,
        missing_relations,
        safe_fixes,
    })
}

fn request_mode(request: &ContextPackRequest<'_>) -> Result<&'static str, ContentQueryError> {
    match (request.collection, request.id) {
        (Some(_), Some(_)) => Ok("object"),
        (None, None) => Ok("diagnostics"),
        _ => Err(ContentQueryError::configuration(
            "context-pack requires --collection and --id together",
        )),
    }
}

struct BoundsBuilder {
    limit: usize,
    truncated: Vec<ContextPackTruncationOutput>,
    omissions: Vec<ContextPackOmissionOutput>,
}

impl BoundsBuilder {
    fn new(limit: usize) -> Self {
        Self {
            limit,
            truncated: Vec::new(),
            omissions: Vec::new(),
        }
    }

    fn truncate<T>(&mut self, field: &'static str, values: &mut Vec<T>) {
        let original_count = values.len();
        if original_count > self.limit {
            values.truncate(self.limit);
            self.truncated.push(ContextPackTruncationOutput {
                field,
                original_count,
                returned_count: values.len(),
            });
        }
    }

    fn omit(&mut self, field: &'static str, reason: &'static str) {
        self.omissions
            .push(ContextPackOmissionOutput { field, reason });
    }

    fn finish(self) -> ContextPackBoundsOutput {
        ContextPackBoundsOutput {
            limit: self.limit,
            truncated: self.truncated,
            omissions: self.omissions,
        }
    }
}
