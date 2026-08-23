export function handleLauncherSearchKey(
  event,
  {
    move = () => {},
    moveTo = () => {},
    rowCount = 0,
    current = () => null,
    activate = () => {},
    close = () => {},
  } = {},
) {
  if (event.isComposing) return false;
  const moves = { ArrowDown: 1, ArrowUp: -1, PageDown: 5, PageUp: -5 };
  if (event.key in moves) move(moves[event.key]);
  else if (event.key === "Home") moveTo(0);
  else if (event.key === "End") moveTo(rowCount - 1);
  else if (event.key === "Enter") {
    const row = current();
    if (row) activate(row, event.metaKey || event.ctrlKey ? "copy" : "default");
  } else if (event.key === "Escape") close();
  else return false;
  event.preventDefault();
  event.stopPropagation();
  return true;
}
