mod game;
mod solver;

use std::process;
use std::time::Instant;
use std::io::{self, Write};
use game::{Game, Direction};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};

fn main() {
    // 获取所有关卡
    let level_files = Game::get_all_levels();
    
    if level_files.is_empty() {
        println!("错误: 没有找到任何关卡文件！请在 levels/ 目录下添加 .txt 关卡文件。");
        return;
    }

    println!("🎮 推箱子游戏");
    println!("═══════════════════════════════════════════════════════════════");
    println!("正在检测所有关卡是否可解...\n");

    // 打印表头
    println!("┌───────────────────────────────────────────────────────────────┐");
    println!("│  关卡名称              │ 状态     │ 最小步数 │ 检测耗时     │");
    println!("├───────────────────────────────────────────────────────────────┤");
    io::stdout().flush().unwrap();

    // 检测所有关卡的可解性（实时输出）
    for level_path in &level_files {
        let level_name = std::path::Path::new(level_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // 显示正在检测
        print!("│  {:<20} │ 检测中... ", level_name);
        io::stdout().flush().unwrap();
        
        let (map, player_pos) = Game::load_level_from_path(level_path);
        
        let start = Instant::now();
        let result = solver::solve(&map, player_pos);
        let duration = start.elapsed().as_millis();
        
        // 清除"检测中..."并输出结果
        let status = if result.solvable { "✅ 可解" } else { "❌ 不可解" };
        let steps_str = match result.min_steps {
            Some(s) => format!("{:>6}", s),
            None => "   N/A".to_string(),
        };
        print!("\r│  {:<20} │ {} │ {} 步 │ {:>8} ms  │\n", level_name, status, steps_str, duration);
        io::stdout().flush().unwrap();
    }
    
    println!("└───────────────────────────────────────────────────────────────┘");
    println!("\n共 {} 个关卡", level_files.len());
    println!("\n按任意键开始游戏，按 'q' 退出...");
    io::stdout().flush().unwrap();

    // 启用原始模式
    enable_raw_mode().unwrap();

    // 等待用户确认开始
    if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
        if let KeyCode::Char('q') = code {
            disable_raw_mode().unwrap();
            println!("\r\n游戏退出！");
            process::exit(0);
        }
    }

    let max_level = level_files.len();
    let mut current_level = 0;

    loop {
        if current_level >= max_level {
            disable_raw_mode().unwrap();
            println!("\r\n🎊 恭喜！你完成了所有关卡！");
            process::exit(0);
        }

        let level_path = &level_files[current_level];
        let mut game = Game::new(current_level + 1, level_path);
        game.render();

        loop {
            // 读取按键事件
            if let Ok(Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. })) = event::read() {
                let moved = match code {
                    KeyCode::Char('w') | KeyCode::Up => game.move_player(Direction::Up),
                    KeyCode::Char('s') | KeyCode::Down => game.move_player(Direction::Down),
                    KeyCode::Char('a') | KeyCode::Left => game.move_player(Direction::Left),
                    KeyCode::Char('d') | KeyCode::Right => game.move_player(Direction::Right),
                    KeyCode::Char('q') => {
                        disable_raw_mode().unwrap();
                        println!("\r\n游戏退出！");
                        process::exit(0);
                    }
                    KeyCode::Char('r') => {
                        game.reset(level_path);
                        true
                    }
                    _ => false,
                };

                if moved {
                    game.render();

                    if game.is_win() {
                        print!("\r\n🎉 恭喜！你完成了 {}！用了 {} 步。\r\n", game.level_name, game.moves);
                        print!("按任意键继续下一关，或按 'q' 退出...\r\n");

                        // 等待按键
                        if let Ok(Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. })) = event::read() {
                            if let KeyCode::Char('q') = code {
                                disable_raw_mode().unwrap();
                                println!("\r\n游戏退出！");
                                process::exit(0);
                            }
                        }

                        current_level += 1;
                        break; // 进入下一关
                    }
                }
            }
        }
    }
}
