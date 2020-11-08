mod rubiks;
mod solver;

use solver::RubiksCubeSolver;

use std::time::Instant;

use std::io;

fn solve_given()
{
    // ygrwbwgrgbbyowobooryygrw
    // wygywoogogobgggroyyywrrrggrbbrybbbrywborooobbrwywywwwg
    // wwrwwwwwogggggggggrrbrryrrrybybbbwbwbooooooooyybyyryyb
    // wwoyybrgggrrwoorrwwowgwwbbygwooyrrbyygwwybrwbryrybgobgyybrgbgbrobryyooybbwboggrgroyyrrwboybwboywwbwrogwrrggwgyooogroogyrywygwroooogbbwbwyybyrrgbbgwgog
    let mut solver = RubiksCubeSolver::from_state_string(&String::from("yworrygogbwrwbyoobyrggwb"));
    let t0 = Instant::now();
    solver.calc_heuristics_table();
    println!("Done calculating heuristics table in {} secs.", t0.elapsed().as_secs_f64());
    //let t0 = Instant::now();
    let res0 = solver.solver_2x2x2_heuristics_table(14);
    println!("Found {:?} turn solution: {}", res0.clone().1.map(|l| l.turns.len()), res0.1.unwrap());

    loop
    {
        println!("Input cube state:");

        let mut input = String::new();
        match io::stdin().read_line(&mut input)
        {
            Ok(_) => 
            {
                solver.change_state(rubiks::RubiksCubeState::from_state_string(&input.trim().to_owned()));
                println!("We got:\n{:?}", solver.borrow_state());
                if solver.borrow_state().size() == 2
                {
                    match solver.solver_2x2x2_heuristics_table(14)
                    {
                        (true, Some(the_move)) => println!("Solution: {}", the_move),
                        _ => println!("No Solution"),
                    }
                }
                else
                {
                    match solver.solve_dpll(15)
                    {
                        (true, Some(the_move)) => println!("Solution: {}", the_move),
                        _ => println!("No Solution"),
                    }
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}

fn main() 
{
    solve_given();
    // let (r_state, _turns) = rubiks::RubiksCubeState::rnd_scramble(2, 100);
    // //println!("{}\n{:?}", turns, r_state);
    // let mut solver = RubiksCubeSolver::from_state(r_state);
    // solver.calc_heuristics_table();
    // let t0 = Instant::now();
    // let res0 = solver.solver_dpll_2x2x2(14);
    // println!("Found {:?} turn solution in {} secs.", res0.1.map(|l| l.turns.len()), t0.elapsed().as_secs_f64());

    // let solved_3x3_state = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    // let state = rubiks::RubiksCubeState::from_state_string(&solved_3x3_state);
    // println!("{:?}", state);
    
    // let solved_3x3_state_str = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    // let mut r_state = rubiks::RubiksCubeState::from_state_string(&solved_3x3_state_str);
    // r_state.turn(rubiks::Face::Left, true, 0);
    // r_state.turn(rubiks::Face::Up, false, 0);
    // r_state.turn(rubiks::Face::Down, false, 0);

    let (r_state, turns) = rubiks::RubiksCubeState::rnd_scramble(3, 100);
    println!("{}\n{:?}", turns, r_state);
    let mut solver = RubiksCubeSolver::from_state(r_state);
    let mut t0 = Instant::now();
    solver.calc_heuristics_table();
    println!("Done calculating heuristics table in {} secs.", t0.elapsed().as_secs_f64());

    t0 = Instant::now();
    let res1 = solver.solve_dpll(15);
    println!("Found solution in {} secs.\n{:?}", t0.elapsed().as_secs_f64(), res1);
    t0 = Instant::now();
    let res12 = solver.new_solve_dpll(15);
    println!("Found solution in {} secs.\n{:?}", t0.elapsed().as_secs_f64(), res12);
    if let (_, Some(r)) = res1
    {
        println!("{}", r);
    }

    t0 = Instant::now();
    let res2 = solver.solve_dpll(20);
    println!("Found solution in {} secs.\n{:?}", t0.elapsed().as_secs_f64(), res2);
    t0 = Instant::now();
    let res22 = solver.new_solve_dpll(20);
    println!("Found solution in {} secs.\n{:?}", t0.elapsed().as_secs_f64(), res22);
    if let (_, Some(r)) = res2
    {
        println!("{}", r);
    }
}