//! Line input from the serial console, with echo and backspace.

use core::cell::Cell;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_io_async::Read;
use esp_hal::Async;
use esp_hal::usb::usb_serial_jtag::UsbSerialJtagRx;

use crate::out;

/// Longest accepted line.
pub const LINE_MAX: usize = 64;

/// One line typed at the console.
#[derive(Clone, Copy)]
pub struct Line {
    data: [u8; LINE_MAX],
    len: usize,
}

impl Line {
    const EMPTY: Line = Line { data: [0; LINE_MAX], len: 0 };

    /// The raw bytes typed so far.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// The line as text; invalid UTF-8 reads as empty.
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}

/// The line being typed, so other output can redraw it (see `out::line`).
static PARTIAL: Mutex<CriticalSectionRawMutex, Cell<Line>> = Mutex::new(Cell::new(Line::EMPTY));

/// The line being typed, if any.
pub fn visible_partial() -> Option<Line> {
    let line = PARTIAL.lock(Cell::get);
    (line.len > 0).then_some(line)
}

/// Completed lines for the app.
pub static LINES: Channel<CriticalSectionRawMutex, Line, 4> = Channel::new();

/// Reads bytes, echoes them, and queues each line on Enter.
#[embassy_executor::task]
pub async fn task(mut rx: UsbSerialJtagRx<'static, Async>) {
    let mut line = Line::EMPTY;
    let mut buf = [0u8; 32];
    loop {
        // USB reads can't fail.
        let Ok(n) = rx.read(&mut buf).await;
        for &b in &buf[..n] {
            match b {
                b'\r' | b'\n' => {
                    out::write(b"\r\n").await;
                    PARTIAL.lock(|p| p.set(Line::EMPTY));
                    if line.len > 0 {
                        // Drop the line rather than block if the app is busy.
                        let _ = LINES.try_send(line);
                        line.len = 0;
                    }
                }
                0x08 | 0x7F if line.len > 0 => {
                    line.len -= 1;
                    out::write(b"\x08 \x08").await;
                }
                0x20..=0x7E if line.len < LINE_MAX => {
                    line.data[line.len] = b;
                    line.len += 1;
                    out::write(&[b]).await;
                }
                _ => {}
            }
            PARTIAL.lock(|p| p.set(line));
        }
    }
}
