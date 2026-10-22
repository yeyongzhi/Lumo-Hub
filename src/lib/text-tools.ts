export const textOperations = [
 {id:'dedupe',label:'逐行去重'}, {id:'trim',label:'去除首尾空白'}, {id:'blank',label:'去除空行'},
 {id:'upper',label:'转大写'}, {id:'lower',label:'转小写'}, {id:'sort',label:'逐行排序'},
 {id:'json',label:'JSON 格式化'}, {id:'compact',label:'JSON 压缩'},
 {id:'urlencode',label:'URL 编码'}, {id:'urldecode',label:'URL 解码'},
 {id:'base64encode',label:'Base64 编码'}, {id:'base64decode',label:'Base64 解码'},
] as const
export function transformText(id:string,text:string):string {
 switch(id){
 case 'dedupe':return [...new Set(text.split(/\r?\n/))].join('\n')
 case 'trim':return text.split(/\r?\n/).map(line=>line.trim()).join('\n')
 case 'blank':return text.split(/\r?\n/).filter(line=>line.trim()).join('\n')
 case 'upper':return text.toUpperCase()
 case 'lower':return text.toLowerCase()
 case 'sort':return text.split(/\r?\n/).sort((a,b)=>a.localeCompare(b)).join('\n')
 case 'json':return JSON.stringify(JSON.parse(text),null,2)
 case 'compact':return JSON.stringify(JSON.parse(text))
 case 'urlencode':return encodeURIComponent(text)
 case 'urldecode':return decodeURIComponent(text)
 case 'base64encode':return btoa(Array.from(new TextEncoder().encode(text),b=>String.fromCharCode(b)).join(''))
 case 'base64decode':return new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(text),c=>c.charCodeAt(0)))
 default:throw new Error('未知操作')
 }
}
