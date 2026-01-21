use std::collections::{HashSet, VecDeque};
use crate::game::Tile;

/// 解题结果
pub struct SolveResult {
    pub solvable: bool,
    pub min_steps: Option<u32>,
}

/// 检查关卡是否可解，并返回最小步数（使用 BFS 搜索）
pub fn solve(map: &[Vec<Tile>], player_pos: (usize, usize)) -> SolveResult {
    // 提取初始状态：所有箱子位置和目标位置
    let mut initial_boxes: Vec<(usize, usize)> = Vec::new();
    let mut targets: HashSet<(usize, usize)> = HashSet::new();
    
    for (y, row) in map.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            match tile {
                Tile::Box => { initial_boxes.push((x, y)); }
                Tile::BoxOnTarget => { 
                    initial_boxes.push((x, y)); 
                    targets.insert((x, y));
                }
                Tile::Target | Tile::PlayerOnTarget => { targets.insert((x, y)); }
                _ => {}
            }
        }
    }
    
    initial_boxes.sort();
    
    // 状态：(玩家位置, 箱子位置列表, 步数)
    let initial_state = (player_pos, initial_boxes, 0u32);
    
    // BFS
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<((usize, usize), Vec<(usize, usize)>, u32)> = VecDeque::new();
    
    let state_key = get_state_key(&initial_state.0, &initial_state.1);
    visited.insert(state_key);
    queue.push_back(initial_state);
    
    let directions = [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)];
    
    while let Some((player_pos, boxes, steps)) = queue.pop_front() {
        // 检查是否所有箱子都在目标上
        let all_on_target = boxes.iter().all(|b| targets.contains(b));
        if all_on_target {
            return SolveResult {
                solvable: true,
                min_steps: Some(steps),
            };
        }
        
        let boxes_set: HashSet<(usize, usize)> = boxes.iter().cloned().collect();
        
        // 尝试四个方向
        for (dx, dy) in &directions {
            let nx = player_pos.0 as i32 + dx;
            let ny = player_pos.1 as i32 + dy;
            
            // 边界检查
            if ny < 0 || ny >= map.len() as i32 || 
               nx < 0 || nx >= map[ny as usize].len() as i32 {
                continue;
            }
            
            let nx = nx as usize;
            let ny = ny as usize;
            
            // 墙壁检查
            if map[ny][nx] == Tile::Wall {
                continue;
            }
            
            // 检查是否有箱子
            if boxes_set.contains(&(nx, ny)) {
                // 尝试推箱子
                let nnx = nx as i32 + dx;
                let nny = ny as i32 + dy;
                
                if nny < 0 || nny >= map.len() as i32 || 
                   nnx < 0 || nnx >= map[nny as usize].len() as i32 {
                    continue;
                }
                
                let nnx = nnx as usize;
                let nny = nny as usize;
                
                // 箱子目标位置必须是空的且不是墙
                if map[nny][nnx] == Tile::Wall || boxes_set.contains(&(nnx, nny)) {
                    continue;
                }
                
                // 简单死锁检测：角落检测
                if is_corner_deadlock(map, nnx, nny, &targets) {
                    continue;
                }
                
                // 创建新状态
                let mut new_boxes: Vec<(usize, usize)> = boxes.iter()
                    .filter(|&&b| b != (nx, ny))
                    .cloned()
                    .collect();
                new_boxes.push((nnx, nny));
                new_boxes.sort();
                
                let new_player = (nx, ny);
                let state_key = get_state_key(&new_player, &new_boxes);
                
                if !visited.contains(&state_key) {
                    visited.insert(state_key);
                    queue.push_back((new_player, new_boxes, steps + 1));
                }
            } else {
                // 玩家移动到空位
                let new_player = (nx, ny);
                let state_key = get_state_key(&new_player, &boxes);
                
                if !visited.contains(&state_key) {
                    visited.insert(state_key);
                    queue.push_back((new_player, boxes.clone(), steps + 1));
                }
            }
        }
    }
    
    SolveResult {
        solvable: false,
        min_steps: None,
    }
}

/// 生成状态唯一标识
fn get_state_key(player: &(usize, usize), boxes: &[(usize, usize)]) -> String {
    let boxes_str: Vec<String> = boxes.iter()
        .map(|(x, y)| format!("{},{}", x, y))
        .collect();
    format!("{}:{}|{}", player.0, player.1, boxes_str.join(";"))
}

/// 角落死锁检测
fn is_corner_deadlock(map: &[Vec<Tile>], x: usize, y: usize, targets: &HashSet<(usize, usize)>) -> bool {
    // 如果在目标位置，不算死锁
    if targets.contains(&(x, y)) {
        return false;
    }
    
    let height = map.len();
    let width = map[y].len();
    
    let up_wall = y == 0 || map[y - 1][x] == Tile::Wall;
    let down_wall = y >= height - 1 || map[y + 1][x] == Tile::Wall;
    let left_wall = x == 0 || map[y][x - 1] == Tile::Wall;
    let right_wall = x >= width - 1 || map[y][x + 1] == Tile::Wall;
    
    // 角落：两个相邻方向都是墙
    (up_wall && left_wall) || (up_wall && right_wall) || 
    (down_wall && left_wall) || (down_wall && right_wall)
}
