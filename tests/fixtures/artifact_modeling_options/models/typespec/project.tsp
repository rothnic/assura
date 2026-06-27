namespace Assura.Artifacts;

alias ArtifactStatus = "draft" | "active" | "completed" | "superseded";

model Goal {
  @doc("Stable artifact identifier.")
  @minLength(1)
  id: string;

  @minLength(1)
  title: string;

  status: "active" | "completed";

  @minItems(1)
  owners: string[];

  @doc("Referenced Spec identifiers.")
  specs?: string[];
}

model Spec {
  @minLength(1)
  id: string;
  @minLength(1)
  title: string;
  status: "draft" | "active";
}

model Decision {
  @minLength(1)
  id: string;
  @minLength(1)
  title: string;
  status: "active" | "superseded";
  @minLength(1)
  supersedes?: string;
}
