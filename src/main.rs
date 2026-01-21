mod game;

use std::process;
use game::{Game, Direction};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

fn main() {
    // 启用原始模式
    enable_raw_mode().unwrap();

    let mut current_level = 1;
    let max_level = 3;

    loop {
        let mut game = Game::new(current_level);
        game.render();

        loop {
            // 读取按键事件
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                let moved = match code {
                    KeyCode::Char('w') | KeyCode::Up => game.move_player(Direction::Up),
                    KeyCode::Char('s') | KeyCode::Down => game.move_player(Direction::Down),
                    KeyCode::Char('a') | KeyCode::Left => game.move_player(Direction::Left),
                    KeyCode::Char('d') | KeyCode::Right => game.move_player(Direction::Right),
                    KeyCode::Char('q') => {
                        disable_raw_mode().unwrap();
                        println!("游戏退出！");
                        process::exit(0);
                    }
                    KeyCode::Char('r') => {
                        game.reset();
                        true
                    }
                    _ => false,
                };

                if moved {
                    game.render();

                    if game.is_win() {
                        print!("\r\n🎉 恭喜！你完成了关卡 {}！用了 {} 步。\r\n", current_level, game.moves);
                        print!("按任意键继续下一关，或按 'q' 退出...\r\n");

                        // 等待按键
                        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                            if let KeyCode::Char('q') = code {
                                disable_raw_mode().unwrap();
                                println!("游戏退出！");
                                process::exit(0);
                            }
                        }

                        current_level += 1;
                        if current_level > max_level {
                            disable_raw_mode().unwrap();
                            println!("\n🎊 恭喜！你完成了所有关卡！");
                            process::exit(0);
                        }
                        break; // 进入下一关
                    }
                }
            }
        }
    }
}
