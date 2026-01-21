use std::io::{self, Write, BufRead, BufReader};
use std::fs::File;
use std::path::Path;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

// 地图元素
#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
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
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// 游戏状态
pub struct Game {
    pub level: usize,
    pub map: Vec<Vec<Tile>>,
    pub player_pos: (usize, usize),
    pub moves: u32,
}

impl Game {
    pub fn new(level: usize) -> Self {
        let (map, player_pos) = Self::load_level(level);
        Game {
            level,
            map,
            player_pos,
            moves: 0,
        }
    }

    // 从文件加载关卡
    pub fn load_level(level: usize) -> (Vec<Vec<Tile>>, (usize, usize)) {
        let level_path = format!("levels/level_{}.txt", level);
        let path = Path::new(&level_path);

        // 如果文件不存在，尝试加载关卡1
        if !path.exists() {
            if level != 1 {
                return Self::load_level(1);
            } else {
                panic!("无法找到关卡文件: {}", level_path);
            }
        }

        let file = File::open(path).expect(&format!("无法打开关卡文件: {}", level_path));
        let reader = BufReader::new(file);

        let mut map = Vec::new();
        let mut player_pos = (0, 0);

        for (y, line) in reader.lines().enumerate() {
            let line = line.expect("读取关卡文件行失败");
            let mut map_row = Vec::new();
            for (x, ch) in line.chars().enumerate() {
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
    pub fn render(&self) {
        execute!(
            io::stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0)
        ).unwrap();
        
        print!("推箱子游戏 - 关卡 {} | 步数: {}\r\n", self.level, self.moves);
        print!("使用 WASD 或方向键移动，按 'q' 退出，按 'r' 重置\r\n");
        print!("\r\n");

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
            print!("\r\n");
        }
        print!("\r\n");
        print!("提示: 将所有箱子推到目标位置即可过关！\r\n");
        io::stdout().flush().unwrap();
    }

    // 检查是否胜利
    pub fn is_win(&self) -> bool {
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
    pub fn move_player(&mut self, dir: Direction) -> bool {
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
    pub fn reset(&mut self) {
        let (map, player_pos) = Self::load_level(self.level);
        self.map = map;
        self.player_pos = player_pos;
        self.moves = 0;
    }
}
