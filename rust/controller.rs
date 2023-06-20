use std::time::{SystemTime, Duration};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const KEY_CTL_QUIT: i32 = 113; // 'q'
const KEY_CTL_PAUSE: i32 = 112; // 'p'
const KEY_CTL_LEFT: i32 = 104; // 'h'
const KEY_CTL_UP: i32 = 107; // 'k'
const KEY_CTL_RIGHT: i32 = 108; // 'l'
const KEY_CTL_DOWN: i32 = 106; // 'j'
const KEY_CTL_FAST: i32 = 43; // '+'
const KEY_CTL_SLOW: i32 = 45; // '-'

enum ControlStatus {
    Running,
    Pause,
    Stop,
}

struct Controller {
    status: ControlStatus,
    speed: u32,
    nodelay: bool,
    ctime: SystemTime,
}

impl Controller {
    fn new() -> Self {
        Controller {
            status: ControlStatus::Stop,
            speed: 0,
            nodelay: false,
            ctime: SystemTime::now(),
        }
    }
}

struct ControllerHandle {
    controller: Arc<Controller>,
    key_sender: Sender<i32>,
}

impl ControllerHandle {
    fn new(controller: Arc<Controller>, key_sender: Sender<i32>) -> Self {
        ControllerHandle {
            controller,
            key_sender,
        }
    }

    fn handle_key_event(&self, ch: i32) {
        if ch == KEY_CTL_QUIT {
            self.controller.status = ControlStatus::Stop;
        } else if ch == KEY_CTL_PAUSE {
            if self.controller.status == ControlStatus::Running {
                self.controller.status = ControlStatus::Pause;
            } else {
                self.controller.status = ControlStatus::Running;
            }
        } else {
            match ch {
                KEY_CTL_LEFT | 104 => {
                    if g_snake.dirc != Direction::Right {
                        self.controller.nodelay = true;
                    }
                    g_snake.dirc = Direction::Left;
                }
                KEY_CTL_UP | 107 => {
                    if g_snake.dirc != Direction::Down {
                        self.controller.nodelay = true;
                    }
                    g_snake.dirc = Direction::Up;
                }
                KEY_CTL_RIGHT | 108 => {
                    if g_snake.dirc != Direction::Left {
                        self.controller.nodelay = true;
                    }
                    g_snake.dirc = Direction::Right;
                }
                KEY_CTL_DOWN | 106 => {
                    if g_snake.dirc != Direction::Up {
                        self.controller.nodelay = true;
                    }
                    g_snake.dirc = Direction::Down;
                }
                KEY_CTL_FAST => {
                    if self.controller.speed < MAX_SPEED {
                        self.controller.speed += 1;
                    }
                }
                KEY_CTL_SLOW => {
                    if self.controller.speed > MIN_SPEED {
                        self.controller.speed -= 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn init_controller() -> ControllerHandle {
    let controller = Arc::new(Controller::new());
    let (key_sender, key_receiver) = mpsc::channel();

    thread::spawn(move || {
        initscr();
        nodelay(stdscr(), true);
        keypad(stdscr(), true);

        let handle = ControllerHandle::new(controller.clone(), key_sender);

        loop {
            if let Ok(ch) = getch() {
                handle.handle_key_event(ch);
            }

            if controller.status == ControlStatus::Stop {
                break;
            }

            move_snake(&g_snake);

            mainscene();

            if g_snake.state != SnakeState::Normal {
                break;
            }

            if !controller.nodelay {
                thread::sleep(Duration::from_millis(1000 / controller.speed as u64));
            } else {
                controller.nodelay = false;
            }

            // Handle key events
            while let Ok(ch) = key_receiver.try_recv() {
                handle.handle_key_event(ch);
            }

            if controller.status == ControlStatus::Pause {
                pausescene();
                thread::sleep(Duration::from_millis(100));
            }
        }

        endwin();
    });

    ControllerHandle::new(controller, key_sender)
}
