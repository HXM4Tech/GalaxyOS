use core::arch::asm;
use spin::Mutex;

pub fn configure_pit() {
    assert_has_not_been_called!("PIT can be configured only once");

    unsafe {
        asm!(
            "mov al, 36h",
            "out 43h, al",
            "mov ax, 1193",
            "out 40h, al",
            "mov al, ah",
            "out 40h, al",
        );
    }
}

#[derive(Clone, Copy)]
pub struct Timer {
    time: u64,
}

impl Timer {
    pub fn increment(&mut self) {
        self.time += 1;
    }

    pub fn read(self) -> u64 {
        return self.time;
    }
}

static TIMER: Mutex<Timer> = Mutex::new(Timer{time: 0});

pub fn pit_interrupt() {
    x86_64::instructions::interrupts::without_interrupts(|| {TIMER.lock().increment()});
}

pub fn sleep(ms: u64) {
    if ms == 0 {
        return;
    }

    let time = x86_64::instructions::interrupts::without_interrupts(|| {TIMER.lock().read() });
    let end_time = time + ms;

    loop {
        let time = x86_64::instructions::interrupts::without_interrupts(|| {TIMER.lock().read() });
        if time >= end_time {
            return;
        }
        x86_64::instructions::hlt();
    }
}

pub fn get_uptime() -> u64 {
    return TIMER.lock().read();
}
