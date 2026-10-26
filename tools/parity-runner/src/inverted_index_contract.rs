#[derive(Clone, Copy)]
struct Handle { owner: u64, slot: usize, generation: u64 }
#[derive(Clone, Copy)]
struct Slot { row: usize, generation: u64, live: bool }

struct Rows {
    owner: u64,
    rows: Vec<usize>,
    slots: Vec<Slot>,
    free: Vec<usize>,
    closed: bool,
}

impl Rows {
    fn new(owner: u64) -> Self {
        Self { owner, rows: vec![], slots: vec![], free: vec![], closed: false }
    }
    fn push(&mut self) -> Result<Handle, String> {
        if self.closed { return Err("closed".into()); }
        if self.rows.len() >= i32::MAX as usize { return Err("row count".into()); }
        let row = self.rows.len();
        let slot = if let Some(slot) = self.free.pop() { slot } else {
            self.slots.push(Slot { row:0, generation:1, live:false });
            self.slots.len()-1
        };
        self.slots[slot].row=row; self.slots[slot].live=true; self.rows.push(slot);
        Ok(self.issue(slot))
    }
    fn index(&self, value: i64) -> Result<Handle, String> {
        if self.closed { return Err("closed".into()); }
        if value < 0 || value > i32::MAX as i64 { return Err("bounds".into()); }
        let row=value as usize;
        if row >= self.rows.len() { return Err("bounds".into()); }
        let slot=self.rows[row];
        let state=self.slots.get(slot).ok_or("corrupt")?;
        if !state.live || state.row != row { return Err("corrupt".into()); }
        Ok(self.issue(slot))
    }
    fn resolve(&self, h: Handle) -> Result<usize, String> {
        if self.closed { return Err("closed".into()); }
        if h.owner != self.owner { return Err("foreign".into()); }
        let state=self.slots.get(h.slot).ok_or("foreign")?;
        if !state.live || state.generation != h.generation { return Err("stale".into()); }
        Ok(state.row)
    }
    fn erase_swap(&mut self, h: Handle) -> Result<(), String> {
        let row=self.resolve(h)?;
        let last=self.rows.len()-1;
        if row != last { let moved=self.rows[last]; self.rows[row]=moved; self.slots[moved].row=row; }
        self.rows.pop();
        let state=&mut self.slots[h.slot]; state.live=false;
        if state.generation != u64::MAX { state.generation+=1; self.free.push(h.slot); }
        Ok(())
    }
    fn issue(&self, slot: usize) -> Handle {
        Handle { owner:self.owner, slot, generation:self.slots[slot].generation }
    }
}

pub fn self_check() -> Result<(), String> {
    let mut rows=Rows::new(1);
    for bad in [i64::MIN,-1,i32::MAX as i64+1,i64::MAX] {
        if rows.index(bad).as_deref() != Err("bounds") { return Err("index boundary drift".into()); }
    }
    if rows.index(0).as_deref() != Err("bounds") { return Err("empty index drift".into()); }
    let first=rows.push()?;
    let middle=rows.push()?;
    let last=rows.push()?;
    let saved=rows.index(1)?;
    if saved.slot != middle.slot || saved.generation != middle.generation {
        return Err("dense index did not capture logical row".into());
    }
    rows.erase_swap(first)?;
    if rows.resolve(last)? != 0 || rows.resolve(saved)? != 1 || rows.index(0)?.slot != last.slot {
        return Err("swap-delete identity drift".into());
    }
    rows.erase_swap(saved)?;
    if rows.resolve(saved).as_deref() != Err("stale") { return Err("stale handle revived".into()); }
    let reused=rows.push()?;
    if reused.slot != middle.slot || reused.generation == middle.generation {
        return Err("ABA generation drift".into());
    }
    if rows.resolve(saved).as_deref() != Err("stale") { return Err("ABA stale handle revived".into()); }
    let mut foreign=Rows::new(2);
    let other=foreign.push()?;
    if rows.resolve(other).as_deref() != Err("foreign") { return Err("owner fence drift".into()); }
    rows.closed=true;
    if rows.index(0).as_deref() != Err("closed") { return Err("close fence drift".into()); }
    Ok(())
}
