use std::collections::VecDeque;
use std::collections::HashMap;
use std::rc::Rc;
use std::io;
use std::fmt;

use super::rubiks;

#[derive(Default)]
pub struct HeuristicsTables
{
    corners: Option<HashMap<rubiks::RubiksCubeState, u8>>,
}

impl HeuristicsTables
{
    pub fn new() -> Self
    {
        Self::default()
    }

    pub fn calc_corner_heuristics_table(&mut self)
    {

        let mut hash_table: HashMap<rubiks::RubiksCubeState, u8> = HashMap::with_capacity(18000000); // TODO: change size
        let mut num_pos = 0;

        let solv_state = rubiks::RubiksCubeState::std_solved_nxnxn(2);

        let mut vq: VecDeque<(rubiks::RubiksCubeState, u8)> = VecDeque::with_capacity(3000000);
        vq.push_back((solv_state, 0));

        while let Some((state, i)) = vq.pop_front()
        {
            if hash_table.contains_key(&state) { continue; }

            // Note, the bottom left cubie is the same for all states
            if i < 14
            {
                for turn_type in state.all_turns().into_iter()
                    .filter(|t| matches!(t.into_axis_based(), rubiks::Turn::AxisBased{index, ..} if index > 0)) // remove negative index turns
                {
                    let mut new_state = state.clone();
                    new_state.turn(turn_type);
                    if ! hash_table.contains_key(&new_state)
                    {
                        // already been found and in less turns
                        vq.push_back((new_state, i+1))
                    }
                }
            }

            hash_table.insert(state, i);
            num_pos += 1;
        }

        self.corners = Some(hash_table);
        assert_eq!(num_pos, 3674160);
    }

}

impl fmt::Debug for HeuristicsTables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeuristicsTables")
         .field("corners", &matches!(&self.corners, Some(_)))
         .finish()
    }
}

#[derive(Debug)]
pub enum RubikSolveError
{
    Unsolveable,
    BadInput,
    NoHeuristicsTable,
}

#[derive(Clone, Debug)]
pub struct RubiksCubeSolver
{
    //state: rubiks::RubiksCubeState,
    heuristic_table: Option<Rc<HeuristicsTables>>,
}

impl RubiksCubeSolver
{
    pub fn new() -> Self
    {
        RubiksCubeSolver{heuristic_table: None}
    }

    // #[allow(dead_code)]
    // pub fn from_state_string(s: &String) -> io::Result<Self>
    // {
    //     Ok(RubiksCubeSolver{state: rubiks::RubiksCubeState::from_state_string(s)?, heuristic_table: None})
    // }
    //
    // pub fn from_state(state: rubiks::RubiksCubeState) -> Self
    // {
    //     RubiksCubeSolver{state, heuristic_table: None}
    // }
    //
    // pub fn change_state(&mut self, new_state: rubiks::RubiksCubeState)
    // {
    //     self.state = new_state;
    // }
    //
    // #[allow(dead_code)]
    // pub fn rnd_scramble(n: usize, moves: usize) -> (Self, rubiks::Move)
    // {
    //     let (state, rubiks_move) = rubiks::RubiksCubeState::rnd_scramble(n, moves);
    //     return (RubiksCubeSolver{state, heuristic_table: None}, rubiks_move);
    // }
    //
    // #[allow(dead_code)]
    // pub fn from_corners_to_2x2x2(ref_state: &rubiks::RubiksCubeState, heuristic_table: Option<&Rc<HeuristicsTables>>) -> Self
    // {
    //     match heuristic_table 
    //     {
    //         None => RubiksCubeSolver{state: rubiks::RubiksCubeState::from_corners_to_2x2x2(ref_state), heuristic_table: None},
    //         Some(v) => RubiksCubeSolver{state: rubiks::RubiksCubeState::from_corners_to_2x2x2(ref_state), heuristic_table: Some(v.clone())}
    //     }
    // }

    // pub fn solver_dpll_2x2x2(&self, k: usize) -> (bool, Option<rubiks::Move>)
    // {
    //     if self.state.size() != 2 { return (false, None); }
    //
    //     if self.state.is_solved()
    //     {
    //         return (true, Some(rubiks::Move::empty()));
    //     }
    //     else if k <= 0
    //     {
    //         return (false, None);
    //     }
    //
    //     // if !valid
    //     // {
    //     //     return (false, None);
    //     // }
    //
    //     let mut state_history: Vec<Option<(rubiks::Move, rubiks::RubiksCubeState)>> = vec![None ; k+1];
    //     state_history[0] = Some((rubiks::Move::empty(), self.state.clone()));
    //     let mut possible_turns: Vec<(usize, rubiks::Turn)> = vec![];
    //
    //     for turn_type in self.state.all_turns().into_iter()
    //             .filter(|t| matches!(t.into_axis_based(), rubiks::Turn::AxisBased{index, ..} if index > 0)) // remove negative index turns
    //     {
    //         possible_turns.push((1, turn_type))
    //     }
    //
    //     while let Some((i, rubiks_turn)) = possible_turns.pop()
    //     {
    //         // do turn, add to history
    //         let mut mut_move = (&state_history[i-1]).as_ref().unwrap().0.clone();
    //         let mut mut_state = (&state_history[i-1]).as_ref().unwrap().1.clone();
    //         mut_state.turn(rubiks_turn);
    //         mut_move.turns.push(rubiks_turn);
    //         state_history[i] = Some((mut_move, mut_state));
    //
    //         if state_history[i].as_ref().unwrap().1.is_solved()
    //         {
    //             return (true, Some(state_history[i].as_ref().unwrap().0.clone()));
    //         }
    //
    //         if i >= k
    //         {
    //             // just made kth move and it was not solved
    //             continue;
    //         }
    //
    //         if self.state.size() > 2
    //         {
    //             if let None = self.calc_heuristics(&state_history[i].as_ref().unwrap().1, k-i) // !Self::from_corners_to_2x2x2(&state_history[i].as_ref().unwrap().1, (&self.heuristic_table).as_ref()).solver_dpll_2x2x2(k-i).0
    //             {
    //                 // our lower bound is to high
    //                 continue;
    //             }
    //         }
    //
    //         for turn_type in self.state.all_turns().into_iter()
    //             .filter(|t| matches!(t.into_axis_based(), rubiks::Turn::AxisBased{index, ..} if index > 0)) // remove negative index turns
    //         {
    //             if !state_history[i].as_ref().unwrap().0.is_next_turn_efficient(turn_type)
    //             {
    //                 continue;
    //             }
    //
    //             possible_turns.push((i + 1, turn_type));
    //         }
    //     }
    //
    //     return (false, None);
    // }

    // pub fn solve_dpll(&self, k: usize) -> (bool, Option<rubiks::Move>)
    // {
    //     // TODO: use lower bound by solving a relaxation in poly-time.
    //     // possible relaxations:
    //     // only look at diagonal pieces or just corners or something (i.e. 2x2x2).
    //     // there must be some way to 
    //
    //     if self.state.is_solved()
    //     {
    //         return (true, Some(rubiks::Move::empty()));
    //     }
    //     else if k <= 0
    //     {
    //         return (false, None);
    //     }
    //
    //     // if !valid
    //     // {
    //     //     return (false, None);
    //     // }
    //
    //     let mut state_history: Vec<Option<(rubiks::Move, rubiks::RubiksCubeState)>> = vec![None ; k+1];
    //     state_history[0] = Some((rubiks::Move::empty(), self.state.clone()));
    //     let mut possible_turns: Vec<(usize, rubiks::Turn)> = vec![];
    //
    //     for turn_type in self.state.all_turns()
    //     {
    //         possible_turns.push((1, turn_type))
    //     }
    //
    //     while let Some((i, rubiks_turn)) = possible_turns.pop()
    //     {
    //         // do turn, add to history
    //         let mut mut_move = (&state_history[i-1]).as_ref().unwrap().0.clone();
    //         let mut mut_state = (&state_history[i-1]).as_ref().unwrap().1.clone();
    //         mut_state.turn(rubiks_turn);
    //         mut_move.turns.push(rubiks_turn);
    //         state_history[i] = Some((mut_move, mut_state));
    //
    //         if state_history[i].as_ref().unwrap().1.is_solved()
    //         {
    //             return (true, Some(state_history[i].as_ref().unwrap().0.clone()));
    //         }
    //
    //         if i >= k
    //         {
    //             // just made kth move and it was not solved
    //             continue;
    //         }
    //
    //         if self.state.size() > 2 && k-i < 14 // note: every 2x2x2 cube can be solved in 14 moves or less
    //         {
    //             if let None = self.calc_heuristics(&state_history[i].as_ref().unwrap().1, k-i) // !Self::from_corners_to_2x2x2(&state_history[i].as_ref().unwrap().1, (&self.heuristic_table).as_ref()).solver_dpll_2x2x2(k-i).0
    //             {
    //                 // our lower bound is to high
    //                 continue;
    //             }
    //         }
    //
    //         for turn_type in self.state.all_turns()
    //         {
    //             if !state_history[i].as_ref().unwrap().0.is_next_turn_efficient(turn_type)
    //             {
    //                 continue;
    //             }
    //
    //             possible_turns.push((i + 1, turn_type));
    //         }
    //     }
    //
    //     return (false, None);
    // }

    pub fn calc_new_heuristics_table(&mut self)
    {
        let mut ht = HeuristicsTables::new();
        ht.calc_corner_heuristics_table();

        self.heuristic_table = Some(Rc::new(ht));
    }

    #[allow(dead_code)]
    pub fn add_heuristics_table(&mut self, heuristics_table: Rc<HeuristicsTables>)
    {
        if let None = self.heuristic_table
        {
            self.heuristic_table = Some(heuristics_table);
        }
    }

    // fn get_next_turn_order(&self, cube_state: &rubiks::RubiksCubeState, move_so_far: &rubiks::Move, k: usize) -> Vec<rubiks::Turn>
    // {
    //     let mut turns_with_heuristics: Vec<(rubiks::Turn, usize)> = cube_state.all_turns()
    //         .into_iter().filter(|t| move_so_far.is_next_turn_efficient(*t)).map(|t| 
    //     {
    //         let mut mut_state = cube_state.clone();
    //         mut_state.turn(t);
    //
    //         let heuristic = self.calc_heuristics(&mut_state, k-1);
    //
    //         (t, heuristic)
    //     }).filter(|(_, op)| matches!(op, Some(_))).map(|(t, op)| (t, op.unwrap())).collect();
    //
    //     turns_with_heuristics.sort_by(|(_, a), (_, b)| a.cmp(b));
    //     turns_with_heuristics.into_iter().map(|(t, _)| t).collect()
    // }

    // pub fn new_solve_dpll(&self, k: usize) -> (bool, Option<rubiks::Move>)
    // {
    //     if self.state.is_solved()
    //     {
    //         return (true, Some(rubiks::Move::empty()));
    //     }
    //     else if k <= 0
    //     {
    //         return (false, None);
    //     }
    //
    //     // if !valid
    //     // {
    //     //     return (false, None);
    //     // }
    //
    //     // used to order the branch we look at (best heuristic first)
    //
    //     let mut state_history: Vec<Option<(rubiks::Move, rubiks::RubiksCubeState)>> = vec![None ; k+1];
    //     state_history[0] = Some((rubiks::Move::empty(), self.state.clone()));
    //     let mut possible_turns: Vec<(usize, rubiks::Turn)> = vec![];
    //
    //     for turn_type in self.get_next_turn_order(&self.state, &rubiks::Move::empty(), k)
    //     {
    //         possible_turns.push((1, turn_type))
    //     }
    //
    //     while let Some((i, rubiks_turn)) = possible_turns.pop()
    //     {
    //         // do turn, add to history
    //         let mut mut_move = (&state_history[i-1]).as_ref().unwrap().0.clone();
    //         let mut mut_state = (&state_history[i-1]).as_ref().unwrap().1.clone();
    //         mut_state.turn(rubiks_turn);
    //         mut_move.turns.push(rubiks_turn);
    //         state_history[i] = Some((mut_move, mut_state));
    //
    //         if state_history[i].as_ref().unwrap().1.is_solved()
    //         {
    //             return (true, Some(state_history[i].as_ref().unwrap().0.clone()));
    //         }
    //
    //         if i >= k
    //         {
    //             // just made kth move and it was not solved
    //             continue;
    //         }
    //
    //         for turn_type in self.get_next_turn_order(&state_history[i].as_ref().unwrap().1, &state_history[i].as_ref().unwrap().0, k-i)
    //         {
    //             possible_turns.push((i + 1, turn_type));
    //         }
    //     }
    //
    //     return (false, None);
    // }

    // pub fn borrow_state(&'_ self) -> &'_ rubiks::RubiksCubeState
    // {
    //     &self.state
    // }

    pub fn solver_2x2x2_with_heuristics_table(&self, rubiks_state: &rubiks::RubiksCubeState) -> Result<rubiks::Move, RubikSolveError>
    {
        if rubiks_state.size() != 2 { return Err(RubikSolveError::BadInput); }

        if let Some(heuristic_table) = &self.heuristic_table
        {
            if let Some(ref corner_ht) = &heuristic_table.corners
            {
                let mut tmp_state = rubiks_state.clone();
                tmp_state.rotate_to_normal_2x2x2();
                if rubiks_state.is_solved()
                {
                    return Ok(rubiks::Move::empty());
                }
                else if let None = corner_ht.get(&tmp_state)
                {
                    return Err(RubikSolveError::Unsolveable);
                }

                let v = corner_ht.get(&tmp_state).map(|v| *v as usize).unwrap();

                let mut this_state = rubiks_state.clone();
                let mut this_move = rubiks::Move::empty();

                let mut v_left = v;
                for _ in 0..v
                {
                    let mut next_turn: Option<rubiks::Turn> = None;
                    for turn_type in rubiks_state.all_turns()
                    {
                        let mut tmp_state = this_state.clone();
                        tmp_state.turn(turn_type);
                        tmp_state.rotate_to_normal_2x2x2();
                        if let Some(new_v) = corner_ht.get(&tmp_state).map(|v| *v as usize)
                        {
                            if new_v < v_left 
                            {
                                next_turn = Some(turn_type);
                                v_left = new_v;
                                break;
                            }
                        }
                    }
                    if let Some(nt) = next_turn 
                    {
                        this_state.turn(nt);
                        this_move *= nt.as_move();
                    }
                    else
                    {
                        if this_state.is_solved()
                        {
                            break
                        }
                        else
                        {
                            todo!();
                            //return (false, None);
                        }
                    }
                }

                return Ok(this_move);
            }
            else
            {
                return Err(RubikSolveError::NoHeuristicsTable);
            }
        }
        else
        {
            return Err(RubikSolveError::NoHeuristicsTable);
        }
    }

    fn calc_corner_heuristics(&self, rubiks_state: &rubiks::RubiksCubeState) -> Option<usize>
    {
        // make it solve the 2x2x2 with dpll if not table exists
        if let Some(ref heuristic_table) = self.heuristic_table
        {
            if let Some(ref corner_ht) = &heuristic_table.corners
            {
                let mut cube_state2 = rubiks::RubiksCubeState::from_corners_to_2x2x2(rubiks_state);
                cube_state2.rotate_to_normal_2x2x2(); // this is for hashing // TODO: do better
                return corner_ht.get(&cube_state2).map(|v| *v as usize);
            }
        }

        return None;

        // todo!() //Self::from_corners_to_2x2x2(cube_state, (&self.heuristic_table).as_ref())
                //.solver_dpll_2x2x2(k).1.map(|m| m.turns.len())
    }

    /// will use heuristics if available
    pub fn solve_dpll(&self, rubiks_state: &rubiks::RubiksCubeState, k: usize) -> Result<rubiks::Move, RubikSolveError>
    {
        if rubiks_state.is_solved()
        {
            return Ok(rubiks::Move::empty());
        }
        else if k <= 0
        {
            return Err(RubikSolveError::Unsolveable);
        }
    
        // if !valid
        // {
        //     return (false, None);
        // }
    
        let mut state_history: Vec<Option<(rubiks::Move, rubiks::RubiksCubeState)>> = vec![None ; k+1];
        state_history[0] = Some((rubiks::Move::empty(), rubiks_state.clone()));
        let mut possible_turns: Vec<(usize, rubiks::Turn)> = vec![];
    
        for turn_type in rubiks_state.all_turns()
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
                return Ok(state_history[i].as_ref().unwrap().0.clone());
            }
    
            if i >= k
            {
                // just made kth move and it was not solved
                continue;
            }
    
            // TODO: update to use a general smaller cube, not just 2x2x2
            if rubiks_state.size() > 2 && k-i < 14 // note: every 2x2x2 cube can be solved in 14 moves or less
            {
                //if there are no heuristics, we can't do anything
                if let Some(h_val) = self.calc_corner_heuristics(&state_history[i].as_ref().unwrap().1)
                {
                    if h_val > k-1
                    {
                        // our lower bound is to high
                        continue;
                    }
                }
            }
    
            for turn_type in rubiks_state.all_turns()
            {
                if !state_history[i].as_ref().unwrap().0.is_next_turn_efficient(turn_type)
                {
                    continue;
                }
    
                possible_turns.push((i + 1, turn_type));
            }
        }
    
        return Err(RubikSolveError::Unsolveable);
    }

    #[allow(dead_code)]
    pub fn solve_with_ida_3x3x3(&self, rubiks_state: &rubiks::RubiksCubeState) -> Result<rubiks::Move, RubikSolveError>
    {
        // https://en.wikipedia.org/wiki/Iterative_deepening_A*
        // https://github.com/FarhanShoukat/Rubiks-Cube-Solver/blob/master/RubixcubeSolutionPatternDB.py#L78
        todo!()
    }

    #[allow(dead_code)]
    pub fn solve_with_ida(&self, rubiks_state: &rubiks::RubiksCubeState) -> Result<rubiks::Move, RubikSolveError>
    {
        // https://en.wikipedia.org/wiki/Iterative_deepening_A*
        // https://github.com/FarhanShoukat/Rubiks-Cube-Solver/blob/master/RubixcubeSolutionPatternDB.py#L78
        todo!()
    }

    #[allow(dead_code)]
    pub fn solve_best_approximation(&self, rubiks_state: &rubiks::RubiksCubeState) -> Result<rubiks::Move, RubikSolveError>
    {
        todo!()
    }
}

fn time_solves()
{
    
}

// #[test]
// fn test_calc_heuristics_table()
// {
//     assert!(false);
//     let (r_state, _scram_move) = rubiks::RubiksCubeState::rnd_scramble(2, 1000);
//
//     let mut solver = RubiksCubeSolver::from_state(r_state.clone());
//     solver.calc_heuristics_table();
//
//     //println!("moves away: {:?}", solver.calc_heuristics(&r_state, 14));
//     assert!(solver.calc_heuristics(&r_state, 14).unwrap() <= 14);
//
//     for _ in 0..100
//     {
//         let (r_state2, _scram_move) = rubiks::RubiksCubeState::rnd_scramble(2, 1000);
//         let num = solver.calc_heuristics(&r_state2, 14).unwrap();
//         //println!("moves away: {}", num);
//         assert!(num <= 14);
//
//         if num > 1
//         {
//             assert_eq!(solver.calc_heuristics(&r_state2, num-1), None)
//         }
//     }
// }
//
// #[test]
// fn encode_bit_strings()
// {
//     let n = 5;
//     let m = 3;
//     let s = 6*n+2*m;
//
//     let ls: Vec<[u8; 3]> = vec![[0, 1, 1], [1, 1, 0], [1, 1, 1], [1, 0, 0], [0, 0, 0]];
//
//     // let n = 3;
//     // let m = 2;
//     // let s = 6*n+2*m;
//
//     // let ls: Vec<[u8; 2]> = vec![[1, 1], [0, 1], [0, 0]];
//
//     let bs: Vec<rubiks::Move> = ls.clone().into_iter().enumerate().map(|(i,l)| 
//     {
//         let mut a_i = rubiks::Move::empty();
//         for (j, bit) in l.iter().enumerate()
//         {
//             if *bit != 0 
//             { 
//                 a_i *= rubiks::Turn::AxisBased{
//                     axis: rubiks::Axis::X, pos_rot: true, index: (j+1) as isize, cube_size: s}.as_move();
//             }
//         }
//         let z_m_i = rubiks::Turn::AxisBased{
//                     axis: rubiks::Axis::Z, pos_rot: true, index: (m+i+1) as isize, cube_size: s}.as_move();
//
//         a_i.clone() * z_m_i * a_i.invert()
//     }).collect();
//
//     let mut state = rubiks::RubiksCubeState::std_solved_nxnxn(s);
//
//     let mut a_1 = rubiks::Move::empty();
//     for (j, bit) in ls[0].iter().enumerate()
//     {
//         if *bit != 0 
//         { 
//             a_1 *= rubiks::Turn::AxisBased{
//                 axis: rubiks::Axis::X, pos_rot: true, index: (j+1) as isize, cube_size: s}.as_move();
//         }
//     }
//
//     let mut tb = rubiks::Move::empty();
//     let mut t = rubiks::Move::empty();
//
//     for bi in bs.clone().into_iter().rev() // rev doesn't matter, all bis commute
//     { 
//         //println!("{}", bi);
//         tb *= bi;
//     }
//
//     t = tb * a_1;
//
//     state.do_move(&t.clone());
//
//     println!("{}\n{:?}", t,state);
//
//     let soln = rubiks::Move{turns: vec![rubiks::Turn::AxisBased{axis: rubiks::Axis::Z, pos_rot: false, index:4, cube_size: s},
//                                         rubiks::Turn::AxisBased{axis: rubiks::Axis::X, pos_rot: true,  index:1, cube_size: s},
//                                         rubiks::Turn::AxisBased{axis: rubiks::Axis::Z, pos_rot: false, index:6, cube_size: s},
//                                         rubiks::Turn::AxisBased{axis: rubiks::Axis::X, pos_rot: false, index:3, cube_size: s},
//                                         rubiks::Turn::AxisBased{axis: rubiks::Axis::Z, pos_rot: false, index:5, cube_size: s},
//                                         rubiks::Turn::AxisBased{axis: rubiks::Axis::X, pos_rot: false, index:2, cube_size: s},
//                                         rubiks::Turn::AxisBased{axis: rubiks::Axis::Z, pos_rot: false, index:7, cube_size: s},
//                                         rubiks::Turn::AxisBased{axis: rubiks::Axis::X, pos_rot: false, index:1, cube_size: s},
//                                         rubiks::Turn::AxisBased{axis: rubiks::Axis::Z, pos_rot: false, index:8, cube_size: s}]};
//    
//     state.do_move(&soln);
//
//     println!("{}\n{:?}\nsolved: {}", soln, state, state.is_solved());
// }
//
// #[test]
// fn test_solve_2x2x2_with_heuristics_table()
// {
//     assert!(false);
//     let (r_state, _scram_move) = rubiks::RubiksCubeState::rnd_scramble(2, 1000);
//
//     let mut solver = RubiksCubeSolver::from_state(r_state.clone());
//     solver.calc_heuristics_table();
//
//     //println!("moves away: {:?}", solver.calc_heuristics(&r_state, 14));
//     assert!(solver.solver_2x2x2_heuristics_table(14).1.unwrap().turns.len() <= 14);
//
//     for _ in 0..100
//     {
//         let (mut r_state2, _scram_move) = rubiks::RubiksCubeState::rnd_scramble(2, 1000);
//         solver.change_state(r_state2.clone());
//         let (ret, soln) = solver.solver_2x2x2_heuristics_table(14);
//         assert_eq!(ret, true);
//         let num = soln.clone().unwrap().turns.len();
//         assert!(num <= 14);
//
//         r_state2.do_move(&soln.unwrap());
//         assert_eq!(r_state2.is_solved(), true);
//         //println!("moves away: {}", num);
//
//         if num > 1
//         {
//             assert_eq!(solver.solver_2x2x2_heuristics_table(num-1), (false, None));
//         }
//     }
// }
