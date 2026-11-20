import { describe, expect, it } from 'vitest';
import { convertUnit } from './unit-conversion';

describe('unit conversion', () => {
  it('converts common units across the added categories', () => {
    expect(convertUnit(1, 'area', 'mu', 'm2')).toBeCloseTo(2000 / 3);
    expect(convertUnit(1, 'volume', 'gal', 'L')).toBeCloseTo(3.785411784);
    expect(convertUnit(1, 'time', 'day', 'h')).toBe(24);
    expect(convertUnit(36, 'speed', 'kmh', 'ms')).toBeCloseTo(10);
    expect(convertUnit(1, 'pressure', 'atm', 'Pa')).toBe(101325);
    expect(convertUnit(1, 'energy', 'kWh', 'kJ')).toBe(3600);
  });
});
