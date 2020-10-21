mod rubix;

#[derive(Clone, Debug)]
struct RubixCubeSolver
{
    state: rubix::RubixCubeState
}

impl RubixCubeSolver
{
    #[allow(dead_code)]
    pub fn from_state_string(s: &String) -> Self
    {
        Self{state: rubix::RubixCubeState::from_state_string(s)}
    }

    pub fn from_state(state: rubix::RubixCubeState) -> Self
    {
        Self{state}
    }

    #[allow(dead_code)]
    pub fn rnd_scramble(n: usize, moves: usize) -> (Self, rubix::Move)
    {
        let (state, rubix_move) = rubix::RubixCubeState::rnd_scramble(n, moves);
        return (Self{state}, rubix_move);
    }

    pub fn solve_dpll(&self, k: usize) -> (bool, Option<rubix::Move>)
    {
        if self.state.is_solved()
        {
            return (true, Some(rubix::Move{turns: vec![]}));
        }

        // if !valid
        // {
        //     return (false, None);
        // }

        let mut state_history: Vec<Option<(rubix::Move, rubix::RubixCubeState)>> = vec![None ; k+1];
        state_history[0] = Some((rubix::Move{turns: vec![]}, self.state.clone()));
        let mut possible_turns: Vec<(usize, rubix::Turn)> = vec![];

        for turn_type in self.state.all_turns()
        {
            possible_turns.push((1, turn_type))
        }

        while let Some((i, rubix_turn)) = possible_turns.pop()
        {
            // do turn, add to history
            let mut mut_move = (&state_history[i-1]).as_ref().unwrap().0.clone();
            let mut mut_state = (&state_history[i-1]).as_ref().unwrap().1.clone();
            mut_state.turn(rubix_turn);
            mut_move.turns.push(rubix_turn);
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
}


fn main() 
{
    // let solved_3x3_state = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    // let state = rubix::RubixCubeState::from_state_string(&solved_3x3_state);
    // println!("{:?}", state);
    
    // let solved_3x3_state_str = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    // let mut r_state = rubix::RubixCubeState::from_state_string(&solved_3x3_state_str);
    // r_state.turn(rubix::Face::Left, true, 0);
    // r_state.turn(rubix::Face::Up, false, 0);
    // r_state.turn(rubix::Face::Down, false, 0);

    let (r_state, turns) = rubix::RubixCubeState::rnd_scramble(3, 2);
    println!("{}\n{:?}", turns, r_state);
    let solver = RubixCubeSolver::from_state(r_state);


    let res1 = solver.solve_dpll(5);
    println!("{:?}", res1);
    if let (_, Some(r)) = res1
    {
        println!("{}", r);
    }

    let res2 = solver.solve_dpll(7);
    println!("{:?}", res2);
    if let (_, Some(r)) = res2
    {
        println!("{}", r);
    }
}
