use std::sync::{Mutex, Arc};
use std::thread::JoinHandle;
use std::time::{Instant, Duration};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, ContextBuilder, GameResult, GameError};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};
use m6502::Cpu;

const GRID: u8 = 16;
const TILE_SIZE: i32 = 32;
const SCREEN_SIZE: f32 = GRID as f32 * TILE_SIZE as f32;

fn main() {
    let program = include_bytes!(concat!(env!("OUT_DIR"), "/program"));

    let mem = Arc::new(Mutex::new([0u8; 2usize.pow(16)]));
    // Initialise the memory to random values
    for i in mem.lock().unwrap().iter_mut() {
        *i = rand::random()
    };

    for (i, byte) in program.iter().enumerate() {
        mem.lock().unwrap()[0x0200 + i] = *byte;        
    }

    let bus = Bus::new(mem.clone());
    let mut cpu = m6502::Cpu::new(bus, Clock);
    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("6502 snake", "")
        .window_setup(ggez::conf::WindowSetup::default().title("snake on the 6502!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE, SCREEN_SIZE))
        .build()
        .unwrap();

    let handle = std::thread::spawn(move || run(&mut cpu));
    let state = State::new(&mut ctx, mem, handle);
    event::run(ctx, event_loop, state);
}

struct State {
    handle: JoinHandle<()>,
    mem: Arc<Mutex<[u8; 2usize.pow(16)]>>
}

impl State {
    pub fn new(_ctx: &mut Context, mem: Arc<Mutex<[u8; 2usize.pow(16)]>>, handle: JoinHandle<()>) -> State {
        State {
            handle,
            mem
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.handle.is_finished() {
            // Sleep for 3 seconds, then exit the program
            std::thread::sleep(std::time::Duration::from_secs(3));
            std::process::exit(0);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        for (i, v) in self.mem.lock().unwrap()[0xfd00..0xfe00].iter().enumerate() {
            let v = *v &0x03;
            let rect = graphics::Rect::new_i32((i as u8 & 0x0f) as i32 * TILE_SIZE, (i as u8 >> 4) as i32 * TILE_SIZE, TILE_SIZE, TILE_SIZE);
            canvas.draw(&graphics::Quad, graphics::DrawParam::new().dest_rect(rect).color(match v {
                0 => {
                    [0.0, 0.0, 0.0, 1.0]
                }
                1 => {
                    [0.0, 1.0, 0.0, 1.0]
                }
                2 => {
                    [1.0, 0.0, 0.0, 1.0]
                }
                3 => {
                    [0.0, 0.0, 1.0, 1.0]
                }
                _ => unreachable!()
            }
            ))
        }
        canvas.finish(ctx)
    }

    fn key_down_event(
            &mut self,
            _ctx: &mut Context,
            input: ggez::input::keyboard::KeyInput,
            _repeated: bool,
        ) -> Result<(), GameError> {
        if let Some(keycode) = input.keycode {
            let mut mem = self.mem.lock().unwrap();
            let prev_direction = mem[0x00ff];
            let direction = match keycode {
                // set the direction in memory somewhere
                KeyCode::Up => 3,
                KeyCode::Down => 0,
                KeyCode::Left => 1,
                KeyCode::Right => 2,

                _ => return Ok(())
            };
            
            if (!direction & 0x03) != prev_direction {
                mem[0x00ff] = direction;
            }
        }    
        Ok(())
    }
}

#[derive(Debug)]
struct Clock;

impl m6502::Clock for Clock {
    fn cycles(&mut self, n: u8, start: Instant) {
        // This ensures the emulator runs at ~80kHz, it is much more reliable than using std::thread::sleep because this doesn't need a syscall
        while start.elapsed() < Duration::from_micros(12 * n as u64) {
          std::hint::spin_loop();
        }  
    }
}

#[derive(Debug)]
struct Bus(Arc<Mutex<[u8; 2usize.pow(16)]>>);

impl Bus {
    pub fn new(inner: Arc<Mutex<[u8; 2usize.pow(16)]>>) -> Self {
        Self(inner)
    }
    pub fn print_snake(&self) {
        println!("{:?}", &self.0.lock().unwrap()[0xfe00..0xff00]);
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

fn run(cpu: &mut Cpu<Bus, Clock>) {
    use m6502::Bus;
    loop {
        let instruction = cpu.fetch();
        let brk = cpu.execute(instruction);
        //println!("{:?}, PC:{:04x}, X:{}, Y:{}, S:{:08b}, A:{}, 0x0010:{}", instruction, cpu.pc, cpu.x, cpu.y, cpu.status, cpu.accumulator, cpu.bus.load(0x0010));
        if brk {
            break;
        };
        cpu.bus.store(0x00, 0);
        cpu.bus.store(0x01, rand::random());
    }
}