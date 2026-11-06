export function formatJson(source: string, compact: boolean) {
  JSON.parse(source);
  const tokens = source.match(/"(?:\\[\s\S]|[^"\\])*"|[{}[\],:]|[^\s{}[\],:"]+/g) ?? [];
  if (compact) return tokens.join('');

  let depth = 0;
  const line = () => '\n' + '  '.repeat(depth);
  return tokens
    .map((token, index) => {
      const next = tokens[index + 1];
      const previous = tokens[index - 1];
      switch (token) {
        case '{':
        case '[':
          if (next === '}' || next === ']') return token;
          depth++;
          return token + line();
        case '}':
        case ']':
          if (previous === '{' || previous === '[') return token;
          depth--;
          return line() + token;
        case ',':
          return token + line();
        case ':':
          return ': ';
        default:
          return token;
      }
    })
    .join('');
}

export function jsonError(source: string, error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  const position = /(?:at position|at character) (\d+)/i.exec(message);
  let offset = position
    ? Number(position[1])
    : /unexpected end/i.test(message)
      ? source.length
      : null;
  if (offset === null) {
    const token = /Unexpected token '(.)'/i.exec(message)?.[1];
    if (token) {
      const first = source.indexOf(token);
      if (first >= 0 && first === source.lastIndexOf(token)) offset = first;
    } else if (/^"(?:undefined|NaN|Infinity)" is not valid JSON$/i.test(message)) {
      offset = source.search(/\S/);
    }
  }
  if (offset === null) return { message: 'JSON 语法错误。' + message, offset };
  const before = source.slice(0, offset).split(/\r\n|\r|\n/);
  return {
    message:
      '第 ' +
      before.length +
      ' 行，第 ' +
      (before[before.length - 1].length + 1) +
      ' 列 · ' +
      message,
    offset,
  };
}
