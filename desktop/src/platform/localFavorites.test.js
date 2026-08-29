import { beforeEach, describe, expect, it } from "vitest";
import { resetMemoryLibrary, setLocalSetting } from "./library.js";
import { filterLocalItems, listLocalFavoriteIds, parseFavoriteIds, toggleLocalFavorite } from "./localFavorites.js";

describe("localFavorites", () => {
  beforeEach(() => {
    resetMemoryLibrary();
  });

  it("parses a stored id list and ignores garbage", () => {
    expect(parseFavoriteIds('["a","b"]')).toEqual(["a", "b"]);
    expect(parseFavoriteIds("not-json")).toEqual([]);
  });

  it("toggles a local favorite id in settings", async () => {
    await setLocalSetting("local_favorite_ids", "[]");
    expect(await toggleLocalFavorite("p-1")).toEqual(["p-1"]);
    expect(await listLocalFavoriteIds()).toEqual(["p-1"]);
    expect(await toggleLocalFavorite("p-1")).toEqual([]);
  });

  it("filters recent and favorite library rows", () => {
    const items = [
      { id: "old", kind: "prompt", last_used_at: "1", title: "旧" },
      { id: "new", kind: "prompt", last_used_at: "9", title: "新" },
      { id: "unused", kind: "prompt", title: "未用" },
      { id: "star", kind: "prompt", title: "星" },
    ];
    expect(filterLocalItems(items, { tab: "最近" }).map((row) => row.id)).toEqual(["new", "old"]);
    expect(filterLocalItems(items, { tab: "收藏", favoriteIds: ["star"] }).map((row) => row.id)).toEqual(["star"]);
  });
});
