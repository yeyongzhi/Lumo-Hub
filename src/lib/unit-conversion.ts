export const unitCategories = {
  length: {
    label: '长度',
    units: {
      m: ['米 (m)', 1],
      cm: ['厘米 (cm)', 0.01],
      mm: ['毫米 (mm)', 0.001],
      km: ['千米 (km)', 1000],
      in: ['英寸 (in)', 0.0254],
      ft: ['英尺 (ft)', 0.3048],
      yd: ['码 (yd)', 0.9144],
      mi: ['英里 (mi)', 1609.344],
    },
  },
  mass: {
    label: '重量',
    units: {
      kg: ['千克 (kg)', 1],
      g: ['克 (g)', 0.001],
      mg: ['毫克 (mg)', 0.000001],
      t: ['吨 (t)', 1000],
      lb: ['磅 (lb)', 0.45359237],
      oz: ['盎司 (oz)', 0.028349523125],
    },
  },
  temperature: {
    label: '温度',
    units: {
      C: ['摄氏度 (°C)', 1, 0],
      F: ['华氏度 (°F)', 5 / 9, 32],
      K: ['开尔文 (K)', 1, 273.15],
    },
  },
  data: {
    label: '文件大小',
    units: {
      B: ['字节 (B)', 1],
      KB: ['千字节 (KB)', 1000],
      MB: ['兆字节 (MB)', 1000 ** 2],
      GB: ['吉字节 (GB)', 1000 ** 3],
      TB: ['太字节 (TB)', 1000 ** 4],
      KiB: ['KiB（二进制）', 1024],
      MiB: ['MiB（二进制）', 1024 ** 2],
      GiB: ['GiB（二进制）', 1024 ** 3],
      TiB: ['TiB（二进制）', 1024 ** 4],
    },
  },
} as const;

export type UnitCategory = keyof typeof unitCategories;
type Unit = readonly [label: string, factor: number, origin?: number];

export function unitsFor(category: UnitCategory): Record<string, Unit> {
  return unitCategories[category].units;
}

export function convertUnit(value: number, category: UnitCategory, from: string, to: string) {
  const units = unitsFor(category);
  const [, fromFactor, fromOrigin = 0] = units[from];
  const [, toFactor, toOrigin = 0] = units[to];
  if (from === to) return value;
  return (value - fromOrigin) * (fromFactor / toFactor) + toOrigin;
}
