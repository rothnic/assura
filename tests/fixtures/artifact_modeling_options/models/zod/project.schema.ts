import { z } from "zod";

export const Goal = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  status: z.enum(["active", "completed"]),
  owners: z.array(z.string().min(1)).min(1),
  specs: z.array(z.string().min(1)).optional(),
});

export const Spec = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  status: z.enum(["draft", "active"]),
});

export const Decision = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  status: z.enum(["active", "superseded"]),
  supersedes: z.string().min(1).optional(),
});
