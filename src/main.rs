use std::time::Instant;

mod rubiks;

#[derive(Clone, Debug)]
struct RubiksCubeSolver
{
    state: rubiks::RubiksCubeState
}

impl RubiksCubeSolver
{
    #[allow(dead_code)]
    pub fn from_state_string(s: &String) -> Self
    {
        RubiksCubeSolver{state: rubiks::RubiksCubeState::from_state_string(s)}
    }

    pub fn from_state(state: rubiks::RubiksCubeState) -> Self
    {
        RubiksCubeSolver{state}
    }

    #[allow(dead_code)]
    pub fn rnd_scramble(n: usize, moves: usize) -> (Self, rubiks::Move)
    {
        let (state, rubiks_move) = rubiks::RubiksCubeState::rnd_scramble(n, moves);
        return (RubiksCubeSolver{state}, rubiks_move);
    }

    pub fn from_corners_to_2x2x2(ref_state: &rubiks::RubiksCubeState) -> Self
    {
        RubiksCubeSolver{state: rubiks::RubiksCubeState::from_corners_to_2x2x2(ref_state)}
    }

    pub fn solver_dpll_2x2x2(&self, k: usize) -> (bool, Option<rubiks::Move>)
    {
        assert_eq!(self.state.size(), 2);

        if self.state.is_solved()
        {
            return (true, Some(rubiks::Move{turns: vec![]}));
        }

        // if !valid
        // {
        //     return (false, None);
        // }

        let mut state_history: Vec<Option<(rubiks::Move, rubiks::RubiksCubeState)>> = vec![None ; k+1];
        state_history[0] = Some((rubiks::Move{turns: vec![]}, self.state.clone()));
        let mut possible_turns: Vec<(usize, rubiks::Turn)> = vec![];

        for turn_type in self.state.all_turns().into_iter()
                .filter(|t| matches!(t.into_axis_based(), rubiks::Turn::AxisBased{index, ..} if index > 0)) // remove negative index turns
        {
            possible_turns.push((1, turn_type))
        }

        while let Some((i, rubiks_turn)) = possible_turns.pop()
        {
            // do turn, add to history
            let mut mut_move = (&state_history[i-1]).as_ref().unwrap().0.clone();
            let mut mut_state = (&state_history[i-1]).as_ref().unwrap().1.clone();
            mut_state.turn(rubiks_turn);
            mut_move.turns.push(rubiks_turn);
            state_history[i] = Some((mut_move, mut_state));

            if state_history[i].as_ref().unwrap().1.is_solved()
            {
                return (true, Some(state_history[i].as_ref().unwrap().0.clone()));
            }

            if i >= k
            {
                // just made kth move and it was not solved
                continue;
            }

            if self.state.size() > 2
            {
                if !Self::from_corners_to_2x2x2(&state_history[i].as_ref().unwrap().1)
                    .solver_dpll_2x2x2(k-i).0
                {
                    // our lower bound is to high
                    continue;
                }
            }

            for turn_type in self.state.all_turns().into_iter()
                .filter(|t| matches!(t.into_axis_based(), rubiks::Turn::AxisBased{index, ..} if index > 0)) // remove negative index turns
            {
                if !state_history[i].as_ref().unwrap().0.is_next_turn_efficient(turn_type)
                {
                    continue;
                }

                possible_turns.push((i + 1, turn_type));
            }
        }

        return (false, None);
    }

    pub fn solve_dpll(&self, k: usize) -> (bool, Option<rubiks::Move>)
    {
        // TODO: use lower bound by solving a relaxation in poly-time.
        // possible relaxations:
        // only look at diagonal pieces or just corners or something (i.e. 2x2x2).
        // there must be some way to 

        if self.state.is_solved()
        {
            return (true, Some(rubiks::Move{turns: vec![]}));
        }

        // if !valid
        // {
        //     return (false, None);
        // }

        let mut state_history: Vec<Option<(rubiks::Move, rubiks::RubiksCubeState)>> = vec![None ; k+1];
        state_history[0] = Some((rubiks::Move{turns: vec![]}, self.state.clone()));
        let mut possible_turns: Vec<(usize, rubiks::Turn)> = vec![];

        for turn_type in self.state.all_turns()
        {
            possible_turns.push((1, turn_type))
        }

        while let Some((i, rubiks_turn)) = possible_turns.pop()
        {
            // do turn, add to history
            let mut mut_move = (&state_history[i-1]).as_ref().unwrap().0.clone();
            let mut mut_state = (&state_history[i-1]).as_ref().unwrap().1.clone();
            mut_state.turn(rubiks_turn);
            mut_move.turns.push(rubiks_turn);
            state_history[i] = Some((mut_move, mut_state));

            if state_history[i].as_ref().unwrap().1.is_solved()
            {
                return (true, Some(state_history[i].as_ref().unwrap().0.clone()));
            }

            if i >= k
            {
                // just made kth move and it was not solved
                continue;
            }

            if self.state.size() > 2 && k-i < 14 // note: every 2x2x2 cube can be solved in 14 moves or less
            {
                if !Self::from_corners_to_2x2x2(&state_history[i].as_ref().unwrap().1)
                    .solver_dpll_2x2x2(k-i).0
                {
                    // our lower bound is to high
                    continue;
                }
            }

            for turn_type in self.state.all_turns()
            {
                if !state_history[i].as_ref().unwrap().0.is_next_turn_efficient(turn_type)
                {
                    continue;
                }

                possible_turns.push((i + 1, turn_type));
            }
        }

        return (false, None);
    }

    #[allow(dead_code)]
    pub fn solve_best_approximation(&self) -> (bool, Option<rubiks::Move>)
    {
        todo!()
    }
}


fn main() 
{
    let (r_state, _turns) = rubiks::RubiksCubeState::rnd_scramble(2, 100);
    //println!("{}\n{:?}", turns, r_state);
    let solver = RubiksCubeSolver::from_state(r_state);
    let t0 = Instant::now();
    let res0 = solver.solver_dpll_2x2x2(14);
    println!("Found {:?} turn solution in {} secs.", res0.1.map(|l| l.turns.len()), t0.elapsed().as_secs_f64());

    // let solved_3x3_state = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    // let state = rubiks::RubiksCubeState::from_state_string(&solved_3x3_state);
    // println!("{:?}", state);
    
    // let solved_3x3_state_str = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    // let mut r_state = rubiks::RubiksCubeState::from_state_string(&solved_3x3_state_str);
    // r_state.turn(rubiks::Face::Left, true, 0);
    // r_state.turn(rubiks::Face::Up, false, 0);
    // r_state.turn(rubiks::Face::Down, false, 0);

    let (r_state, turns) = rubiks::RubiksCubeState::rnd_scramble(3, 15);
    println!("{}\n{:?}", turns, r_state);
    let solver = RubiksCubeSolver::from_state(r_state);

    let t0 = Instant::now();
    let res1 = solver.solve_dpll(7);
    println!("Found solution in {} secs.\n{:?}", t0.elapsed().as_secs_f64(), res1);
    if let (_, Some(r)) = res1
    {
        println!("{}", r);
    }

    let t0 = Instant::now();
    let res2 = solver.solve_dpll(10);
    println!("Found solution in {} secs.\n{:?}", t0.elapsed().as_secs_f64(), res2);
    if let (_, Some(r)) = res2
    {
        println!("{}", r);
    }
}
