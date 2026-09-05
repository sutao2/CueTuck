// Tauri's command macro accepts camelCase argument names by default.
// Only command arguments are converted; nested API/SQLite records keep their schema.
export async function invokeCommand(command, args) {
  const { invoke } = await import("@tauri-apps/api/core");
  const payload = args && Object.fromEntries(Object.entries(args).map(([key, value]) => [
    key.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase()), value,
  ]));
  return invoke(command, payload);
}
