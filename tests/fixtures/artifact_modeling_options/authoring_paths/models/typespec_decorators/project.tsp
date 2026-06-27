using TypeSpec.Reflection;

extern dec id(target: ModelProperty);
extern dec ref(target: ModelProperty, targetType: valueof string);
extern dec collection(target: Model, path: valueof string);
extern dec adapter(target: Model, adapter: valueof string);

enum ArtifactStatus {
  active,
  complete,
  deprecated,
}

enum RiskLevel {
  low,
  medium,
  high,
}

enum Priority {
  p0,
  p1,
  p2,
}

enum EvidenceKind {
  file,
  url,
}

model ArtifactMetadata {
  summary: string;
  risk: RiskLevel;
  tags?: string[];
}

model Evidence {
  kind: EvidenceKind;
  value: string;
}

@collection("docs/goals/*.md")
@adapter("markdown_frontmatter")
model Goal {
  @id
  id: string;
  title: string;
  status: ArtifactStatus;
  owners: string[];
  @ref("Spec")
  specs?: string[];
  @ref("Task")
  tasks?: string[];
  @ref("Decision")
  decisions?: string[];
  metadata: ArtifactMetadata;
}

@collection("specs/*.json")
@adapter("json_record")
model Spec {
  @id
  id: string;
  title: string;
  status: ArtifactStatus;
  owner: string;
  metadata: ArtifactMetadata;
  @ref("Decision")
  decisions?: string[];
}

@collection("tasks/*.yaml")
@adapter("yaml_record")
model Task {
  @id
  id: string;
  title: string;
  status: ArtifactStatus;
  @ref("Goal")
  goal: string;
  @ref("Spec")
  spec?: string;
  assignee?: string;
  priority: Priority;
  evidence?: Evidence[];
  metadata: ArtifactMetadata;
}

@collection("decisions/*.jsonl")
@adapter("jsonl_record")
model Decision {
  @id
  id: string;
  title: string;
  status: ArtifactStatus;
  @ref("Decision")
  supersedes?: string;
  @ref("Spec")
  affects_specs?: string[];
  @ref("Task")
  affects_tasks?: string[];
  metadata: ArtifactMetadata;
}
