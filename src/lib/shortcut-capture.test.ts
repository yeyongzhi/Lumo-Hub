import { describe, expect, it } from 'vitest';
import { shortcutFromKeyboard } from './shortcut-capture';

const event = (code: string, modifiers: Partial<KeyboardEvent> = {}) => ({
  code,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  ...modifiers,
});

describe('shortcutFromKeyboard', () => {
  it('formats modifier combinations in the order accepted by the shortcut parser', () => {
    expect(shortcutFromKeyboard(event('Space', { altKey: true }))).toBe('Alt+Space');
    expect(shortcutFromKeyboard(event('KeyK', { ctrlKey: true, shiftKey: true }))).toBe(
      'Control+Shift+K',
    );
    expect(shortcutFromKeyboard(event('Digit1', { ctrlKey: true, altKey: true }))).toBe(
      'Control+Alt+1',
    );
  });

  it('supports navigation and keypad codes while rejecting unsupported or incomplete input', () => {
    expect(shortcutFromKeyboard(event('ArrowUp', { ctrlKey: true }))).toBe('Control+ArrowUp');
    expect(shortcutFromKeyboard(event('NumpadAdd', { altKey: true }))).toBe('Alt+NumpadAdd');
    expect(shortcutFromKeyboard(event('F24', { altKey: true }))).toBe('Alt+F24');
    expect(shortcutFromKeyboard(event('KeyA'))).toBeNull();
    expect(shortcutFromKeyboard(event('ControlLeft', { ctrlKey: true }))).toBeNull();
    expect(shortcutFromKeyboard(event('F25', { altKey: true }))).toBeNull();
  });
});
