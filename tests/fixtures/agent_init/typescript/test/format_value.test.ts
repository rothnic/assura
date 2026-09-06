import { expect, test } from "bun:test";
import { formatValue } from "../src/format_value";

test("formats an existing utility value", () => {
  expect(formatValue(" value ")).toBe("value");
});
