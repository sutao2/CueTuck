export function listAdminOperations(openapiText) {
  const operations = [];
  let path = null;
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
      continue;
    }
    const methodMatch = raw.match(/^    (get|post|put|delete):$/);
    if (methodMatch && path) {
      flush();
      pending = { method: methodMatch[1].toUpperCase(), path, auth: "admin" };
      continue;
    }
    if (pending && /^\s+security:\s*\[\s*\]\s*$/.test(raw)) {
      pending.auth = "none";
    }
  }
  flush();
  return operations;
}
