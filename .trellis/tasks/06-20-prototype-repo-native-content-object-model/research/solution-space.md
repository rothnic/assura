# Solution Space

## Categories

The traditional category closest to the requirement is a content repository:
a hierarchical store for structured and unstructured content with typed nodes,
properties, references, query, versioning, and observation. The Java Content
Repository lineage and Apache Jackrabbit/Oak are useful conceptual references,
but they are not a practical dependency target for Assura because they are
Java- and repository-server-oriented.

Git-backed CMS and file-based CMS tools are the closest product references.
They keep content in files while offering schemas, collections, editing
interfaces, and sometimes references. They usually focus on website content and
human editing rather than repo-wide validation and agent-safe mutation.

Static content collection systems are a narrower read/build-time variant. They
validate Markdown/MDX/data files and generate typed query APIs, but usually do
not provide safe write APIs or repo-wide relationship semantics.

Embedded document/object stores solve persistence and querying, not the
repo-file object model. They can help once Assura has normalized objects, but
they do not replace format adapters or source-preserving writes.

## Sources

* Apache Jackrabbit describes a content repository as a hierarchical content
  store for structured and unstructured content with search, versioning,
  transactions, and observation: https://jackrabbit.apache.org/
* Jackrabbit Oak targets scalable hierarchical content repositories:
  https://jackrabbit.apache.org/oak/
* Keystatic positions itself as editing Markdown, JSON, and YAML content in a
  codebase locally or through GitHub: https://keystatic.com/
* Astro Content Collections validate collection files with schemas and generate
  TypeScript types: https://docs.astro.build/en/guides/content-collections/

## Assura Fit

Assura should probably name this capability as a repo-native content repository
or writable content collections layer. The core differentiator is that files,
directories, parsed content, references, validation, and controlled writes all
belong to the repository model.
