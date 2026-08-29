const HINTS = {
  城市: "例如：京都",
  天数: "例如：3",
  姓名: "例如：林晚",
  city: "e.g. Kyoto",
  name: "e.g. Alex",
};

export function hintForVariable(name) {
  const key = String(name ?? "").trim();
  return HINTS[key] ?? HINTS[key.toLowerCase()] ?? "";
}
