import { getLocalSetting, setLocalSetting } from "./library.js";
import { getSession } from "./session.js";
import { createPublication, deleteFavorite, putFavorite } from "./square.js";

const QUEUE_KEY = "sync_queue";
let operations = Promise.resolve();

function exclusively(action) {
  const result = operations.then(action, action);
  operations = result.catch(() => {});
  return result;
}

async function readQueue() {
  const raw = await getLocalSetting(QUEUE_KEY);
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

async function writeQueue(jobs) {
  await setLocalSetting(QUEUE_KEY, JSON.stringify(jobs));
}

export async function listSyncQueue() {
  await operations;
  return readQueue();
}

function requireEmail() {
  const email = getSession().email;
  if (!email) throw new Error("同步需要登录");
  return email;
}

async function enqueue(job, email) {
  const next = { ...job, email };
  const current = await readQueue();
  const filtered = current.filter((item) => {
    if (item.email !== email) return true;
    if (next.kind === "favorite" && item.kind === "favorite" && item.id === next.id) return false;
    if (next.kind === "publish" && item.kind === "publish" && item.sourceId === next.sourceId) return false;
    return true;
  });
  filtered.push(next);
  await writeQueue(filtered);
}

export async function favoriteWithQueue(id, method) {
  const email = requireEmail();
  return exclusively(async () => {
    if (getSession().email !== email) throw new Error("账号已改变，请重试");
    try {
      if (method === "DELETE") await deleteFavorite(id);
      else await putFavorite(id);
    } catch (error) {
      if ((await getLocalSetting("auto_sync_queue")) !== "1") throw error;
      await enqueue({ kind: "favorite", method, id }, email);
      return { queued: true };
    }
    await removeSuperseded({ kind: "favorite", id }, email);
    await flushQueueFor(email);
    return { queued: false };
  });
}

export async function publishWithQueue(payload) {
  const email = requireEmail();
  return exclusively(async () => {
    if (getSession().email !== email) throw new Error("账号已改变，请重试");
    let result;
    try {
      result = await createPublication(payload);
    } catch (error) {
      if ((await getLocalSetting("auto_sync_queue")) !== "1") throw error;
      await enqueue({
        kind: "publish",
        sourceId: payload.sourceId,
        title: payload.title,
        content: payload.content,
        categoryId: payload.categoryId,
        model: payload.model,
        sourceKind: payload.kind,
        members: payload.members,
        assetRefs: payload.assetRefs,
      }, email);
      return { queued: true };
    }
    await removeSuperseded({ kind: "publish", sourceId: payload.sourceId }, email);
    await flushQueueFor(email);
    return { queued: false, result };
  });
}

async function removeSuperseded(job, email) {
  const jobs = await readQueue();
  await writeQueue(jobs.filter((item) => item.email !== email || item.kind !== job.kind
    || (job.kind === "favorite" ? item.id !== job.id : item.sourceId !== job.sourceId)));
}

export async function applyQueuedFavorites(ids) {
  const email = getSession().email;
  let next = [...ids];
  for (const job of await readQueue()) {
    if (job.kind !== "favorite" || job.email !== email) continue;
    if (job.method === "PUT") {
      if (!next.includes(job.id)) next.push(job.id);
    } else if (job.method === "DELETE") {
      next = next.filter((id) => id !== job.id);
    }
  }
  return next;
}

async function runJob(job) {
  if (job.kind === "favorite") {
    if (job.method === "DELETE") await deleteFavorite(job.id);
    else await putFavorite(job.id);
    return;
  }
  if (job.kind === "publish") {
    await createPublication({
      sourceId: job.sourceId,
      title: job.title,
      content: job.content,
      categoryId: job.categoryId,
      model: job.model,
      kind: job.sourceKind,
      members: job.members,
      assetRefs: job.assetRefs,
    });
    return;
  }
  throw new Error("未知队列任务");
}

export async function flushSyncQueue() {
  const email = getSession().email;
  if (!email) return [];
  return exclusively(() => flushQueueFor(email));
}

async function flushQueueFor(email) {
  const jobs = await readQueue();
  const remaining = [];
  let failed = false;
  for (const job of jobs) {
    if (failed || job.email !== email || getSession().email !== email) {
      remaining.push(job);
      continue;
    }
    try {
      await runJob(job);
    } catch {
      failed = true;
      remaining.push(job);
    }
  }
  await writeQueue(remaining);
  return remaining;
}
