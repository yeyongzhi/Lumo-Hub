use crate::AppState;
use chrono::{Datelike,NaiveDate,Local,Duration};
use tauri::{Manager,Emitter};
#[tauri::command]
pub async fn test_notification(app:tauri::AppHandle)->Result<String,String>{
    tauri::async_runtime::spawn_blocking(move || {
        crate::notifications::send(&app,"Lumo · 通知测试","这是一条测试通知；倒数日提醒会按设置时间发送。")
            .map_err(|e|format!("系统通知不可用：{e}"))?;
        let message=if cfg!(debug_assertions){"测试请求已提交给 Windows。开发预览的通知可能显示为 PowerShell；未弹出时请检查通知设置、横幅及勿扰模式。"}else{"测试请求已提交给 Windows。未弹出时请检查通知设置、横幅及勿扰模式，也可按 Win + N 查看通知中心。"};
        Ok(message.to_string())
    }).await.map_err(|e|format!("通知任务失败：{e}"))?
}
fn annual_date(date:NaiveDate,year:i32)->NaiveDate{
    NaiveDate::from_ymd_opt(year,date.month(),date.day()).unwrap_or_else(||NaiveDate::from_ymd_opt(year,2,28).unwrap())
}
pub fn next_occurrence(date:NaiveDate,today:NaiveDate)->NaiveDate{
    let this=annual_date(date,today.year());
    if this<today{annual_date(date,today.year()+1)}else{this}
}
pub fn target_date(date:NaiveDate,yearly:bool,today:NaiveDate)->NaiveDate{
    if !yearly{return date}
    let this=annual_date(date,today.year());
    // The page shows the next occurrence. Only notifications may catch up yesterday's
    // delivery, and their message explicitly says "补发提醒 · 昨天到期".
    if this==today-Duration::days(1){this}else{next_occurrence(date,today)}
}
pub fn due_offset(target:NaiveDate,offsets:&[i64],today:NaiveDate,hour:u32,reminder_hour:u8)->Option<i64>{
    if today>target+Duration::days(1){return None}
    offsets.iter().copied().filter(|offset|{let due=target-Duration::days(*offset);today>due||(today==due&&hour>=reminder_hour as u32)}).min()
}
fn check(app:&tauri::AppHandle)->Result<(),String>{
    use chrono::Timelike;
    let state=app.state::<AppState>();
    let _op=state.data_operation.lock().map_err(|_|"数据操作不可用")?;
    if state.recovery_error.lock().map_err(|_|"状态不可用")?.is_some(){return Ok(())}
    let options=state.settings.lock().map_err(|_|"设置不可用")?.product.clone();
    let now=Local::now();let today=now.date_naive();
    let db=state.db.lock().map_err(|_|"数据库不可用")?;
    let mut statement=db.prepare("SELECT id,title,target_date,repeat_type FROM countdowns").map_err(|e|e.to_string())?;
    let rows=statement.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).map_err(|e|e.to_string())?;
    for row in rows {
        let (id,title,date,repeat)=row.map_err(|e|e.to_string())?;
        let Some(offsets)=options.reminders.get(&id)else{continue};
        let Ok(date)=NaiveDate::parse_from_str(&date,"%Y-%m-%d")else{continue};
        let target=target_date(date,repeat=="yearly",today);
        let Some(offset)=due_offset(target,offsets,today,now.hour(),options.reminder_hour)else{continue};
        let key=format!("{id}|{target}|{offset}");
        let sent:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM reminder_deliveries WHERE key=?1)",[&key],|r|r.get(0)).map_err(|e|e.to_string())?;
        if sent{continue}
        let days=(target-today).num_days();
        let message=if days==0{format!("{title} · 就是今天")}else if days>0{format!("{title} · 还有 {days} 天")}else{format!("补发提醒：{title} · 昨天到期")};
        crate::notifications::send(app,"Lumo · 日期提醒",&message)?;
        db.execute("INSERT INTO reminder_deliveries(key,delivered_at) VALUES(?1,?2)",rusqlite::params![key,now.to_rfc3339()]).map_err(|e|e.to_string())?;
        let _=app.emit("lumo-reminder",&message);
    }
    db.execute("DELETE FROM reminder_deliveries WHERE delivered_at < ?1",[(now-Duration::days(400)).to_rfc3339()]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn start(app:tauri::AppHandle){std::thread::spawn(move||loop{if let Err(error)=check(&app){let _=app.emit("lumo-reminder-error",error);}std::thread::sleep(std::time::Duration::from_secs(60));});}
#[cfg(test)]
mod tests{
    use super::*;
    fn day(value:&str)->NaiveDate{NaiveDate::parse_from_str(value,"%Y-%m-%d").unwrap()}
    #[test]fn catchup_chooses_one_latest_milestone(){let target=day("2026-10-10");assert_eq!(due_offset(target,&[0,1,7],target,10,9),Some(0));assert_eq!(due_offset(target,&[0,1,7],target,8,9),Some(1));assert_eq!(due_offset(target,&[0,1,7],day("2026-10-12"),10,9),None);}
    #[test]fn yearly_and_leap_day(){assert_eq!(target_date(day("2024-02-29"),true,day("2026-02-01")),day("2026-02-28"));assert_eq!(target_date(day("2024-10-10"),true,day("2026-10-12")),day("2027-10-10"));}
    #[test]fn next_date_and_catchup_are_distinct(){
        assert_eq!(next_occurrence(day("2024-02-29"),day("2027-03-01")),day("2028-02-29"));
        assert_eq!(next_occurrence(day("2000-10-13"),day("2026-10-14")),day("2027-10-13"));
        assert_eq!(target_date(day("2000-10-13"),true,day("2026-10-14")),day("2026-10-13"));
        assert_eq!(next_occurrence(day("2000-01-03"),day("2026-12-31")),day("2027-01-03"));
    }
}
