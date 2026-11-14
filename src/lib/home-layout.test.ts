import { describe, expect, it } from 'vitest';
import {
  homeGridCellCount,
  homeGridProfileForWidth,
  homeGridProfiles,
  homeGridTrack,
  homeGridTracksPerCell,
  homeWidgetDimensions,
  widgetSizes,
} from './home-layout';

describe('首页卡片响应式尺寸', () => {
  it('selects the correct profile at each window width boundary', () => {
    for (const profile of homeGridProfiles) {
      expect(homeGridProfileForWidth(profile.minWindowWidth)).toEqual(profile);
      if (profile.minWindowWidth > 0)
        expect(homeGridProfileForWidth(profile.minWindowWidth - 1).cell).toBeLessThan(profile.cell);
    }
  });
  it('reduces visible grid columns in narrow spaces and preserves the minimum for two-column cards', () => {
    const profile = homeGridProfiles[3];
    expect(homeGridCellCount(848, profile)).toBe(4);
    expect(homeGridCellCount(632, profile)).toBe(3);
    expect(homeGridCellCount(416, profile)).toBe(2);
    expect(homeGridCellCount(100, profile, 2)).toBe(2);
  });
  it('keeps each base cell square and combines sizes with the profile gap', () => {
    for (const profile of homeGridProfiles) {
      expect(
        homeGridTrack(profile) * homeGridTracksPerCell + profile.gap * (homeGridTracksPerCell - 1),
      ).toBeCloseTo(profile.cell);
      for (const size of widgetSizes) {
        const [columns, rows] = size.split('x').map(Number);
        expect(homeWidgetDimensions(size, profile)).toEqual({
          width: columns! * profile.cell + (columns! - 1) * profile.gap,
          height: rows! * profile.cell + (rows! - 1) * profile.gap,
        });
      }
    }
    expect(homeWidgetDimensions('2x2', homeGridProfiles[0])).toEqual({ width: 208, height: 208 });
    expect(homeWidgetDimensions('2x2', homeGridProfiles[3])).toEqual({ width: 416, height: 416 });
  });
});
