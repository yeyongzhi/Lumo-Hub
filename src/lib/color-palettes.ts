export interface PaletteColor { name: string; hex: string }
export interface ColorPalette { id: string; name: string; description: string; colors: PaletteColor[] }

// Curated display palettes; Morandi and traditional colors are design references,
// rather than a claim of a single standardized historical color specification.
export const colorPalettes: ColorPalette[] = [
  { id: 'classic', name: '经典颜色', description: '常用基础色与明快的经典配色。', colors: [
    {name:'纯白',hex:'#FFFFFF'},{name:'银灰',hex:'#C0C0C0'},{name:'灰色',hex:'#808080'},{name:'纯黑',hex:'#000000'},
    {name:'红色',hex:'#FF0000'},{name:'橙色',hex:'#FF8000'},{name:'金色',hex:'#FFD700'},{name:'黄色',hex:'#FFFF00'},
    {name:'草绿',hex:'#80C000'},{name:'绿色',hex:'#008000'},{name:'青色',hex:'#00FFFF'},{name:'湖蓝',hex:'#00A8CC'},
    {name:'蓝色',hex:'#0000FF'},{name:'海军蓝',hex:'#000080'},{name:'紫色',hex:'#800080'},{name:'品红',hex:'#FF00FF'},
    {name:'珊瑚',hex:'#FF7F50'},{name:'粉红',hex:'#FFC0CB'},{name:'棕色',hex:'#A52A2A'},{name:'米色',hex:'#F5F5DC'},
  ]},
  { id: 'morandi', name: '莫兰迪色', description: '低饱和、柔和的灰调配色，适合安静舒缓的画面。', colors: [
    {name:'雾玫瑰',hex:'#B59898'},{name:'灰粉',hex:'#D5B6B0'},{name:'陶土',hex:'#B58A78'},{name:'暖杏',hex:'#D6B9A3'},
    {name:'燕麦',hex:'#D8CEBB'},{name:'奶油',hex:'#E5DFD2'},{name:'鼠尾草',hex:'#A3AD9B'},{name:'灰橄榄',hex:'#8C927D'},
    {name:'苔绿',hex:'#747C6A'},{name:'雾青',hex:'#9DAEAA'},{name:'灰蓝',hex:'#96A5AF'},{name:'烟蓝',hex:'#738A9C'},
    {name:'雾紫',hex:'#B0A4B8'},{name:'灰丁香',hex:'#C4BAC9'},{name:'藕灰',hex:'#AA9AAB'},{name:'暖灰',hex:'#B8B2A9'},
    {name:'石灰',hex:'#92938D'},{name:'可可',hex:'#8D7B70'},{name:'炭灰',hex:'#62635F'},{name:'纸白',hex:'#ECE8DF'},
  ]},
  { id: 'traditional', name: '中国传统色', description: '精选传统色意象；不同染料、材质与色谱中的色值会有差异。', colors: [
    {name:'朱砂',hex:'#FF461F'},{name:'胭脂',hex:'#9D2933'},{name:'绯红',hex:'#C83C23'},{name:'海棠红',hex:'#DB5A6B'},
    {name:'桃红',hex:'#F47983'},{name:'鹅黄',hex:'#FFF143'},{name:'栀子',hex:'#FFB61E'},{name:'秋香',hex:'#D9B611'},
    {name:'竹青',hex:'#789262'},{name:'松花',hex:'#BCE672'},{name:'碧绿',hex:'#2ADD9C'},{name:'石绿',hex:'#16A951'},
    {name:'天青',hex:'#7FECAD'},{name:'缥色',hex:'#7FECED'},{name:'靛青',hex:'#177CB0'},{name:'黛蓝',hex:'#425066'},
    {name:'紫棠',hex:'#56004F'},{name:'藕荷',hex:'#E4C6D0'},{name:'月白',hex:'#D6ECF0'},{name:'玄色',hex:'#622A1D'},
  ]},
]
