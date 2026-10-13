// Run with: node scripts/product-ui-check.mjs <path-to-playwright> [artifact-directory]
// All desktop commands are replaced with fixtures. No real user data is read or changed.
import { createRequire } from 'node:module'
import assert from 'node:assert/strict'
const require=createRequire(import.meta.url)
const {chromium}=require(process.argv[2]||'playwright')
const browser=await chromium.launch({headless:true,channel:'msedge'})
const artifacts=process.argv[3]
const errors=[]
let lastPage
const settings={client:{},product:{close_to_tray:true,autostart:false,reminder_hour:9,reminders:{},pinned_stickies:[],note_tags:{}},theme_mode:'light',accent:'blue',font_size:14,border_radius:16,sidebar_width:140,common_launchers:['computer','explorer'],welcome_completed:true,home_layout:[{id:'clock',kind:'clock',size:'2x1'},{id:'note',kind:'note',size:'1x1'},{id:'countdowns',kind:'countdowns',size:'1x1'}],launcher:{enabled:true,shortcut:'Alt+Space',minimal_mode:false},auto_backup:{enabled:true,interval_hours:24,retain:7}}
async function page(label='main'){
  const p=await browser.newPage({viewport:{width:1280,height:860}})
  lastPage=p;p.setDefaultTimeout(10_000);p.on('pageerror',error=>errors.push(error.message))
  await p.addInitScript(({settings,label})=>{
    const callbacks=new Map(), listeners=new Map();let callbackId=0
    const resources=[{id:'a',name:'测试应用A',kind:'app',target:'C:/A.exe',group_id:null,is_favorite:true,icon_data:null,aliases:'test',usage_count:1,last_opened_at:null},{id:'b',name:'测试应用B',kind:'app',target:'C:/B.exe',group_id:null,is_favorite:false,icon_data:null,aliases:'test',usage_count:0,last_opened_at:null}]
    const notes=[{id:'n',title:'测试笔记',content:'需要找到的测试内容',created_at:'2026-01-01',updated_at:'2026-01-01'}]
    const stickies=[]
    const countdowns=[{id:'d',title:'年度日期',target_date:'2000-12-31',repeat_type:'yearly',description:'测试',created_at:'2026-01-01',updated_at:'2026-01-01'}]
    window.isTauri=true
    window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}}
    window.__fixture={resources,notes,stickies,countdowns,settings,emit:(name,payload)=>{for(const handler of listeners.get(name)||[])callbacks.get(handler)?.({event:name,id:handler,payload})}}
    window.__TAURI_INTERNALS__={metadata:{currentWindow:{label},currentWebview:{label}},transformCallback:(fn)=>{callbacks.set(++callbackId,fn);return callbackId},unregisterCallback:id=>callbacks.delete(id),convertFileSrc:path=>path,invoke:async(command,args={})=>{
      if(command==='plugin:event|listen'){const list=listeners.get(args.event)||[];list.push(args.handler);listeners.set(args.event,list);return args.handler}
      if(command==='bootstrap_state')return {bootstrap:{data_dir:'fixture',default_data_dir:'fixture'},settings:structuredClone(settings),clean_uninstall:false,recovery_error:null}
      if(command==='read_local')return settings.client[args.key]??null
      if(command==='write_local'){settings.client[args.key]=args.value;return}
      if(command==='set_setting'){await new Promise(r=>setTimeout(r,20));settings[args.key]=args.value;return structuredClone(settings)}
      if(command==='list_notes')return structuredClone(notes)
      if(command==='list_stickies')return structuredClone(stickies)
      if(command==='list_groups'||command==='list_trash')return []
      if(command==='auto_backup_status')return {last_backup:null,count:0,directory:'fixture/backups',error:null}
      if(command==='launcher_status')return null
      if(command==='list_countdowns')return countdowns
      if(command==='list_resources')return structuredClone(resources)
      if(command==='common_launcher_icons')return {}
      if(command==='resource_health')return {a:true,b:false}
      if(command==='search_resources')return resources.filter(r=>(r.name+r.aliases).includes(args.query)).map(r=>({id:r.id,title:r.name,subtitle:r.target,kind:'resource'}))
      if(command==='search_all')return [{id:'n',title:'测试笔记',subtitle:'需要找到的测试内容',kind:'note'}]
      if(command==='launch_resource'){await new Promise(r=>setTimeout(r,250));resources.find(r=>r.id===args.id).usage_count++;return new Date().toISOString()}
      if(command==='toggle_favorite'){const r=resources.find(r=>r.id===args.id);r.is_favorite=!r.is_favorite;return}
      if(command==='create_resource'){const r={...args.value,id:'new',is_favorite:false,usage_count:0,icon_data:null,last_opened_at:null};resources.push(r);return r}
      if(command==='save_launch_options'){settings.client[`lumo-resource-options-${args.id}`]=args.options;return}
      if(command==='find_duplicate')return resources.find(r=>r.id!==args.exclude&&r.target.toLowerCase()===args.target.toLowerCase())?.id??null
      if(command==='preview_targets')return args.paths.map(target=>({target,kind:'app',name:'拖入的应用',icon_data:null}))
      if(command==='create_sticky'){const item={id:`s${stickies.length+1}`,title:args.title,content:args.content||'',created_at:'2026-01-01',updated_at:'2026-01-01'};stickies.push(item);return item}
      if(command==='create_note'){const item={id:`n${notes.length+1}`,title:args.title,content:args.content||'',created_at:'2026-01-01',updated_at:'2026-01-01'};notes.push(item);return item}
      if(command==='open_main_page'){for(const handler of listeners.get('lumo-navigate')||[])callbacks.get(handler)?.({event:command,id:handler,payload:{page:args.page,id:args.id}});return null}
      if(command==='test_notification')return null
      if(command==='update_sticky')return
      if(command==='plugin:window|is_maximized')return false
      if(command.startsWith('plugin:')||command==='open_startup_view'||command==='launch_common'||command==='configure_launcher')return null
      throw new Error(`Missing fixture: ${command}`)
    }}
  },{settings,label})
  await p.goto('http://localhost:1420/')
  return p
}
try{
  const main=await page()
  await main.getByRole('button',{name:'应用库',exact:true}).click()
  await main.getByRole('button',{name:'测试应用A',exact:true}).waitFor()
  await main.getByRole('button',{name:'测试应用A',exact:true}).click()
  await main.getByRole('button',{name:'测试应用A',exact:true}).filter({hasText:'正在打开…'}).waitFor()
  assert.equal(await main.getByRole('button',{name:'测试应用B',exact:true}).isEnabled(),true)
  await main.getByRole('button',{name:'测试应用A',exact:true}).waitFor()
  await main.getByRole('button',{name:'测试应用A',exact:true}).hover()
  await main.getByRole('tooltip').filter({hasText:'测试应用A'}).waitFor()
  const density=main.getByRole('combobox',{name:'网格密度'})
  await density.click();await main.getByRole('option',{name:'紧凑',exact:true}).click()
  const template=await main.locator('[data-resource-tile]').first().evaluate(node=>getComputedStyle(node.parentElement).gridTemplateColumns)
  assert.ok(template.split(' ').length>=8,template)
  await main.getByRole('button',{name:'添加',exact:true}).click()
  await main.getByLabel('名称',{exact:true}).fill('重复应用')
  await main.getByLabel('程序或文件路径').fill('C:/A.exe')
  await main.getByText('已存在：测试应用A').waitFor()
  await main.getByRole('button',{name:'取消',exact:true}).click()
  await main.evaluate(()=>window.__fixture.emit('tauri://drag-drop',{type:'drop',paths:['C:/Drop.exe'],position:{x:200,y:200}}))
  await main.getByRole('dialog').filter({hasText:'添加拖入的启动项'}).waitFor()
  await main.getByRole('button',{name:'添加所选',exact:true}).click()
  await main.getByRole('button',{name:'拖入的应用',exact:true}).waitFor()
  if(artifacts)await main.screenshot({path:`${artifacts}/library.png`})
  await main.getByRole('button',{name:'首页',exact:true}).click()
  await main.getByRole('button',{name:'快速记录',exact:true}).click()
  await main.getByLabel('记录内容').fill('一条新的想法')
  await main.getByRole('button',{name:'保存记录',exact:true}).click()
  await main.getByRole('dialog').waitFor({state:'hidden'})
  await main.getByRole('button',{name:'编辑首页',exact:true}).click()
  await main.getByRole('button',{name:'打开卡片库',exact:true}).click()
  await main.getByRole('button').filter({hasText:'天气'}).first().focus()
  await main.keyboard.press('Enter')
  await main.getByRole('button',{name:'撤销调整',exact:true}).click()
  await main.getByRole('button',{name:'取消',exact:true}).click()
  await main.keyboard.press('Control+k')
  assert.equal(await main.getByLabel('全局搜索').evaluate(node=>node===document.activeElement),true)
  if(artifacts)await main.screenshot({path:`${artifacts}/home.png`})
  const quick=await page('quick-launch')
  await quick.getByLabel('搜索内容').fill('测试')
  await quick.getByText('需要找到的测试内容').waitFor()
  assert.ok(await quick.locator('mark').count()>0)
  await quick.getByRole('combobox',{name:'搜索范围'}).click()
  await quick.getByRole('option',{name:'仅启动项',exact:true}).click()
  await quick.getByRole('button',{name:'用此关键词创建记录'}).waitFor()
  if(artifacts)await quick.screenshot({path:`${artifacts}/search.png`})
  await main.getByRole('button',{name:'便签',exact:true}).click()
  await main.getByRole('button',{name:'新建便签',exact:true}).click()
  await main.getByLabel('便签标题').fill('临时想法')
  await main.getByLabel('便签内容').fill('要整理成正式笔记的内容')
  await main.waitForTimeout(700)
  await main.getByRole('button',{name:'转为笔记'}).click()
  await main.getByText('要整理成正式笔记的内容').last().waitFor()
  await main.getByRole('button',{name:'更多笔记操作'}).click()
  await main.getByRole('menuitem',{name:'复制为便签'}).click()
  await main.getByLabel('便签内容').waitFor()
  assert.equal(await main.getByLabel('便签内容').inputValue(),'要整理成正式笔记的内容')
  await main.getByRole('button',{name:'设置',exact:true}).click()
  await main.getByRole('button',{name:'快捷启动',exact:true}).click()
  await main.getByRole('button',{name:'发送测试通知'}).click()
  await main.getByText('测试通知已发送，请确认 Windows 通知中心能看到它。').waitFor()
  assert.deepEqual(errors,[])
  console.log('UI checks passed: library and search, quick capture, sticky-to-note content flow, notification test action, launch feedback and home layout.')
}catch(error){console.error('Browser errors:',errors);console.error((await lastPage.locator('body').innerText()).slice(0,4000));if(artifacts)await lastPage.screenshot({path:`${artifacts}/failure.png`});throw error}finally{await browser.close()}
