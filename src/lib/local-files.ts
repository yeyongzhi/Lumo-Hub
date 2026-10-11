import { invoke } from '@tauri-apps/api/core'
export async function readLocal<T>(key:string):Promise<T|null>{
 const saved=await invoke<T|null>('read_local',{key})
 if(saved!==null){localStorage.removeItem(key);return saved}
 const legacy=localStorage.getItem(key)
 if(legacy===null)return null
 let value:unknown
 try{value=JSON.parse(legacy)}catch{value=legacy}
 await writeLocal(key,value);localStorage.removeItem(key)
 return value as T
}
export function writeLocal(key:string,value:unknown){return invoke<void>('write_local',{key,value})}

export async function migrateLegacyStorage(){for(const key of Object.keys(localStorage)){if(key.startsWith('lumo-'))await readLocal(key)}}
