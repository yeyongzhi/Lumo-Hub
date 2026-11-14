import { describe, expect, it } from 'vitest';
import {
  libraryColumns,
  libraryDefaultFilter,
  libraryIconSize,
  libraryInitialFilter,
} from './library-preferences';

const groups = [{ id: 'work' }, { id: 'personal' }];

describe('application library preferences', () => {
  it('uses existing defaults when no preferences have been saved', () => {
    expect(libraryInitialFilter(undefined, groups)).toBe('all');
    expect(libraryIconSize(undefined)).toBe('standard');
    expect(libraryColumns(undefined)).toEqual(['group']);
  });

  it('opens a chosen group or restores the last selected range', () => {
    expect(libraryInitialFilter({ 'lumo-library-default-filter': 'favorites' }, groups)).toBe(
      'favorites',
    );
    expect(libraryInitialFilter({ 'lumo-library-default-filter': 'group:work' }, groups)).toBe(
      'group:work',
    );
    expect(
      libraryInitialFilter(
        { 'lumo-library-default-filter': 'remember', 'lumo-library-last-filter': 'group:personal' },
        groups,
      ),
    ).toBe('group:personal');
    expect(
      libraryInitialFilter(
        { 'lumo-library-default-filter': 'remember', 'lumo-library-last-filter': 'web' },
        groups,
      ),
    ).toBe('web');
  });

  it('falls back safely when a saved group or value no longer exists', () => {
    expect(libraryDefaultFilter({ 'lumo-library-default-filter': 'group:removed' }, groups)).toBe(
      'all',
    );
    expect(
      libraryInitialFilter(
        { 'lumo-library-default-filter': 'remember', 'lumo-library-last-filter': 'group:removed' },
        groups,
      ),
    ).toBe('all');
    expect(libraryIconSize({ 'lumo-library-icon-size': 'oversized' })).toBe('standard');
  });

  it('preserves an intentionally empty column selection and ignores invalid columns', () => {
    expect(libraryColumns({ 'lumo-library-columns': [] })).toEqual([]);
    expect(
      libraryColumns({ 'lumo-library-columns': ['path', 'recent', 'path', 'unknown'] }),
    ).toEqual(['path', 'recent']);
    expect(libraryIconSize({ 'lumo-library-icon-size': 'large' })).toBe('large');
  });
});
