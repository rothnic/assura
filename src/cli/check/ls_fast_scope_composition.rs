/// Precompose overlapping static hierarchy scopes for one-match fast lookup.
fn compose_static_ancestor_scopes(scopes: &mut [FastScope], naming_cache: &mut FastNamingCache) {
    let has_composable_target = scopes.iter().any(|scope| {
        !scope.has_scope_magic && scope.inherit && scope.path.parent().is_some()
    });
    if !has_composable_target {
        return;
    }

    let by_path = scopes
        .iter()
        .enumerate()
        .filter(|(_, scope)| !scope.has_scope_magic)
        .map(|(index, scope)| (scope.path.clone(), index))
        .collect::<HashMap<_, _>>();
    let targets = scopes
        .iter()
        .enumerate()
        .filter(|(_, target)| !target.has_scope_magic && target.inherit)
        .filter(|(_, target)| {
            target
                .path
                .ancestors()
                .skip(1)
                .any(|ancestor| by_path.contains_key(ancestor))
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return;
    }

    let original = scopes.to_vec();
    for target_index in targets {
        let target = &mut scopes[target_index];
        let mut matching = target
            .path
            .ancestors()
            .filter_map(|ancestor| by_path.get(ancestor).and_then(|index| original.get(*index)))
            .collect::<Vec<_>>();
        if matching.len() < 2 {
            continue;
        }
        matching.reverse();

        let mut exact = EffectiveRules::default();
        let mut descendant = EffectiveRules::default();
        for scope in matching {
            let exact_rules = if scope.path == target.path {
                &scope.exact.effective
            } else {
                &scope.descendant.effective
            };
            exact = merge_fast_effective_rules(&exact, exact_rules, scope.inherit);
            descendant =
                merge_fast_effective_rules(&descendant, &scope.descendant.effective, scope.inherit);
        }
        target.exact = FastRules::new_with_cache(exact, naming_cache);
        target.descendant = FastRules::new_with_cache(descendant, naming_cache);
    }
}

fn merge_fast_effective_rules(
    parent: &EffectiveRules,
    child: &EffectiveRules,
    inherit: bool,
) -> EffectiveRules {
    if !inherit {
        return child.clone();
    }
    EffectiveRules {
        files: merge_file_bundle(parent.files.as_ref(), child.files.as_deref()),
        directories: merge_directory_bundle(
            parent.directories.as_ref(),
            child.directories.as_deref(),
        ),
        self_directory: merge_directory_bundle(
            parent.self_directory.as_ref(),
            child.self_directory.as_deref(),
        ),
        markdown: merge_markdown_bundle(parent.markdown.as_ref(), child.markdown.as_deref()),
        limit_children: child
            .limit_children
            .clone()
            .or_else(|| parent.limit_children.clone()),
    }
}
