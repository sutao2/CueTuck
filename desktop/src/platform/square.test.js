import { beforeEach, describe, expect, it, vi } from "vitest";
import { createLocalPrompt, deleteLocalPrompt, listLocalCollections, listCollectionMembers, listLocalPrompts, resetMemoryLibrary, setLocalSetting } from "./library.js";
import {
  createPublication,
  downloadSquareItem,
  listMyPublications,
  listSquareItems,
  putFavorite,
  resetSquare,
  setDownloadStatsTransport,
  setFavoriteTransport,
  setMineTransport,
  setPublishTransport,
  setSquareContentTransport,
  setSquareTransport,
} from "./square.js";
import { loginSession, resetMemorySession, setSessionTransport } from "./session.js";

describe("square client", () => {
  beforeEach(() => {
    resetSquare();
    resetMemoryLibrary();
    resetMemorySession();
  });

  it("returns injected items when online", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    const rows = await listSquareItems({ sort: "推荐" });
    expect(rows).toHaveLength(1);
    expect(rows[0].title).toBe("自然光群像");
  });

  it('does not append corpus attribution metadata to downloaded prompt text', async () => {
    const content = 'A mountain valley in golden-hour light, watercolor illustration.';
    setSquareContentTransport(async () => ({ id: 'corpus-clean', title: 'Valley', content, reference: {
      repository: 'poloclub/diffusiondb', author: 'Contributors', license: 'CC0 1.0',
      url: 'https://huggingface.co/datasets/poloclub/diffusiondb', images: [],
    } }));
    await downloadSquareItem('corpus-clean');
    const [row] = await listLocalPrompts();
    expect(row.content).toBe(content);
    expect(row.remote_id).toBe('corpus-clean');
  });

  it("deduplicates concurrent downloads and permits another after deletion", async () => {
    let reads = 0, stats = 0;
    setSquareContentTransport(async id => { reads++; return {id,title:'同一条',content:'正文'}; });
    await setLocalSetting('anonymous_download_stats','1');
    setDownloadStatsTransport(async () => { stats++; });
    const [first, second] = await Promise.all([downloadSquareItem('same'),downloadSquareItem('same')]);
    expect(first.id).toBe(second.id);
    await downloadSquareItem('same');
    expect(await listLocalPrompts()).toHaveLength(1);
    expect(reads).toBe(1); expect(stats).toBe(1);
    await deleteLocalPrompt(first.id);
    await downloadSquareItem('same');
    expect(await listLocalPrompts()).toHaveLength(1);
    expect(reads).toBe(2);
  });

  it("does not delay local download success for stalled anonymous statistics", async () => {
    setSquareContentTransport(async id => ({id,title:'完成',content:'正文'}));
    await setLocalSetting('anonymous_download_stats','1');
    setDownloadStatsTransport(() => new Promise(() => {}));
    expect((await downloadSquareItem('done')).remote_id).toBe('done');
  });

  it("counts by default only after server confirmation without delaying the download", async () => {
    setSquareContentTransport(async id => ({ id, title: '计数', content: '正文' }));
    let confirm;
    setDownloadStatsTransport(() => new Promise(resolve => { confirm = resolve; }));
    const updated = vi.fn();
    const row = await downloadSquareItem('counted', updated);
    expect(row.remote_id).toBe('counted');
    expect(updated).not.toHaveBeenCalled();
    confirm({ download_count: 27 });
    await vi.waitFor(() => expect(updated).toHaveBeenCalledWith(27));
    await downloadSquareItem('counted', updated);
    expect(updated).toHaveBeenCalledTimes(1);
  });

  it.each([null, {}, { download_count: -1 }, { download_count: '2' }])("does not invent a count for invalid or old server responses %j", async result => {
    setSquareContentTransport(async id => ({ id, title: '计数', content: '正文' }));
    setDownloadStatsTransport(async () => result);
    const updated = vi.fn();
    await downloadSquareItem('invalid-count', updated);
    expect(updated).not.toHaveBeenCalled();
  });

  it("does not publish a count when the browser statistics request fails", async () => {
    setSquareContentTransport(async id => ({ id, title: '计数', content: '正文' }));
    const fetcher = vi.fn(async () => new Response('{"download_count":99}', { status: 500 }));
    vi.stubGlobal('fetch', fetcher);
    const updated = vi.fn();
    await downloadSquareItem('failed-stat', updated);
    expect(fetcher).toHaveBeenCalledWith(expect.stringContaining('/failed-stat/downloads'), expect.objectContaining({ method: 'POST', credentials: 'omit' }));
    expect(updated).not.toHaveBeenCalled();
    expect(await listLocalPrompts()).toHaveLength(1);
  });

  it("downloads collection members atomically without duplicate copies", async () => {
    const payload = { id: "remote", kind: "collection", title: "合集", category_id: "cat-image", members: [
      { title: "人像", content: "光影", category_id: "cat-image-0", model: "Flux" },
      { title: "代码", content: "测试", category_id: "cat-software-0", model: "GPT" },
    ] };
    setSquareContentTransport(async () => payload);
    await downloadSquareItem("remote");
    await downloadSquareItem("remote");
    const collections = await listLocalCollections();
    expect(collections).toHaveLength(1);
    for (const collection of collections) {
      expect(collection.category_id).toBe("cat-image");
      const members = await listCollectionMembers(collection.id);
      expect(members).toHaveLength(2);
      expect(members.find((row) => row.title === "人像")).toMatchObject({ content: "光影", model: "Flux", category_id: "cat-image-0", source: "downloaded", remote_id: "remote" });
    }
    payload.members[1].category_id = "missing";
    payload.id = "remote-2";
    await downloadSquareItem("remote-2");
    expect(await listLocalCollections()).toHaveLength(2);
    expect(await listLocalPrompts()).toHaveLength(4);
    expect((await listLocalPrompts()).filter(item=>item.category_id===null)).toHaveLength(1);
    payload.members = [];
    await expect(downloadSquareItem("remote-3")).rejects.toThrow("缺少成员快照");
  });

  it("retains category and model when downloading", async () => {
    setSquareContentTransport(async () => ({ id: "sq-image", title: "人像", content: "光影", category_id: "cat-image-0", model: "Flux" }));
    await downloadSquareItem("sq-image");
    const rows = await listLocalPrompts({ categoryId: "cat-image" });
    expect(rows).toHaveLength(1);
    expect(rows[0].model).toBe("Flux");
    expect(rows[0].category_id).toBe("cat-image-0");
  });

  it("forwards the selected model to the square transport", async () => {
    let seen;
    setSquareTransport(async (request) => {
      seen = request;
      return [];
    });
    await listSquareItems({ sort: "最新", query: "光", model: "Flux" });
    expect(seen).toEqual({ sort: "最新", query: "光", model: "Flux" });
  });

  it("surfaces offline as a thrown error", async () => {
    setSquareTransport(async () => {
      throw new Error("广场暂时不可用");
    });
    await expect(listSquareItems()).rejects.toThrow("广场暂时不可用");
  });

  it("writes a local copy with source=downloaded", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    const row = await downloadSquareItem("sq-1");
    expect(row.source).toBe("downloaded");
    expect(row.title).toBe("自然光群像");
    const listed = await listLocalPrompts({ query: "自然光群像" });
    expect(listed).toHaveLength(1);
    expect(listed[0].source).toBe("downloaded");
  });

  it("copies author onto the local row only when keep_author_on_download is on", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
      author: "林晚",
    }));
    await setLocalSetting("keep_author_on_download", "1");
    const kept = await downloadSquareItem("sq-1");
    expect(kept.author).toBe("林晚");
    expect(kept.content).toBe("清透蓝天下的多元人物群像。");

    await setLocalSetting("keep_author_on_download", "0");
    setSquareContentTransport(async (id) => ({
      id,
      title: "夜景街拍",
      content: "潮湿路面的霓虹倒影。",
      author: "林晚",
    }));
    const skipped = await downloadSquareItem("sq-2");
    expect(skipped.author).toBeFalsy();
    expect(skipped.content).toBe("潮湿路面的霓虹倒影。");
  });

  it("posts anonymous download stats without auth after a successful download when the setting is on", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    const calls = [];
    setDownloadStatsTransport(async (request) => {
      calls.push(request);
    });
    await setLocalSetting("anonymous_download_stats", "1");
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const row = await downloadSquareItem("sq-1");
    expect(row.source).toBe("downloaded");
    expect(calls).toHaveLength(1);
    expect(calls[0].id).toBe("sq-1");
    expect(calls[0].method).toBe("POST");
    expect(calls[0].path).toBe("/v1/square/items/sq-1/downloads");
    expect(calls[0].headers).toEqual({});
    expect(JSON.stringify(calls[0])).not.toMatch(/dev@promptark|自然光群像|清透蓝天|Authorization|authorization/);
  });

  it("does not post download stats when the setting is off", async () => {
    await setLocalSetting("anonymous_download_stats", "0");
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    const calls = [];
    setDownloadStatsTransport(async (request) => {
      calls.push(request);
    });
    const row = await downloadSquareItem("sq-1");
    expect(row.source).toBe("downloaded");
    expect(calls).toEqual([]);
  });

  it("keeps the local download when stats post fails", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    setDownloadStatsTransport(async () => {
      throw new Error("stats down");
    });
    await setLocalSetting("anonymous_download_stats", "1");
    const row = await downloadSquareItem("sq-1");
    expect(row.source).toBe("downloaded");
    const listed = await listLocalPrompts({ query: "自然光群像" });
    expect(listed).toHaveLength(1);
  });

  it("submits a publication without changing the local copy", async () => {
    const created = await createLocalPrompt({ title: "本地源", content: "旧正文" });
    const calls = [];
    setPublishTransport(async (payload) => {
      calls.push(payload);
      return { id: "pub-1", source_id: payload.sourceId, status: "pending" };
    });
    const result = await createPublication({
      sourceId: created.id,
      title: created.title,
      content: created.content,
    });
    expect(result.status).toBe("pending");
    expect(calls[0]).toEqual({ sourceId: created.id, title: "本地源", content: "旧正文" });
    const listed = await listLocalPrompts({ query: "本地源" });
    expect(listed[0].content).toBe("旧正文");
  });

  it("lists my publications from the injected transport", async () => {
    setMineTransport(async () => [
      { id: "pub-1", source_id: "mem-1", status: "pending", title: "新稿" },
    ]);
    const rows = await listMyPublications();
    expect(rows).toEqual([
      { id: "pub-1", source_id: "mem-1", status: "pending", title: "新稿" },
    ]);
  });

  it("puts a favorite without writing a local copy", async () => {
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const calls = [];
    setFavoriteTransport(async (request) => {
      calls.push(request);
      return { id: request.id };
    });
    await putFavorite("sq-1");
    expect(calls).toEqual([{ method: "PUT", id: "sq-1" }]);
    expect(await listLocalPrompts({ query: "" })).toHaveLength(0);
  });
});
