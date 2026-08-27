import { getLocalSetting, setLocalSetting } from "./library.js";
import { getSession } from "./session.js";
import { createPublication, deleteFavorite, putFavorite } from "./square.js";

const QUEUE_KEY = "sync_queue";

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
  return readQueue();
}

function requireEmail() {
  const email = getSession().email;
  if (!email) throw new Error("同步需要登录");
  return email;
}

async function enqueue(job) {
  const email = requireEmail();
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
  try {
    if (method === "DELETE") await deleteFavorite(id);
    else await putFavorite(id);
    await flushSyncQueue();
    return { queued: false };
  } catch (error) {
    if ((await getLocalSetting("auto_sync_queue")) !== "1") throw error;
    await enqueue({ kind: "favorite", method, id });
    return { queued: true };
  }
}

export async function publishWithQueue(payload) {
  try {
    const result = await createPublication(payload);
    await flushSyncQueue();
    return { queued: false, result };
  } catch (error) {
    if ((await getLocalSetting("auto_sync_queue")) !== "1") throw error;
    await enqueue({
      kind: "publish",
      sourceId: payload.sourceId,
      title: payload.title,
      content: payload.content,
    });
    return { queued: true };
  }
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
    });
  }
}

export async function flushSyncQueue() {
  const email = getSession().email;
  if (!email) return [];
  const jobs = await readQueue();
  const remaining = [];
  let failed = false;
  for (const job of jobs) {
    if (failed || job.email !== email) {
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
