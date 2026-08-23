export function listSquareOperations(openapiText) {
  const operations = [];
  let path = null;
  let method = null;
  let pending = null;
  const flush = () => {
    if (pending) operations.push(pending);
    pending = null;
  };
  for (const raw of openapiText.split("\n")) {
    const pathMatch = raw.match(/^  (\/[^\s:]+):$/);
    if (pathMatch) {
      flush();
      path = pathMatch[1];
      method = null;
      continue;
    }
    const methodMatch = raw.match(/^    (get|post|put|delete):$/);
    if (methodMatch && path) {
      flush();
      method = methodMatch[1].toUpperCase();
      pending = { method, path, auth: "user" };
      continue;
    }
    if (pending && /^\s+security:\s*\[\s*\]\s*$/.test(raw)) {
      pending.auth = "none";
    }
  }
  flush();
  return operations;
}
