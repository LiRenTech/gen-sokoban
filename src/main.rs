use std::io::{self, Write};
use std::process;

// 地图元素
#[derive(Clone, Copy, PartialEq)]
enum Tile {
    Wall,      // 墙
    Floor,     // 地板
    Target,    // 目标位置
    Box,       // 箱子
    BoxOnTarget, // 箱子在目标上
    Player,    // 玩家
    PlayerOnTarget, // 玩家在目标上
}

// 方向
#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// 游戏状态
struct Game {
    level: usize,
    map: Vec<Vec<Tile>>,
    player_pos: (usize, usize),
    moves: u32,
}

impl Game {
    fn new(level: usize) -> Self {
        let (map, player_pos) = Self::load_level(level);
        Game {
            level,
            map,
            player_pos,
            moves: 0,
        }
    }

    // 加载关卡
    fn load_level(level: usize) -> (Vec<Vec<Tile>>, (usize, usize)) {
        let levels = vec![
            // 关卡 1 - 简单
            vec![
                "#######",
                "#.X...#",
                "#.@...#",
                "#.$...#",
                "#.....#",
                "#######",
            ],
            // 关卡 2 - 中等
            vec![
                "########",
                "#......#",
                "#.X.X..#",
                "#.@....#",
                "#.$.$..#",
                "#......#",
                "########",
            ],
            // 关卡 3 - 困难
            vec![
                "########",
                "#......#",
                "#.X.X..#",
                "#......#",
                "#.@....#",
                "#.$.$..#",
                "#......#",
                "########",
            ],
        ];

        if level > levels.len() {
            return Self::load_level(1);
        }

        let level_data = &levels[level - 1];
        let mut map = Vec::new();
        let mut player_pos = (0, 0);

        for (y, row) in level_data.iter().enumerate() {
            let mut map_row = Vec::new();
            for (x, ch) in row.chars().enumerate() {
                let tile = match ch {
                    '#' => Tile::Wall,
                    '.' => Tile::Floor,
                    '@' => {
                        player_pos = (x, y);
                        Tile::Player
                    }
                    '$' => Tile::Box,
                    '*' => Tile::BoxOnTarget,
                    '+' => {
                        player_pos = (x, y);
                        Tile::PlayerOnTarget
                    }
                    'X' => Tile::Target,
                    _ => Tile::Floor,
                };
                map_row.push(tile);
            }
            map.push(map_row);
        }

        (map, player_pos)
    }

    // 渲染地图
    fn render(&self) {
        print!("\x1B[2J\x1B[1;1H"); // 清屏
        println!("推箱子游戏 - 关卡 {} | 步数: {}", self.level, self.moves);
        println!("使用 WASD 或方向键移动，按 'q' 退出，按 'r' 重置\n");

        for row in &self.map {
            for tile in row {
                let ch = match tile {
                    Tile::Wall => "██",
                    Tile::Floor => "  ",
                    Tile::Target => "░░",
                    Tile::Box => "📦",
                    Tile::BoxOnTarget => "✅",
                    Tile::Player => "👤",
                    Tile::PlayerOnTarget => "👤",
                };
                print!("{}", ch);
            }
            println!();
        }
        println!("\n提示: 将所有箱子推到目标位置即可过关！");
        io::stdout().flush().unwrap();
    }

    // 检查是否胜利
    fn is_win(&self) -> bool {
        for row in &self.map {
            for tile in row {
                if *tile == Tile::Box {
                    return false; // 还有箱子不在目标上
                }
            }
        }
        true
    }

    // 移动玩家
    fn move_player(&mut self, dir: Direction) -> bool {
        let (dx, dy) = match dir {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };

        let (px, py) = self.player_pos;
        let nx = px as i32 + dx;
        let ny = py as i32 + dy;

        // 检查边界
        if ny < 0 || ny >= self.map.len() as i32 || nx < 0 || nx >= self.map[ny as usize].len() as i32 {
            return false;
        }

        let nx = nx as usize;
        let ny = ny as usize;

        // 检查目标位置是什么
        match self.map[ny][nx] {
            Tile::Wall => return false, // 撞墙
            Tile::Floor | Tile::Target => {
                // 可以移动
                let was_on_target = self.map[py][px] == Tile::PlayerOnTarget;
                self.map[py][px] = if was_on_target { Tile::Target } else { Tile::Floor };
                self.map[ny][nx] = if self.map[ny][nx] == Tile::Target {
                    Tile::PlayerOnTarget
                } else {
                    Tile::Player
                };
                self.player_pos = (nx, ny);
                self.moves += 1;
                return true;
            }
            Tile::Box | Tile::BoxOnTarget => {
                // 尝试推箱子
                let nnx = nx as i32 + dx;
                let nny = ny as i32 + dy;

                // 检查箱子移动后的位置
                if nny < 0 || nny >= self.map.len() as i32 || nnx < 0 || nnx >= self.map[nny as usize].len() as i32 {
                    return false;
                }

                let nnx = nnx as usize;
                let nny = nny as usize;

                match self.map[nny][nnx] {
                    Tile::Wall | Tile::Box | Tile::BoxOnTarget => return false, // 箱子后面有障碍
                    Tile::Floor | Tile::Target => {
                        // 可以推箱子
                        let was_on_target = self.map[py][px] == Tile::PlayerOnTarget;
                        let box_was_on_target = self.map[ny][nx] == Tile::BoxOnTarget;

                        // 更新玩家位置
                        self.map[py][px] = if was_on_target { Tile::Target } else { Tile::Floor };
                        self.map[ny][nx] = if box_was_on_target {
                            Tile::PlayerOnTarget
                        } else {
                            Tile::Player
                        };

                        // 更新箱子位置
                        self.map[nny][nnx] = if self.map[nny][nnx] == Tile::Target {
                            Tile::BoxOnTarget
                        } else {
                            Tile::Box
                        };

                        self.player_pos = (nx, ny);
                        self.moves += 1;
                        return true;
                    }
                    _ => return false,
                }
            }
            _ => return false,
        }
    }

    // 重置关卡
    fn reset(&mut self) {
        let (map, player_pos) = Self::load_level(self.level);
        self.map = map;
        self.player_pos = player_pos;
        self.moves = 0;
    }
}

fn main() {
    let mut current_level = 1;
    let max_level = 3;

    loop {
        let mut game = Game::new(current_level);
        game.render();

        loop {
            // 读取用户输入
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();

            if input.is_empty() {
                continue;
            }

            let moved = match input.chars().next().unwrap() {
                'w' | 'k' => game.move_player(Direction::Up),
                's' | 'j' => game.move_player(Direction::Down),
                'a' | 'h' => game.move_player(Direction::Left),
                'd' | 'l' => game.move_player(Direction::Right),
                'q' => {
                    println!("游戏退出！");
                    process::exit(0);
                }
                'r' => {
                    game.reset();
                    true
                }
                '\x1B' => {
                    // 处理方向键（ESC [ A/B/C/D）
                    if input.len() >= 3 {
                        match input.chars().nth(2).unwrap() {
                            'A' => game.move_player(Direction::Up),
                            'B' => game.move_player(Direction::Down),
                            'C' => game.move_player(Direction::Right),
                            'D' => game.move_player(Direction::Left),
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if moved {
                game.render();

                if game.is_win() {
                    println!("\n🎉 恭喜！你完成了关卡 {}！用了 {} 步。", current_level, game.moves);
                    println!("按 Enter 继续下一关，或按 'q' 退出...");

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();

                    if input.trim().to_lowercase() == "q" {
                        println!("游戏退出！");
                        process::exit(0);
                    }

                    current_level += 1;
                    if current_level > max_level {
                        println!("\n🎊 恭喜！你完成了所有关卡！");
                        process::exit(0);
                    }
                    break; // 进入下一关
                }
            }
        }
    }
}
