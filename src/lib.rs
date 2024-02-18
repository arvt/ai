use asfalt_inator::AsfaltInator;
use cargo_commandos_lucky::lucky_function::lucky_spin;
use olympus::channel::Channel;
//use op_map::op_pathfinding::*;
use rip_worldgenerator::MyWorldGen;
use robotics_lib::{
    energy::Energy,
    event::events::Event,
    interface::{
        go, look_at_sky, one_direction_view, put, robot_map, robot_view, teleport, where_am_i,
        Direction, Tools,
    },
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    utils::LibError,
    world::{
        coordinates::Coordinate,
        environmental_conditions::{EnvironmentalConditions, WeatherType},
        tile::{Content, Tile, TileType},
        World,
    },
};
//use searchtool_unwrap::{SearchDirection, SearchTool};
use sense_and_find_by_rustafariani::*;
use std::{borrow::Borrow, cell::RefCell, cmp::{max, min}, collections::HashMap, process::exit, rc::Rc};

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
                    println!("disco");
                    let mut lssf = Lssf::new();
                    let res: Result<Vec<Vec<((usize, usize), Tile, bool)>>, LibError> =
                        lssf.sense_raw_centered_square(40, world, self, 2);
                    if res.is_err() { println!("{:?}", res.err())}
                }
                ComplexAction::AsfaltInator => {}
                ComplexAction::Explore => {
                    let l = 20;
                    let granularity = 2;
                    let c = self.get_direction_to_move_towards(world, l, 4);
                    for _ in 0..c.1 {
                        let res = go(self, world, c.clone().0);
                        if res.is_err() {

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
    pub fn get_direction_to_move_towards(&mut self, world: &mut World, l: usize, granularity: usize) -> (Direction, usize) {
        let mut lssf = Lssf::new();
        let robot_coord = (self.robot.coordinate.get_row(), self.robot.coordinate.get_col());
        let top_left = (max(robot_coord.0 - 20, 0), max(robot_coord.1 - 20, 0));
        let top_right = (max(robot_coord.0, 0), max(robot_coord.1 - 20, 0));
        let bottom_right = (max(robot_coord.0 , 0), max(robot_coord.1, 0));
        let bottom_left = (max(robot_coord.0 - 20, 0), max(robot_coord.1, 0));
        let map_top_left = lssf.sense_raw_square_by_corner(l, world, self, granularity, top_left).unwrap();
        let map_top_right = lssf.sense_raw_square_by_corner(l, world, self, granularity, top_right).unwrap();
        let map_bottom_right = lssf.sense_raw_square_by_corner(l, world, self, granularity, bottom_right).unwrap();
        let map_bottom_left = lssf.sense_raw_square_by_corner(l, world, self, granularity, bottom_left).unwrap();
        let likeability_top_left = calculate_likeability(map_top_left);
        let likeability_top_right = calculate_likeability(map_top_right);
        let likeability_bottom_right = calculate_likeability(map_bottom_right);
        let likeability_bottom_left = calculate_likeability(map_bottom_left);
        let likeability_top = likeability_top_left + likeability_top_right;
        let likeability_right = likeability_top_right + likeability_bottom_right;
        let likeability_bottom = likeability_bottom_right + likeability_bottom_left;
        let likeability_left = likeability_bottom_left + likeability_top_left;
        let mut max = likeability_top;
        for like in vec![likeability_top, likeability_right, likeability_bottom, likeability_left] {
            if like > max { max = like}
        }
        match max {
            likeability_top => ( Direction::Up, robot_coord.1 - top_right.1 ),
            likeability_right => ( Direction::Right,  top_right.0 - robot_coord.0),
            likeability_bottom => ( Direction::Down,  bottom_left.1 - robot_coord.1),
            likeability_left => ( Direction::Left, robot_coord.0 - bottom_left.0 ),
        }
    }
}

pub fn calculate_likeability(map: Vec<Vec<((usize, usize), Tile, bool)>>) -> i32 {
    let mut likeability: i32 = 0;
    for row in map.clone() {
        for col in row {
            match col.2 {
                false => likeability += 1,
                _ => {}
            }
            match col.1.tile_type {
                TileType::DeepWater => likeability -= 10,
                _ => {}
            }
        }
    }
    likeability * 100 / (map.len() * map.len()) as i32
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
