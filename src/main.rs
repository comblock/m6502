use std::sync::{Mutex, Arc};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, ContextBuilder, GameResult, GameError};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};

fn main() {
    let program = include_bytes!(concat!(env!("OUT_DIR"), "/program"));

    let mem = Arc::new(Mutex::new([0u8; 2usize.pow(16)]));
    for (i, byte) in program.iter().enumerate() {
        mem.lock().unwrap()[0x0200 + i] = *byte;        
    }

    let bus = Bus::new(mem.clone());
    let mut cpu = m6502::Cpu::new(bus, Clock);
    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("6502 snake", "")
        .build()
        .unwrap();

    let handle = std::thread::spawn(move || cpu.run());
    let state = State::new(&mut ctx, mem);
    handle.join().unwrap();
    event::run(ctx, event_loop, state);
}

struct State {
    mem: Arc<Mutex<[u8; 2usize.pow(16)]>>
}

impl State {
    pub fn new(_ctx: &mut Context, mem: Arc<Mutex<[u8; 2usize.pow(16)]>>) -> State {
        State {
            mem
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        canvas.finish(ctx)
    }

    fn key_down_event(
            &mut self,
            ctx: &mut Context,
            input: ggez::input::keyboard::KeyInput,
            _repeated: bool,
        ) -> Result<(), GameError> {
        if let Some(keycode) = input.keycode {
            match keycode {
                // set the direction in memory somewhere
                KeyCode::Up => todo!(),
                KeyCode::Down => todo!(),
                KeyCode::Left => todo!(),
                KeyCode::Right => todo!(),

                _ => todo!() // do nothing
            }
        }    
        Ok(())
    }
}

struct Clock;

impl m6502::Clock for Clock {
    fn cycles(&mut self, n: u8) {
        std::thread::sleep(std::time::Duration::from_nanos(500 * n as u64)) // This makes sure the CPU runs at 2 MHz
    }
}

struct Bus(Arc<Mutex<[u8; 2usize.pow(16)]>>);

impl Bus {
    pub fn new(inner: Arc<Mutex<[u8; 2usize.pow(16)]>>) -> Self {
        Self(inner)
    }
}


impl m6502::Bus for Bus {
    fn load(&self, addr: u16) -> u8 {
        self.0.lock().unwrap()[addr as usize]
    }

    fn store(&mut self, addr: u16, value: u8) {
        self.0.lock().unwrap()[addr as usize] = value;
    }
}