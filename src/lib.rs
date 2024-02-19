use asfalt_inator::AsfaltInator;
use cargo_commandos_lucky::lucky_function::lucky_spin;
use olympus::channel::Channel;
//use op_map::op_pathfinding::*;
use rip_worldgenerator::MyWorldGen;
use robotics_lib::{
    energy::Energy,
    event::events::Event,
    interface::{
        discover_tiles, go, look_at_sky, one_direction_view, put, robot_map, robot_view, teleport,
        where_am_i, Direction, Tools,
    },
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    utils::{calculate_cost_go_with_environment, LibError},
    world::{
        coordinates::Coordinate,
        environmental_conditions::{self, EnvironmentalConditions, WeatherType},
        tile::{Content, Tile, TileType},
        World,
    },
};
//use searchtool_unwrap::{SearchDirection, SearchTool};
use sense_and_find_by_rustafariani::*;
use std::{
    borrow::Borrow,
    cell::RefCell,
    clone,
    cmp::{max, min},
    collections::HashMap,
    process::exit,
    rc::Rc,
};

pub struct MyRobot {
    pub robot: Robot,
    pub ticks: i32,
    channel: Rc<RefCell<Channel>>,
}

impl Runnable for MyRobot {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Terminated => {}
            Event::TimeChanged(weather) => {
                self.channel.borrow_mut().send_weather_info(weather);
            }
            _ => {}
        }
    }

    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }

    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }

    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }

    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }

    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }

    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }

    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        self.channel.borrow_mut().send_game_info(self, world);

        println!("tick {:?}", self.get_energy().get_energy_level());
        let variables: Variables = Variables::new(
            self.robot.energy.get_energy_level(),
            self.robot.backpack.get_contents().clone(),
            robot_map(world).unwrap(),
            look_at_sky(world),
            self.ticks,
        );
        robot_view(self, world);
        let complex_actions = variables.interpreter();
        for action in complex_actions {
            match action {
                ComplexAction::Discover => {
                    let mut lssf = Lssf::new();
                    let res: Result<Vec<Vec<((usize, usize), Tile, bool)>>, LibError> =
                        lssf.sense_raw_centered_square(41, world, self, 2);
                    if res.is_err() {
                        println!("{:?}", res.err())
                    }
                }
                ComplexAction::AsfaltInator => {}
                ComplexAction::Explore => {
                    let l = 21;
                    let granularity = 4;
                    let c = self.get_tile_to_move_towards(world, l, granularity);
                    let mut flag = false;
                    let mut coords = (
                        self.robot.coordinate.get_row(),
                        self.robot.coordinate.get_col(),
                    );
                    for dir in c.iter() {
                        print!("{:?} ", dir);
                        match dir {
                            Direction::Up => coords = (coords.0 - 1, coords.1),
                            Direction::Right => coords = (coords.0, coords.1 + 1),
                            Direction::Down => coords = (coords.0 + 1, coords.1),
                            Direction::Left => coords = (coords.0, coords.1 - 1),
                        }
                        if flag {
                            let res = discover_tiles(self, world, &[(coords.0, coords.1)]);
                            if res.is_err() {
                                println!("{:?}", res.err());
                            }
                        } else {
                            let res = go(self, world, dir.clone());
                            if res.is_err() {
                                flag = true;
                                println!("{:?}", res.err());
                            }
                            let view = robot_view(self, world);
                            for row in view {
                                for col in row {
                                    print!("{:?}", col);
                                }
                                println!();
                            }
                        }
                    }
                }
                ComplexAction::GetResources => {}
                ComplexAction::GoToMarket => {}
                ComplexAction::TryEnergyReplenish => {}
                ComplexAction::Wait => {
                    let mut robot = Robot::new();
                    self.robot.energy = robot.energy;
                }
            }
        }
        self.ticks += 1;
        println!("energy: {:?}", self.get_energy().get_energy_level());
    }
}

impl MyRobot {
    pub fn new(channel: Rc<RefCell<Channel>>) -> Self {
        Self {
            robot: Robot::new(),
            ticks: 0,
            channel,
        }
    }
    pub fn get_tile_to_move_towards(
        &mut self,
        world: &mut World,
        l: usize,
        granularity: usize,
    ) -> Vec<Direction> {
        let mut lssf = Lssf::new();
        let map: Result<Vec<Vec<((usize, usize), Tile, bool)>>, LibError> =
            lssf.sense_raw_centered_square(l, world, self, granularity);
        let mut matrix_likeability: Vec<Vec<(i32, Vec<Direction>)>> =
            vec![vec![(0, vec![]); map.as_ref().unwrap().len()]; map.as_ref().unwrap().len()];
        let mut matrix_visited: Vec<Vec<bool>> =
            vec![vec![false; map.as_ref().unwrap().len()]; map.as_ref().unwrap().len()];
        let coords = (
            (map.as_ref().unwrap().len() / 2),
            (map.as_ref().unwrap().len() / 2),
        );
        path_finder(
            coords,
            None,
            &map.unwrap(),
            &mut matrix_likeability,
            &mut matrix_visited,
            &mut vec![],
            0,
            look_at_sky(world),
            1,
        );
        //matrix_likeability[coords.0][coords.1] = (0, matrix_likeability[coords.0][coords.1].1.clone());
        let mut path: Vec<Direction> = vec![];
        let mut max: i32 = 0;
        for row in matrix_likeability {
            for col in row {
                if col.0 > max {
                    path = col.1.clone();
                    max = col.0;
                }
                print!("{:?} ", col.0);
            }
            println!();
        }
        println!("{:?}", max);
        path
    }
}
pub fn path_finder(
    curr: (usize, usize),
    prev: Option<(usize, usize)>,
    map: &Vec<Vec<((usize, usize), Tile, bool)>>,
    matrix_likeability: &mut Vec<Vec<(i32, Vec<Direction>)>>,
    matrix_visited: &mut Vec<Vec<bool>>,
    path: &mut Vec<Direction>,
    cost: usize,
    environmental_conditions: EnvironmentalConditions,
    uncovered_tiles: usize,
) {
    matrix_visited[curr.0][curr.1] = true;
    let dirs = vec![
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ];
    if map[curr.0][curr.1].1.tile_type.properties().walk() {
        for dir in dirs {
            let mut base_cost;
            let next: Option<(usize, usize)> = match dir {
                Direction::Up => {
                    if curr.0 as i32 - 1 < 0
                        || matrix_visited[curr.0 - 1][curr.1]
                        || !map[curr.0 - 1][curr.1].1.tile_type.properties().walk()
                    {
                        None
                    } else {
                        Some((curr.0 - 1, curr.1))
                    }
                }
                Direction::Right => {
                    if curr.1 + 1 > map.len() - 1
                        || matrix_visited[curr.0][curr.1 + 1]
                        || !map[curr.0][curr.1 + 1].1.tile_type.properties().walk()
                    {
                        None
                    } else {
                        Some((curr.0, curr.1 + 1))
                    }
                }
                Direction::Down => {
                    if curr.0 + 1 > map.len() - 1
                        || matrix_visited[curr.0 + 1][curr.1]
                        || !map[curr.0 + 1][curr.1].1.tile_type.properties().walk()
                    {
                        None
                    } else {
                        Some((curr.0 + 1, curr.1))
                    }
                }
                Direction::Left => {
                    if curr.1 as i32 - 1 < 0
                        || matrix_visited[curr.0][curr.1 - 1]
                        || !map[curr.0][curr.1 - 1].1.tile_type.properties().walk()
                    {
                        None
                    } else {
                        Some((curr.0, curr.1 - 1))
                    }
                }
            };
            if prev.is_some() {
                base_cost = map[curr.0][curr.1].1.tile_type.properties().cost();
                base_cost = calculate_cost_go_with_environment(
                    base_cost,
                    environmental_conditions.clone(),
                    map[curr.0][curr.1].1.tile_type,
                );
                let mut elevation_cost = 0;
                if map[curr.0][curr.1].1.elevation
                    > map[prev.unwrap().0][prev.unwrap().1].1.elevation
                {
                    elevation_cost = (map[curr.0][curr.1].1.elevation
                        - map[prev.unwrap().0][prev.unwrap().1].1.elevation)
                        .pow(2);
                }
                let cost = cost + base_cost;
                if cost < 200 {
                    let mut u_tile = uncovered_tiles;
                    if map[curr.0][curr.1].2 == false {
                        u_tile += 10;
                    }
                    let mut distance_row = curr.0 as i32 - (map.len() as i32 / 2);
                    if distance_row < 0 {
                        distance_row = distance_row * -1
                    }
                    let mut distance_col = curr.1 as i32 - (map.len() as i32 / 2);
                    if distance_col < 0 {
                        distance_col = distance_col * -1
                    }
                    let distance = distance_row + distance_col;
                    let likeability = u_tile as i32 * distance;
                    if likeability >= matrix_likeability[curr.0][curr.1].0 {
                        matrix_likeability[curr.0][curr.1] = (likeability, path.clone());
                        if next.is_some()
                            && map[next.unwrap().0][next.unwrap().1]
                                .1
                                .tile_type
                                .properties()
                                .walk()
                        {
                            path.push(dir);
                            path_finder(
                                next.unwrap(),
                                Some(curr),
                                map,
                                matrix_likeability,
                                matrix_visited,
                                path,
                                cost + 5,
                                environmental_conditions.clone(),
                                u_tile,
                            );
                            path.pop();
                        }
                    }
                }
            } else {
                if next.is_some()
                    && map[next.unwrap().0][next.unwrap().1]
                        .1
                        .tile_type
                        .properties()
                        .walk()
                {
                    path.push(dir);
                    path_finder(
                        next.unwrap(),
                        Some(curr),
                        map,
                        matrix_likeability,
                        matrix_visited,
                        path,
                        cost + 5,
                        environmental_conditions.clone(),
                        uncovered_tiles,
                    );
                    path.pop();
                }
            }
        }
    }
    matrix_visited[curr.0][curr.1] = false;
}

pub enum ComplexAction {
    Discover,
    Explore,
    GetResources,
    GoToMarket,
    AsfaltInator,
    TryEnergyReplenish,
    Wait,
}
pub struct Variables {
    energy_lv: usize,
    inventory: HashMap<Content, usize>,
    map: Vec<Vec<Option<Tile>>>,
    e: EnvironmentalConditions,
    ticks: i32,
    city1: Option<(usize, usize)>,
    city2: Option<(usize, usize)>,
}

impl Variables {
    fn new(
        energy_lv: usize,
        inventory: HashMap<Content, usize>,
        map: Vec<Vec<Option<Tile>>>,
        e: EnvironmentalConditions,
        ticks: i32,
    ) -> Self {
        Self {
            energy_lv,
            inventory,
            map,
            e,
            ticks,
            city1: None,
            city2: None,
        }
    }
    fn update() {}
    fn interpreter(&self) -> Vec<ComplexAction> {
        let mut action: Vec<ComplexAction> = Vec::new();
        let mut flag = true;
        let mut cycles = 0;
        while flag {
            if self.ticks == 0 && cycles == 0 {
                action.push(ComplexAction::Discover);
                flag = false;
                println!("Discover");
            } else if self.energy_lv < 500 {
                action.push(ComplexAction::Wait);
                flag = false;
                println!("Wait");
            } else {
                action.push(ComplexAction::Explore);
                flag = false;
                println!("Explore");
            }
            cycles += 1;
        }
        action
    }
}

/*
println!("prova smart");
                    let res = lssf.smart_sensing_centered(40, world, self, 2);
                    if res.is_err() {
                        println!("{:?}", res.err());
                    }
                    println!("prova update");
                    lssf.update_map(robot_map(world).unwrap().borrow());
                    let c = self.get_coordinate();
                    println!("{:?}", c);
                    let res = lssf.update_cost_constrained(c.get_row(), c.get_col(), 100, 500);
                    lssf.update_map(robot_map(world).unwrap().borrow());
                    println!("prova get content");
                    let c = lssf.get_content_vec(&Content::Tree(1));
                    if c.is_empty() {
                        println!("empty c");
                    }
                    let n = (self.ticks % 3) as usize;
                    let (xc, yc) = c[n].clone();
                    println!("prova get action");
                    let res = lssf.get_action_vec(xc, yc);
                    println!("test ok");
                    if res.is_ok() {
                        if res.clone().unwrap().is_empty() {
                            println!("empty res");
                        }
                        for act in res.unwrap() {
                            robot_view(self, world);
                            match act {
                                Action::North => {
                                    let _ = go(self, world, Direction::Up);
                                }
                                Action::South => {
                                    let _ = go(self, world, Direction::Down);
                                }
                                Action::West => {
                                    let _ = go(self, world, Direction::Left);
                                }
                                Action::East => {
                                    let _ = go(self, world, Direction::Right);
                                }
                                Action::Teleport(i, j) => {
                                    let _ = teleport(self, world, (i, j));
                                }
                            }
                        }
                    }

fn main() {
    let mut generator = MyWorldGen::new();
    let mut robot = MyRobot {
        robot: Robot::new(),
        ticks: 0,
    };

    let gui_runner = GuiRunner::new(Box::new(robot), &mut generator).unwrap();

    gui_runner.run().unwrap();
}

ComplexAction::Explore => {
    let mut list = ShoppingList::new(vec![(Content::Rock(1), Some(OpComplexActionInput::Destroy()))]);
    let res = get_best_ComplexAction_to_element(self, world, &mut list);
    if res.is_some() {
        match res.unwrap() {
            OpComplexActionOutput::Move(dir) => {
                println!("{:?}", dir);
                let res = go(self, world, dir);
            }
            OpComplexActionOutput::Destroy(_) => {}
            OpComplexActionOutput::Put(_, _, _) => {}
        }
    }
    else {
        println!("res is none");
    }
}
let content = match self.ticks % 12 {
                        0 => Content::Tree(0),
                        1 => Content::Rock(0),
                        2 => Content::Crate(0..30),
                        3 => Content::Coin(0),
                        4 => Content::Bank(0..30),
                        5 => Content::Building,
                        6 => Content::Garbage(0),
                        7 => Content::Fire,
                        8 => Content::Bin(0..30),
                        9 => Content::JollyBlock(0),
                        10 => Content::Market(0),
                        11 => Content::Fish(0),
                        12 => Content::Bush(0),
                        _ => Content::Rock(0),
                    };
*/
