import { beforeEach, expect, it } from "vitest";
import { downloadSquareItem, resetSquare, setSquareContentTransport } from "./square.js";
import { resetMemoryLibrary, listLocalPrompts, listLocalCollections } from "./memoryLibrary.js";

beforeEach(() => { resetSquare(); resetMemoryLibrary(); });

it("preserves downloaded category and model in the web library", async () => {
  setSquareContentTransport(async () => ({ id: "image", title: "人像", content: "光影", category_id: "cat-image-0", model: "Flux" }));
  await downloadSquareItem("image");
  expect(listLocalPrompts()[0]).toMatchObject({ category_id: "cat-image-0", model: "Flux", source: "downloaded" });
});

it("downloads a collection with membership and rejects partial snapshots", async () => {
  const payload = { id: "c", kind: "collection", title: "合集", members: [{ title: "成员", content: "正文", model: "Flux" }] };
  setSquareContentTransport(async () => payload);
  await downloadSquareItem("c");
  expect(listLocalCollections()).toHaveLength(1);
  expect(listLocalPrompts()[0]).toMatchObject({ collection_id: listLocalCollections()[0].id, source: "downloaded", model: "Flux" });
  payload.members.push({ title: "错误", content: "" });
  await expect(downloadSquareItem("c")).rejects.toThrow();
  expect(listLocalCollections()).toHaveLength(1);
  expect(listLocalPrompts()).toHaveLength(1);
});
