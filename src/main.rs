use asfalt_inator::AsfaltInator;
use cargo_commandos_lucky::lucky_function::lucky_spin;
use op_map::op_pathfinding;
use ragnarok::GuiRunner;
use rip_worldgenerator::MyWorldGen;
use robotics_lib::{
    energy::Energy,
    event::events::Event,
    interface::{go, look_at_sky, put, robot_map, robot_view, where_am_i, Direction, Tools},
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    utils::LibError,
    world::{
        coordinates::Coordinate,
        environmental_conditions::{EnvironmentalConditions, WeatherType},
        tile::{Content, Tile},
        World,
    },
};
use searchtool_unwrap::SearchTool;
use sense_and_find_by_rustafariani::Lssf;
use std::{borrow::{Borrow, BorrowMut}, collections::HashMap, process::exit};

struct MyRobot {
    robot: Robot,
    ticks: i32,
}

impl Runnable for MyRobot {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Terminated => {}
            _ => {}
        }
    }

    fn get_energy(&self) -> &Energy { &self.robot.energy }

    fn get_energy_mut(&mut self) -> &mut Energy { &mut self.robot.energy}

    fn get_coordinate(&self) -> &Coordinate { &self.robot.coordinate }

    fn get_coordinate_mut(&mut self) -> &mut Coordinate { &mut self.robot.coordinate }

    fn get_backpack(&self) -> &BackPack { &self.robot.backpack }

    fn get_backpack_mut(&mut self) -> &mut BackPack { &mut self.robot.backpack }

    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        println!("tick {:?}", self.get_energy().get_energy_level());
        let variables: Variables = Variables::new(
            self.robot.energy.get_energy_level(),
            self.robot.backpack.get_contents().clone(),
            robot_map(world).unwrap(),
            look_at_sky(world),
            self.ticks,
        );
        let actions = variables.interpreter();
        if actions.is_some() {
            for action in actions.unwrap() {
                match action {
                    Action::Discover => {
                        let mut lssf = Lssf::new();
                        let res: Result<Vec<Vec<((usize, usize), Tile, bool)>>, LibError> = lssf.sense_raw_centered_square(70, world, self, 4);
                    }
                    Action::AsfaltInator => {}
                    Action::Explore => {}
                    Action::GetResources => {}
                    Action::GoToMarket => {}
                    Action::TryEnergyReplenish => {}
                    Action::Wait => {}
                }
            }
        }
        println!("energy: {:?}", self.get_energy().get_energy_level());
    }
}

pub enum Action {
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
    fn interpreter(&self) -> Option<Vec<Action>> {
        let mut actions: Vec<Action> = Vec::new();
        if self.ticks == 0 {
            actions.push(Action::Discover);
        }
        if self.energy_lv < 50 {
            actions.push(Action::Wait);
        } else {
            actions.push(Action::Explore);
        }
        let res = Some(actions);
        return res;
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
/*To do
    Capire cosa implicano le condizioni atmosferiche

    Funzione interprete
        Capire come usare la mappa per bene
*/
/*pub fn cheapest(robot_view: Vec<Vec<Option<Tile>>>) -> Direction {
    let mut cheap: Direction = Direction::Up;
    let mut min: usize = 50;
    let mut dirs: Vec<&Option<Tile>> = Vec::new();
    dirs.push(robot_view[1][0].borrow());
    dirs.push(robot_view[2][1].borrow());
    dirs.push(robot_view[1][2].borrow());
    dirs.push(robot_view[0][1].borrow());
    for i in 0..3 {
        match dirs[i].borrow() {
            None => {}
            Some(x) => {
                if x.tile_type.properties().walk() {
                    let y = x.tile_type.properties().cost();
                    if y < min {
                        min = y.clone();
                        cheap = match i {
                            0 => Direction::Up,
                            1 => Direction::Right,
                            2 => Direction::Down,
                            3 => Direction::Left,
                            _ => Direction::Up,
                        }
                    }
                }
            }
        }
    }
    cheap
}*/
