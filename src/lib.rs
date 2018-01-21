#![feature(ascii_ctype)]
use std::fmt;
use std::str::FromStr;
use std::ascii::AsciiExt;
use std::io::Write;
extern crate regex;
use regex::Regex;

mod maps;

/// 4 points of the compass
#[derive(Debug, Copy, Clone, PartialEq)]
enum Dir{
    N,E,W,S
}
impl FromStr for Dir {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s{
            "N" => Ok(Dir::N),
            "E" => Ok(Dir::E),
            "W" => Ok(Dir::W),
            "S" => Ok(Dir::S),
            other => Err(format!("Cannot parse direction '{}' - only N,E,W,S are valid values", other))
        }
    }
}


/// Map location in screen coordinates (positive Y axis is down)
#[derive(Debug, Copy, Clone)]
struct Location{
    pub x: i32,
    pub y: i32
}
impl Location{
    fn offset(&self, dir: Dir, distance: i32) -> Location{
        Location{
            x: self.x + if dir == Dir::E { -distance } else if dir == Dir::W { distance } else { 0 },
            y: self.y + if dir == Dir::N { -distance } else if dir == Dir::S { distance } else { 0 },
        }
    }
    fn next(&self, dir: Dir) -> Location{
        self.offset(dir, 1)
    }
}
impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}


/// 4 values corresponding to the 4 directions
#[derive(Debug, Copy, Clone)]
struct Nearby(pub [bool;4]);

impl Nearby{
    fn get(&self, dir: Dir) -> bool{
        match dir{
            Dir::N => self.0[0],
            Dir::E => self.0[1],
            Dir::W => self.0[2],
            Dir::S => self.0[3],
        }
    }
}
impl fmt::Display for Nearby {
    /// Print NEWS for 1,1,1,1 and NExx for 1,1,0,0 etc.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (letter, value) in "NEWS".chars().zip(self.0.iter()){
            if *value {
                write!(f, "{}", letter)?;
            }else{
                write!(f, "x")?;
            }
        }
        Ok(())
    }
}

/// Represents a map with 1 bit per cell.
/// Tracks the number of true and false values
struct BoolMap{
    values: Vec<bool>,
    width: i32,
    height: i32,
    false_count: u32,
    true_count: u32,
}

impl BoolMap{
    fn in_bounds(&self, loc: Location) -> bool{
        loc.x >= 0 || loc.y >= 0 || loc.x < self.width  || loc.y < self.height
    }
    fn index(&self, loc: Location) -> usize{
        (loc.y * self.width + loc.x) as usize
    }
    /// Returns the coordinates for the `nth` location on the map where
    /// the value matches `where_matches`
    fn get_nth_location(&self, nth: usize, where_matches: bool) -> Option<Location>{
        let mut n = 0;
        for (ix, v) in self.values.iter().enumerate(){
            if *v == where_matches{
                if n == nth {
                    return Some(Location{ x: (ix % self.width as usize) as i32, y: (ix / self.width as usize) as i32 })
                }
                n += 1;
            }
        }
        None
    }
    fn get(&self, loc: Location) -> bool{
        if !self.in_bounds(loc){
            panic!("Out of bounds map access ({})", loc);
            //true
        }else {
            self.values[self.index(loc)]
        }
    }
    fn nearby(&self, loc: Location) -> Nearby {
        Nearby([self.get(loc.next(Dir::N)),
            self.get(loc.next(Dir::E)),
            self.get(loc.next(Dir::W)),
            self.get(loc.next(Dir::S))])
    }

    fn set(&mut self, loc: Location, value: bool){
        if self.get(loc) != value{
            let index = self.index(loc);
            self.values[index] = value;
            if value{
                self.true_count+=1;
                self.false_count-=1;
            }else{
                self.true_count -1;
                self.false_count+=1;
            }
        }
    }
    fn true_count(&self) -> u32{
        self.true_count
    }
    fn false_count(&self) -> u32{
        self.false_count
    }

    fn width(&self) -> i32{
        self.width
    }
    fn height(&self) -> i32{
        self.height
    }

    fn load(map: &'static [[u8; 25]; 25]) -> BoolMap{
        let mut values = Vec::with_capacity(25 * 25);
        let mut true_count = 0;
        for row in map.iter(){
            for cell in row.iter(){
                values.push(*cell != 0);
                if *cell != 0{
                    true_count+=1;
                }
            }
        }
        BoolMap{
            false_count: (values.len() - true_count) as u32,
            true_count: true_count as u32,
            values,
            width: 25,
            height: 25,
        }
    }
    fn clear(width: i32, height: i32, values: bool) -> BoolMap{
        if width < 3 || height < 3{
            panic!("Cannot create a {}x{} map", width, height);
        }
        let total = (width * height) as u32;
        BoolMap{
            false_count: if values { 0 } else {total},
            true_count: if values { total } else {0},
            values: vec![values; total as usize],
            width,
            height
        }
    }

}

/// Represents a combined wall map, visitation map, and picobot state
struct MapState{
    walls: BoolMap,
    visited: BoolMap,
    bot: Location,
    bot_state: u32
}

impl MapState{

    fn unvisited(&self) -> i64{
        self.walls.false_count() as i64 - self.visited.true_count() as i64
    }

    fn nearby_walls(&self) -> Nearby{
        self.walls.nearby(self.bot)
    }
    fn move_to(&mut self, dest: Location) {
        if self.walls.get(dest){
            panic!("Cannot move into wall")
        }
        self.visited.set(dest, true);
        self.bot = dest;
    }
    fn try_move_bot(&mut self, dir: Dir) -> bool{
        if !self.nearby_walls().get(dir){
            let dest = self.bot.next(dir);
            self.move_to( dest);
            true
        }else{
            false
        }
    }
    fn create(from_map: &'static [[u8; 25]; 25], start_index: usize) -> Option<MapState>{
        let walls = BoolMap::load(from_map);
        let visited = BoolMap::clear(walls.width,walls.height, false);

        let start_at = walls.get_nth_location(start_index, false);
        if let Some(start_location) = start_at {
            let mut state = MapState {
                walls,
                visited,
                bot: Location { x: 0, y: 0 },
                bot_state: 0
            };
            state.move_to(start_location);
            Some(state)
        }else{
            None
        }
    }

    fn print(&self) {
        let  stdout = std::io::stdout();
        let mut lock = stdout.lock();

        for y in 0..self.walls.height() {
            for x in 0..self.walls.width() {

                let c = if self.bot.x == x && self.bot.y == y{
                    "@"
                } else {
                    if self.walls.get(Location{x,y}){
                        "#"
                    }else if self.visited.get( Location{x,y}){
                        "-"
                    } else{
                        " "
                    }
                };

                write!(lock, "{}", c).unwrap();

            }
            write!(lock, "\n").unwrap();
        }
        writeln!(lock, "State: {}  Nearby: {}  Remaining: {}\n", self.bot_state, self.nearby_walls(), self.unvisited()).unwrap();
    }
}

#[derive(Copy, Clone)]
enum SpaceCondition{
    Clear,
    Wall,
    Any
}
impl FromStr for SpaceCondition {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s{
            "N" | "E" | "W" | "S" => Ok(SpaceCondition::Wall),
            "x" => Ok(SpaceCondition::Clear),
            "*" => Ok(SpaceCondition::Any),
            other => Err(format!("Cannot parse '{}' - only N,E,W,S,x,* are valid values", other))
        }
    }
}

#[derive(Copy, Clone)]
pub struct Rule{
    match_state: u32,
    match_nearby: [SpaceCondition; 4],
    go: Dir,
    state: u32
}
impl Rule{
    fn matches(&self, current_state: u32, nearby: Nearby) -> bool{
        if self.match_state == current_state {
            nearby.0.iter().zip(self.match_nearby.iter()).all(|(wall, condition)| {
                match *condition{
                    SpaceCondition::Any => true,
                    SpaceCondition::Wall => *wall,
                    SpaceCondition::Clear => !*wall
                }
            })
        }else{
            false
        }
    }

    pub fn parse_all(text: &str) -> Result<Vec<Rule>, String>{
        let results: Vec<Result<Option<Rule>, String>> = text.lines().map(|line| Rule::parse(line)).collect();

        let mut errors = String::new();
        for err in results.iter().filter(|r| r.is_err()){
            if let &Err(ref e) = err{
                errors.push_str(&e);
                errors.push_str("\n");
            }
        }
        if errors.len() > 0{
            Err(errors)
        }else{
            Ok(results.into_iter().map(|r| r.unwrap()).filter(|r| r.is_some()).map(|r| r.unwrap()).collect())
        }

    }

    fn parse(line: &str) -> Result<Option<Rule>, String>{
        if line.is_ascii_whitespace(){
            Ok(None)
        } else {
            let comment = Regex::new("\\s*#.*").unwrap();
            if comment.is_match(line) {
                Ok(None)
            } else {
                let rule = Regex::new("\\s*([0-9]+) +([Nx*])([Ex*])([Wx*])([Sx*]) +-> +([NEWS]) +([0-9]+)").unwrap();
                if let Some(caps) = rule.captures(line){
                    let match_state = caps.get(1).unwrap().as_str().parse::<u32>().unwrap();
                    let state = caps.get(7).unwrap().as_str().parse::<u32>().unwrap();
                    let go = Dir::from_str(caps.get(6).unwrap().as_str()).unwrap();
                    let match_nearby = [
                        SpaceCondition::from_str(caps.get(2).unwrap().as_str()).unwrap(),
                        SpaceCondition::from_str(caps.get(3).unwrap().as_str()).unwrap(),
                        SpaceCondition::from_str(caps.get(4).unwrap().as_str()).unwrap(),
                        SpaceCondition::from_str(caps.get(5).unwrap().as_str()).unwrap()];
                    Ok(Some(Rule{
                        match_nearby,
                        match_state,
                        state,
                        go
                    }))
                }else{
                    Err(format!("Cannot parse rule {}", line))
                }
            }
        }
    }


}
impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", self.match_state)?;
        for (letter, condition) in "NEWS".chars().zip(self.match_nearby.iter()){
            match *condition{
                SpaceCondition::Any => {write!(f, "*")?;},
                SpaceCondition::Clear => {write!(f, "x")?;},
                SpaceCondition::Wall => {write!(f, "{}", letter)?;},
            }
        }
        write!(f, "  -> {:?} {}", self.go, self.state)

    }
}

pub struct Game{
    map: MapState,
    rules: Vec<Rule>
}
impl Game{
    fn matching_rule(&self) -> Option<Rule>{
        let nearby = self.map.nearby_walls();
        let current_state = self.map.bot_state;
        // self.rules.iter().find(|r| r.matches(current_state, nearby))

        let mut matches = self.rules.iter().filter(|r| r.matches(current_state, nearby));

        let first = matches.next();
        if let Some(other) = matches.next(){
            panic!("More than one rule applies:\n{}\n{}\n", first.unwrap(), other);
        }
        first.map(|r| *r)
    }


    pub fn step(&mut self){
        if let Some(rule) = self.matching_rule(){
            self.map.bot_state = rule.state;
            if !self.map.try_move_bot(rule.go){
                panic!("Cannot move {:?} (wall encountered)", rule.go);
            }
            // check if complete
        }else{
            panic!("No rule exists for state {} and surroundings {}", self.map.bot_state, self.map.nearby_walls());
        }
    }

    fn is_complete(&self) -> bool{
        self.map.unvisited() <= 0
    }

    pub fn play_to_end(&mut self, limit_moves: usize) -> bool{
        self.map.print();
        let mut move_count = 0;
        loop{
            if self.is_complete(){
                self.map.print();
                return true;
            }
            if move_count > limit_moves{
                self.map.print();
                return false;
            }
            self.step();
            move_count+=1;
        }

    }

    pub fn create(from_map: &'static [[u8; 25]; 25], starting_position: usize, rules: Vec<Rule>) -> Option<Game>{
        MapState::create(from_map, starting_position).map( |map| {
            Game {
                rules,
                map
            }
        })
    }

}

pub struct RuleSetTester{
    rules: Vec<Rule>,
    map: &'static [[u8; 25]; 25],
    turn_limit: usize

}

impl RuleSetTester {
    pub fn create(map: &'static [[u8; 25]; 25], rules: &str) -> RuleSetTester {
        let rules = Rule::parse_all(rules).expect("Rules must be valid");
        RuleSetTester {
            rules,
            map,
            turn_limit: 1000000
        }
    }
    pub fn test_all(&mut self){
        let mut start_index = 0;
        loop {
            let game_maybe = Game::create(self.map, start_index, self.rules.clone());

            if let Some(mut game) = game_maybe {
                assert_eq!(game.play_to_end(self.turn_limit), true);
            }else{
                assert!(start_index > 0);
                return;
            }
            start_index += 1;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_starting_positions_work() {
        RuleSetTester::create(maps::DIAMOND_MAP,
                              "

").test_all();

    }
}


