export function rankSearchResults<T extends { title: string }>(items: T[], query: string): T[] {
  const term = query.trim().toLocaleLowerCase();
  const score = (title: string) => {
    const text = title.toLocaleLowerCase();
    return text === term ? 0 : text.startsWith(term) ? 1 : 2;
  };
  return [...items].sort((a, b) => score(a.title) - score(b.title));
}
