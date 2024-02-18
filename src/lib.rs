use asfalt_inator::AsfaltInator;
use cargo_commandos_lucky::lucky_function::lucky_spin;
//use op_map::op_pathfinding::
use ragnarok::GuiRunner;
use rip_worldgenerator::MyWorldGen;
use robotics_lib::{
    energy::Energy,
    event::events::Event,
    interface::{go, look_at_sky, one_direction_view, put, robot_map, robot_view, teleport, where_am_i, Direction, Tools},
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    utils::LibError,
    world::{
        coordinates::Coordinate,
        environmental_conditions::{EnvironmentalConditions, WeatherType},
        tile::{Content, Tile},
        World,
    },
};
//use searchtool_unwrap::{SearchDirection, SearchTool};
use sense_and_find_by_rustafariani::*;
use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    process::exit,
};

pub struct MyRobot {
    pub robot: Robot,
    pub ticks: i32,
}

impl Runnable for MyRobot {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Terminated => {}
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
        println!("tick {:?}", self.get_energy().get_energy_level());
        let variables: Variables = Variables::new(
            self.robot.energy.get_energy_level(),
            self.robot.backpack.get_contents().clone(),
            robot_map(world).unwrap(),
            look_at_sky(world),
            self.ticks,
        );
        robot_view(self, world);
        let ComplexActions = variables.interpreter();
        for ComplexAction in ComplexActions {
            match ComplexAction {
                ComplexAction::Discover => {
                    let mut lssf = Lssf::new();
                    let res: Result<Vec<Vec<((usize, usize), Tile, bool)>>, LibError> =
                        lssf.sense_raw_centered_square(40, world, self, 0);
                }
                ComplexAction::AsfaltInator => {}
                ComplexAction::Explore => {
                    let mut lssf = Lssf::new();
                    let x = lssf.sense_raw_centered_square(40, world, self, 2).unwrap();
                    let mut c = (0, 0);
                    for row in x {
                        for col in row {
                            match col.1.content {
                                Content::Tree(_) => { c = col.0; }
                                _ => {}
                            }
                        }
                    }
                    if c != (0, 0) {
                        if self.robot.coordinate.get_row() > c.0 {
                            one_direction_view(self, world, Direction::Left, self.robot.coordinate.get_row() - c.0);
                        } else if self.robot.coordinate.get_row() < c.0 {
                            one_direction_view(self, world, Direction::Right, c.0 - self.robot.coordinate.get_row());
                        } 
                        if self.robot.coordinate.get_col() > c.1 {
                            one_direction_view(self, world, Direction::Down, self.robot.coordinate.get_col() - c.1);
                        } else if self.robot.coordinate.get_col() < c.1 {
                            one_direction_view(self, world, Direction::Down, c.1 - self.robot.coordinate.get_col());
                        }
                        
                    }
                    println!("prova smart");
                    let res = lssf.smart_sensing_centered(40, world, self, 2);
                    if res.is_err() { println!("{:?}", res.err());}
                    println!("prova update");
                    lssf.update_map(robot_map(world).unwrap().borrow());
                    let c = self.get_coordinate();
                    println!("{:?}", c);
                    let res = lssf.update_cost_constrained(c.get_row(), c.get_col(), 100 , 500);
                    lssf.update_map(robot_map(world).unwrap().borrow());
                    println!("prova get content");
                    let c = lssf.get_content_vec(&Content::Tree(1));
                    if c.is_empty() { println!("empty c"); }
                    let n = (self.ticks % 3) as usize;
                    let (xc, yc) = c[n].clone();
                    println!("prova get action");
                    let res = lssf.get_action_vec(xc, yc);
                    println!("test ok");
                    if res.is_ok() {
                        if res.clone().unwrap().is_empty() { println!("empty res");}
                        for act in res.unwrap() {
                            robot_view(self, world);
                            match act {
                                Action::North => { let _ = go(self, world, Direction::Up); }
                                Action::South => { let _ = go(self, world, Direction::Down); }
                                Action::West => { let _ = go(self, world, Direction::Left); }
                                Action::East => { let _ = go(self, world, Direction::Right); }
                                Action::Teleport(i, j) => { let _ = teleport(self, world, (i, j)); }
                            }
                        }
                    }
                }
                ComplexAction::GetResources => {}
                ComplexAction::GoToMarket => {}
                ComplexAction::TryEnergyReplenish => {}
                ComplexAction::Wait => {
                    let mut robot = Robot::new();
                    self.robot.energy = robot.energy;}
            }
        }
        self.ticks += 1;
        println!("energy: {:?}", self.get_energy().get_energy_level());
    }
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
