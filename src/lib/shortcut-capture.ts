const supportedKeys = new Set([
  'Backquote',
  'Backslash',
  'BracketLeft',
  'BracketRight',
  'Pause',
  'Comma',
  'Equal',
  'Minus',
  'Period',
  'Quote',
  'Semicolon',
  'Slash',
  'Backspace',
  'CapsLock',
  'Enter',
  'Space',
  'Tab',
  'Delete',
  'End',
  'Home',
  'Insert',
  'PageDown',
  'PageUp',
  'PrintScreen',
  'ScrollLock',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'ArrowUp',
  'NumLock',
  'NumpadAdd',
  'NumpadDecimal',
  'NumpadDivide',
  'NumpadEnter',
  'NumpadEqual',
  'NumpadMultiply',
  'NumpadSubtract',
  'Escape',
  'AudioVolumeDown',
  'AudioVolumeUp',
  'AudioVolumeMute',
  'MediaPlay',
  'MediaPause',
  'MediaPlayPause',
  'MediaStop',
  'MediaTrackNext',
  'MediaTrackPrevious',
]);

export function shortcutFromKeyboard(
  event: Pick<KeyboardEvent, 'code' | 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey'>,
): string | null {
  const modifiers = [
    event.ctrlKey && 'Control',
    event.altKey && 'Alt',
    event.shiftKey && 'Shift',
    event.metaKey && 'Super',
  ].filter(Boolean);
  if (!modifiers.length) return null;
  const code = event.code;
  let key = code;
  if (/^Key[A-Z]$/.test(code)) key = code.slice(3);
  else if (/^Digit[0-9]$/.test(code)) key = code.slice(5);
  else if (
    !supportedKeys.has(code) &&
    !/^Numpad[0-9]$/.test(code) &&
    !/^F(?:[1-9]|1[0-9]|2[0-4])$/.test(code)
  )
    return null;
  return [...modifiers, key].join('+');
}
