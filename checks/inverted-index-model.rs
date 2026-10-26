use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Handle { owner: u64, slot: usize, generation: u64 }

#[derive(Clone, Copy, Debug)]
struct Slot { row: usize, generation: u64, live: bool }

struct Rows {
    owner: u64,
    rows: Vec<usize>,
    slots: Vec<Slot>,
    free: Vec<usize>,
    closed: bool,
}

impl Rows {
    fn new() -> Self {
        let owner = NEXT_OWNER.fetch_add(1, Ordering::Relaxed);
        assert!(owner != 0 && owner != u64::MAX);
        Self { owner, rows: vec![], slots: vec![], free: vec![], closed: false }
    }

    fn push(&mut self) -> Handle {
        assert!(!self.closed);
        assert!(self.rows.len() < i32::MAX as usize);
        let row = self.rows.len();
        let slot = if let Some(slot) = self.free.pop() {
            slot
        } else {
            self.slots.push(Slot { row: 0, generation: 1, live: false });
            self.slots.len() - 1
        };
        self.slots[slot].row = row;
        self.slots[slot].live = true;
        self.rows.push(slot);
        self.issue(slot)
    }

    fn index(&self, value: i64) -> Result<Handle, &'static str> {
        if self.closed { return Err("closed"); }
        if value < 0 || value > i32::MAX as i64 { return Err("bounds"); }
        let row = value as usize;
        if row >= self.rows.len() { return Err("bounds"); }
        let slot = self.rows[row];
        let state = self.slots.get(slot).ok_or("corrupt")?;
        if !state.live || state.row != row { return Err("corrupt"); }
        Ok(self.issue(slot))
    }

    fn resolve(&self, handle: Handle) -> Result<usize, &'static str> {
        if self.closed { return Err("closed"); }
        if handle.owner != self.owner { return Err("foreign"); }
        let state = self.slots.get(handle.slot).ok_or("foreign")?;
        if !state.live || state.generation != handle.generation { return Err("stale"); }
        Ok(state.row)
    }

    fn erase_swap(&mut self, handle: Handle) -> Result<(), &'static str> {
        let row = self.resolve(handle)?;
        let last = self.rows.len() - 1;
        if row != last {
            let moved = self.rows[last];
            self.rows[row] = moved;
            self.slots[moved].row = row;
        }
        self.rows.pop();
        let state = &mut self.slots[handle.slot];
        state.live = false;
        if state.generation != u64::MAX {
            state.generation += 1;
            self.free.push(handle.slot);
        }
        Ok(())
    }

    fn issue(&self, slot: usize) -> Handle {
        Handle { owner: self.owner, slot, generation: self.slots[slot].generation }
    }

    fn close(&mut self) {
        self.closed = true;
        for slot in &mut self.slots { slot.live = false; }
    }
}

#[test]
fn dense_index_tracks_logical_identity_and_aba() {
    let mut rows = Rows::new();
    for bad in [i64::MIN, -1, i32::MAX as i64 + 1, i64::MAX] {
        assert_eq!(rows.index(bad), Err("bounds"));
    }
    assert_eq!(rows.index(0), Err("bounds"));

    let first = rows.push();
    let middle = rows.push();
    let last = rows.push();
    let saved_middle = rows.index(1).unwrap();
    assert_eq!(saved_middle, middle);
    assert_eq!(rows.index(3), Err("bounds"));

    rows.erase_swap(first).unwrap();
    assert_eq!(rows.resolve(last), Ok(0));
    assert_eq!(rows.resolve(saved_middle), Ok(1));
    assert_eq!(rows.index(0).unwrap().slot, last.slot);

    rows.erase_swap(saved_middle).unwrap();
    assert_eq!(rows.resolve(saved_middle), Err("stale"));
    let reused = rows.push();
    assert_eq!(reused.slot, middle.slot);
    assert_ne!(reused.generation, middle.generation);
    assert_eq!(rows.resolve(saved_middle), Err("stale"));
    assert_eq!(rows.resolve(rows.index(1).unwrap()), Ok(1));
}

#[test]
fn owner_and_close_fences_are_part_of_the_contract() {
    let mut a = Rows::new();
    let mut b = Rows::new();
    let a0 = a.push();
    let b0 = b.push();
    assert_eq!(a.resolve(b0), Err("foreign"));
    assert_eq!(b.resolve(a0), Err("foreign"));
    a.close();
    assert_eq!(a.resolve(a0), Err("closed"));
    assert_eq!(a.index(0), Err("closed"));
}
