type Line = { number: number; text: string };
export type DiffRow = {
  kind: 'same' | 'added' | 'removed' | 'changed';
  before?: Line;
  after?: Line;
};

function lines(text: string) {
  return text === ''
    ? []
    : text.split(/\r\n|\r|\n/).map((text, index) => ({ number: index + 1, text }));
}

export function compareLines(before: string, after: string): DiffRow[] {
  const left = lines(before);
  const right = lines(after);
  if (left.length > 2000 || right.length > 2000) {
    throw new Error('每侧最多可对比 2,000 行。');
  }

  // The longest common subsequence anchors unchanged lines across insertions and deletions.
  const width = right.length + 1;
  const matches = new Uint16Array((left.length + 1) * width);
  for (let i = left.length - 1; i >= 0; i--) {
    for (let j = right.length - 1; j >= 0; j--) {
      const cell = i * width + j;
      matches[cell] =
        left[i].text === right[j].text
          ? 1 + matches[cell + width + 1]
          : Math.max(matches[cell + width], matches[cell + 1]);
    }
  }

  const rows: DiffRow[] = [];
  const removed: Line[] = [];
  const added: Line[] = [];
  function flushChanges() {
    for (let index = 0; index < Math.max(removed.length, added.length); index++) {
      rows.push({
        kind:
          index < removed.length && index < added.length
            ? 'changed'
            : index < removed.length
              ? 'removed'
              : 'added',
        before: removed[index],
        after: added[index],
      });
    }
    removed.length = 0;
    added.length = 0;
  }

  let i = 0;
  let j = 0;
  while (i < left.length || j < right.length) {
    if (i < left.length && j < right.length && left[i].text === right[j].text) {
      flushChanges();
      rows.push({ kind: 'same', before: left[i++], after: right[j++] });
    } else if (
      i < left.length &&
      (j === right.length || matches[(i + 1) * width + j] >= matches[i * width + j + 1])
    ) {
      removed.push(left[i++]);
    } else {
      added.push(right[j++]);
    }
  }
  flushChanges();
  return rows;
}
