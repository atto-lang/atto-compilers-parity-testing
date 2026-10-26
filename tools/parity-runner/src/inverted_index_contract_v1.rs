#[derive(Clone, Copy)]
struct Handle { owner:u64, slot:usize, generation:u64 }
#[derive(Clone, Copy)]
struct Slot { row:usize, generation:u64, live:bool }
struct Rows { owner:u64, rows:Vec<usize>, slots:Vec<Slot>, free:Vec<usize>, closed:bool }

impl Rows {
    fn new(owner:u64)->Self { Self{owner,rows:vec![],slots:vec![],free:vec![],closed:false} }
    fn issue(&self,slot:usize)->Handle { Handle{owner:self.owner,slot,generation:self.slots[slot].generation} }
    fn push(&mut self)->Result<Handle,String>{
        if self.closed{return Err("closed".into())}
        let row=self.rows.len();
        if row>=i32::MAX as usize{return Err("row count".into())}
        let slot=if let Some(s)=self.free.pop(){s}else{
            self.slots.push(Slot{row:0,generation:1,live:false});self.slots.len()-1};
        self.slots[slot].row=row;self.slots[slot].live=true;self.rows.push(slot);Ok(self.issue(slot))
    }
    fn index(&self,value:i64)->Result<Handle,String>{
        if self.closed{return Err("closed".into())}
        if value<0||value>i32::MAX as i64{return Err("bounds".into())}
        let row=value as usize;if row>=self.rows.len(){return Err("bounds".into())}
        let slot=self.rows[row];let s=self.slots.get(slot).ok_or("corrupt")?;
        if !s.live||s.row!=row{return Err("corrupt".into())} Ok(self.issue(slot))
    }
    fn resolve(&self,h:Handle)->Result<usize,String>{
        if self.closed{return Err("closed".into())}
        if h.owner!=self.owner{return Err("foreign".into())}
        let s=self.slots.get(h.slot).ok_or("foreign")?;
        if !s.live||s.generation!=h.generation{return Err("stale".into())} Ok(s.row)
    }
    fn erase_swap(&mut self,h:Handle)->Result<(),String>{
        let row=self.resolve(h)?;let last=self.rows.len()-1;
        if row!=last{let moved=self.rows[last];self.rows[row]=moved;self.slots[moved].row=row}
        self.rows.pop();let s=&mut self.slots[h.slot];s.live=false;
        if s.generation!=u64::MAX{s.generation+=1;self.free.push(h.slot)} Ok(())
    }
}
fn err_is<T>(r:Result<T,String>,expected:&str)->bool{r.err().as_deref()==Some(expected)}

pub fn self_check()->Result<(),String>{
    let mut rows=Rows::new(1);
    for bad in [i64::MIN,-1,i32::MAX as i64+1,i64::MAX] {
        if !err_is(rows.index(bad),"bounds"){return Err("index boundary drift".into())}
    }
    if !err_is(rows.index(0),"bounds"){return Err("empty index drift".into())}
    let first=rows.push()?;let middle=rows.push()?;let last=rows.push()?;let saved=rows.index(1)?;
    if saved.slot!=middle.slot||saved.generation!=middle.generation{return Err("index identity drift".into())}
    rows.erase_swap(first)?;
    if rows.resolve(last)?!=0||rows.resolve(saved)?!=1||rows.index(0)?.slot!=last.slot{return Err("swap identity drift".into())}
    rows.erase_swap(saved)?;
    if !err_is(rows.resolve(saved),"stale"){return Err("stale handle survived delete".into())}
    let reused=rows.push()?;
    if reused.slot!=middle.slot||reused.generation==middle.generation{return Err("ABA generation drift".into())}
    if !err_is(rows.resolve(saved),"stale"){return Err("ABA revived stale handle".into())}
    let mut foreign=Rows::new(2);let other=foreign.push()?;
    if !err_is(rows.resolve(other),"foreign"){return Err("owner fence drift".into())}
    rows.closed=true;
    if !err_is(rows.index(0),"closed"){return Err("close fence drift".into())}
    Ok(())
}
