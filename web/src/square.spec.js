import { beforeEach, expect, it } from "vitest";
import { downloadSquareItem, resetSquare, setSquareContentTransport } from "./square.js";
import { resetMemoryLibrary, listLocalPrompts } from "./memoryLibrary.js";

beforeEach(() => { resetSquare(); resetMemoryLibrary(); });

it("preserves downloaded category and model in the web library", async () => {
  setSquareContentTransport(async () => ({ id: "image", title: "人像", content: "光影", category_id: "cat-image-0", model: "Flux" }));
  await downloadSquareItem("image");
  expect(listLocalPrompts()[0]).toMatchObject({ category_id: "cat-image-0", model: "Flux", source: "downloaded" });
});
