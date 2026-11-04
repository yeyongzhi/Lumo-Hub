import type { ProductSettings } from './types';
export function normalizeNoteTags(product: ProductSettings): ProductSettings {
  const note_tags: Record<string, string[]> = {};
  const catalog = new Set(product.tag_catalog ?? []);
  for (const [id, tags] of Object.entries(product.note_tags)) {
    if (id.startsWith('__tag_catalog__')) tags.forEach((tag) => catalog.add(tag));
    else note_tags[id] = tags;
  }
  return { ...product, note_tags, tag_catalog: [...catalog] };
}
