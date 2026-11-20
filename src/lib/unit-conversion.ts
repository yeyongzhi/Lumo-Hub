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
  area: {
    label: '面积',
    units: {
      m2: ['平方米 (m²)', 1],
      km2: ['平方千米 (km²)', 1_000_000],
      cm2: ['平方厘米 (cm²)', 0.0001],
      hectare: ['公顷 (ha)', 10_000],
      mu: ['亩', 2000 / 3],
      acre: ['英亩 (acre)', 4046.8564224],
      ft2: ['平方英尺 (ft²)', 0.09290304],
    },
  },
  volume: {
    label: '体积',
    units: {
      L: ['升 (L)', 1],
      mL: ['毫升 (mL)', 0.001],
      m3: ['立方米 (m³)', 1000],
      cm3: ['立方厘米 (cm³)', 0.001],
      gal: ['美制加仑 (gal)', 3.785411784],
      floz: ['美制液量盎司 (fl oz)', 0.0295735295625],
    },
  },
  time: {
    label: '时间',
    units: {
      s: ['秒 (s)', 1],
      ms: ['毫秒 (ms)', 0.001],
      min: ['分钟 (min)', 60],
      h: ['小时 (h)', 3600],
      day: ['天', 86400],
      week: ['周', 604800],
    },
  },
  speed: {
    label: '速度',
    units: {
      ms: ['米/秒 (m/s)', 1],
      kmh: ['千米/小时 (km/h)', 1 / 3.6],
      mph: ['英里/小时 (mph)', 0.44704],
      knot: ['节 (kn)', 1852 / 3600],
    },
  },
  pressure: {
    label: '压力',
    units: {
      Pa: ['帕斯卡 (Pa)', 1],
      kPa: ['千帕 (kPa)', 1000],
      bar: ['巴 (bar)', 100000],
      atm: ['标准大气压 (atm)', 101325],
      psi: ['磅/平方英寸 (psi)', 6894.757293168],
    },
  },
  energy: {
    label: '能量',
    units: {
      J: ['焦耳 (J)', 1],
      kJ: ['千焦 (kJ)', 1000],
      cal: ['卡 (cal)', 4.184],
      kcal: ['千卡 (kcal)', 4184],
      Wh: ['瓦时 (Wh)', 3600],
      kWh: ['千瓦时 (kWh)', 3600000],
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
