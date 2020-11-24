//! Rubik's cube simulator.
//! 
//! The module contains [`RubiksCubeState`] which stores the state of a nxnxn rubik's cube.
//! You can then construct and apply moves to be done on the cube.
//! 
//! # Examples
//! ```rust
//! use rubiks::*;
//! let mut state = RubiksCubeState::std_solved_nxnxn(3);
//! 
//! let u_inv_t = Turn::FaceBased{face: Face::Up, inv: true, num_in:0, cube_size: 3};
//! let f_inv_t = Turn::FaceBased{face: Face::Front, inv: true, num_in:0, cube_size: 3};
//! let l_inv_t = Turn::FaceBased{face: Face::Left, inv: true, num_in:0, cube_size: 3};
//! 
//! let three_turn_move = u_inv_t.as_move() * f_inv_t.as_move() * l_inv_t.as_move();
//! 
//! state.do_move(&three_turn_move);
//! 
//! println!("{:?}", state);
//! ``` 
//! Gives us.
//! ```
//!     GWW
//!     GWW
//!     GBB
//! WWW ORR YRR BBR
//! OGG YRR YBB OOW
//! OGG YRR YBB OOW
//!     OGG
//!     OYY
//!     BYY
//! ```
//! [`RubiksCubeState`]: struct.RubiksCubeState.html

use core::hash::{Hash, Hasher};
#[allow(unused_imports)]
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::ops;
use rand;
use rand::prelude::*;
use std::io;//::{Error, ErrorKind, Result};

/// ULFRBD face
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face
{
    Up,
    Left,
    Front,
    Right,
    Back,
    Down
}

impl Face
{
    /// Converts to the capital of the first letter as `char`.
    pub fn as_char(&self) -> char
    {
        match self
        {
            Self::Up => 'U',
            Self::Left => 'L',
            Self::Front => 'F',
            Self::Right => 'R',
            Self::Back => 'B',
            Self::Down => 'D'
        }
    }
}

/// XYZ axis
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis
{
    X,
    Y,
    Z,
}

/// WGRBOY color
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Color
{
    White,
    Green,
    Red,
    Blue,
    Orange,
    Yellow
}

impl Color
{
    /// Converts to the capital of the first letter as `char`.
    pub fn as_char(&self) -> char
    {
        match self
        {
            Self::White => 'W',
            Self::Green => 'G',
            Self::Red => 'R',
            Self::Blue => 'B',
            Self::Orange => 'O',
            Self::Yellow => 'Y'
        }
    }
}

/// Single Slice Quarter Turn
/// 
/// Mappings between the to types:
/// - Up = +Z
/// - Left = +X
/// - Front = +Y
/// - Right = -X
/// - Back = -Y
/// - Down = -Z
/// 
/// num_in = cube_size/2 - index
/// 
#[derive(Clone, Copy, Eq, Debug)]
pub enum Turn
{
    /// A turn with the axis. `index` is the layer away from the center where positive index is in the positive direction.
    /// If there is an even `cube_size` then we pretend that there is still a center index 0 layer that doesn't show up. 
    /// the direction we rotate is according to the right hand rule such that if the normal vector is in the positive direction then we say `pos_rot = true`.
    AxisBased
    {
        axis: Axis,
        pos_rot: bool,
        index: isize,
        cube_size: usize
    },

    /// A normal, `inv = false`, turn is clockwise relative to the face, inverted is counter clockwise.
    /// `num_in` is how many layers in we turn. `num_in = 0` is the outer most face. `num_in = 1` is the layer right behind that and so on.
    /// Note, you can not turn the middle layer or layers closer to the other side.
    FaceBased
    {
        face: Face,
        inv: bool,
        num_in: usize,
        cube_size: usize
    }
}

impl Default for Turn
{
    fn default() -> Self {
            Turn::FaceBased {
            face: Face::Up,
            inv: false,
            num_in: 0,
            cube_size: 3
        }
    }
}

impl PartialEq for Turn
{
    fn eq(&self, other: &Turn) -> bool
    {
        match *self
        {
            Turn::AxisBased{axis: axis1, pos_rot: pos_rot1, index: index1, cube_size: cube_size1} => 
            {
                if let Turn::AxisBased{axis: axis2, pos_rot: pos_rot2, index: index2, cube_size: cube_size2} = other.into_axis_based()
                {
                    return axis1 == axis2 && pos_rot1 == pos_rot2 && index1 == index2 && cube_size1 == cube_size2;
                }
                else
                {
                    unreachable!();
                }
            },
            Turn::FaceBased{face: face1, inv: inv1, num_in: num_in1, cube_size: cube_size1} => 
            {
                if let Turn::FaceBased{face: face2, inv: inv2, num_in: num_in2, cube_size: cube_size2} = other.into_face_based()
                {
                    return face1 == face2 && inv1 == inv2 && num_in1 == num_in2 && cube_size1 == cube_size2;
                }
                else
                {
                    unreachable!();
                }
            }
        }
    }
}

impl Turn
{
    /// Converts to `Turn::FaceBased` enum variant.
    pub fn into_face_based(self) -> Self
    {
        match self
        {
            Turn::AxisBased{axis: Axis::X, pos_rot, index, cube_size} if index > 0 => Turn::FaceBased{face: Face::Left, inv: pos_rot, num_in: cube_size/2 - index as usize, cube_size},
            Turn::AxisBased{axis: Axis::X, pos_rot, index, cube_size} => Turn::FaceBased{face: Face::Right, inv: !pos_rot, num_in: cube_size/2 - (-index) as usize, cube_size},
            Turn::AxisBased{axis: Axis::Y, pos_rot, index, cube_size} if index > 0 => Turn::FaceBased{face: Face::Front, inv: pos_rot, num_in: cube_size/2 - index as usize, cube_size},
            Turn::AxisBased{axis: Axis::Y, pos_rot, index, cube_size} => Turn::FaceBased{face: Face::Back, inv: !pos_rot, num_in: cube_size/2 - (-index) as usize, cube_size},
            Turn::AxisBased{axis: Axis::Z, pos_rot, index, cube_size} if index > 0 => Turn::FaceBased{face: Face::Up, inv: pos_rot, num_in: cube_size/2 - index as usize, cube_size},
            Turn::AxisBased{axis: Axis::Z, pos_rot, index, cube_size} => Turn::FaceBased{face: Face::Down, inv: !pos_rot, num_in: cube_size/2 - ((-index) as usize), cube_size},
            
            t @ Turn::FaceBased{..} => t
        }
    }
    
    /// Converts to `Turn::AxisBased` enum variant.
    pub fn into_axis_based(self) -> Self
    {
        match self
        {
            Turn::FaceBased{face: Face::Up, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::Z, pos_rot: inv, index: cube_size as isize/2 - num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Left, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::X, pos_rot: inv, index: cube_size as isize/2 - num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Front, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::Y, pos_rot: inv, index: cube_size as isize/2 - num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Right, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::X, pos_rot: !inv, index: - (cube_size as isize)/2 + num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Back, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::Y, pos_rot: !inv, index: - (cube_size as isize)/2 + num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Down, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::Z, pos_rot: !inv, index: - (cube_size as isize)/2 + num_in as isize, cube_size},

            t @ Turn::AxisBased{..} => t
        }
    }

    /// Changes the size of the cube to `new_cube_size`. This is needed because turns hold the size of the cube they are for.
    /// The `index`/`num_in` of the turn is re-calculated relative to the center of the cube (so `index` remains the same).
    /// Well return `Err(())` if any turn can't exist for a cube with the new cube size.
    #[allow(dead_code)]
    pub fn change_cube_size_hold_center(self, new_cube_size: usize) -> Result<Self, ()>
    {
        if let Turn::AxisBased{axis, pos_rot, index, ..} = self.into_axis_based()
        {
            if index.abs() as usize > new_cube_size/2
            {
                Err(())
            }
            else
            {
                Ok(Turn::AxisBased{axis, pos_rot, index, cube_size: new_cube_size})
            }
        }
        else
        {
            unreachable!()
        }
    }

    /// Changes the size of the cube to `new_cube_size`. This is needed because turns hold the size of the cube they are for.
    /// The `index`/`num_in` of the turn is re-calculated relative to the faces (so `num_in` remains the same).
    /// Well return `Err(())` if any turn can't exist for a cube with the new cube size.
    #[allow(dead_code)]
    pub fn change_cube_size_hold_face(self, new_cube_size: usize) -> Result<Self, ()>
    {
        if let Turn::FaceBased{face, inv, num_in, ..} = self.into_face_based()
        {
            if num_in >= new_cube_size/2
            {
                Err(())
            }
            else
            {
                Ok(Turn::FaceBased{face, inv, num_in, cube_size: new_cube_size})
            }
        }
        else
        {
            unreachable!()
        }
    }

    /// inverts the turn
    pub fn invert(self) -> Self
    {
        match self 
        {
            Turn::AxisBased{axis, pos_rot, index, cube_size} => Turn::AxisBased{axis, pos_rot: !pos_rot, index, cube_size},
            Turn::FaceBased{face, inv, num_in, cube_size} => Turn::FaceBased{face, inv: !inv, num_in, cube_size}
        }
    }

    /// Checks if two turns commute with each other. If they are on the same axis then they commute, otherwise they don't.
    pub fn commutes_with(&self, other: &Turn) -> bool
    {
        if let Turn::AxisBased{axis: s_axis, ..} = self.into_axis_based()
        {
            if let Turn::AxisBased{axis: o_axis, ..} = other.into_axis_based()
            {
                // same face
                if s_axis == o_axis
                {
                    true
                }
                else
                {
                    // never commutes
                    false
                }
            }
            else {unreachable!()}
        }
        else {unreachable!()}
    }

    /// Creates a move with just the one turn.
    pub fn as_move(self) -> Move
    {
        Move{turns: vec![self]}
    }
}

/// A list of turns
#[derive(Debug, Clone)]
pub struct Move
{
    pub turns: Vec<Turn>
}

impl Move 
{
    // todo: assert that all turns have same cube size

    /// Will invert the move such that `M.invert() * M == M * M.invert()` is an identity.
    #[allow(dead_code)]
    pub fn invert(self) -> Self
    {
        Move{turns: self.turns.into_iter().rev().map(|turn| turn.invert()).collect()}
    }

    /// Will append moves.
    /// Use `*` operator: `M1 * M2`.
    pub fn append(&mut self, other: &mut Self)
    {
        self.turns.append(&mut other.turns);
    }

    /// Will create a random move for an nxnxn rubik's cube with `num_turns` turns.
    pub fn rnd_move(n: usize, num_turns: usize) -> Self
    {
        let mut rng = rand::thread_rng();

        let mut turns = vec![];

        for _ in 0..num_turns
        {
            let face = match rng.gen_range(0, 6)
            {
                0 => Face::Up,
                1 => Face::Left,
                2 => Face::Front,
                3 => Face::Right,
                4 => Face::Back,
                _ => Face::Down
            };
            let inv = rng.gen();
            let num_in = rng.gen_range(0,n/2);
            turns.push(Turn::FaceBased{face, inv, num_in, cube_size: n});
        }
        return Move{turns};
    }

    /// We check to see if adding the next turn makes the move inefficient. 
    /// The turn can make the move inefficient in 3 ways:
    /// - The turn is the inverse of the last turn in the current move.
    /// - The turn is the 3rd of the same type of move in a row.
    /// - The turn commutes with the last move and it is not in the order U->D (larger index turns first) L->R F->B.
    /// 
    /// These are an attempt to make each branch on the dpll algorithm lead to a different cube configuration.
    pub fn is_next_turn_efficient(&self, next_turn: Turn) -> bool
    {
        if let Some(last_turn) = self.turns.last()
        {
            if last_turn.invert() == next_turn
            {
                // We don't want to make the inv of prev turn
                return false;
            }

            if self.turns.len() > 1
            {
                let last_last_turn = self.turns[self.turns.len() - 2];
                if last_last_turn == *last_turn && *last_turn == next_turn
                {
                    // 3 of the same turn in a row is not optimal
                    return false;
                }
            }

            // Now we check for commuting moves
            // We want moves to be in the order U->D L->R F->B, if two commuting moves are next to each other
            if let Turn::AxisBased{axis: nt_axis, index: nt_index, ..} = next_turn.into_axis_based()
            {
                if let Turn::AxisBased{axis: lt_axis, index: lt_index, ..} = last_turn.into_axis_based()
                {
                    if next_turn.commutes_with(&last_turn)
                    {
                        // if commute and are in good order
                        return match lt_axis
                        {
                            Axis::Z => { nt_axis != Axis::Z || nt_index <= lt_index },
                            Axis::Y => { nt_axis != Axis::Y || nt_index <= lt_index },
                            Axis::X => { nt_axis != Axis::X || nt_index <= lt_index },
                        };
                    }
                }
                else {unreachable!()}
            }
            else {unreachable!()}

            return true;
        }
        else
        {
            // and move is "efficient" appending to identity 
            return true;
        }
    }

    /// Changes the size of the cube to `new_cube_size` for each [`Turn`]. This is needed because [`Turn`]s hold the size of the cube they are for.
    /// The `index`/`num_in` of the [`Turn`] is re-calculated relative to the center of the cube (so `index` remains the same) for the each turn in the move.
    /// Any turn that can't exist for a cube with the new cube size will be removed from the move.
    /// 
    /// [`Turn`]: enum.Turn.html
    #[allow(dead_code)]
    pub fn change_cube_size_hold_center(self, new_cube_size: usize) -> Self
    {
        
        // let mut turns: Vec<Turn> = vec![Turn::default(); self.turns.len()];
        
        // for (i, turn) in self.turns.into_iter().enumerate()
        // {
        //     turns[i] = turn.change_cube_size_hold_center(new_cube_size)?;
        // }
        // Will return `Err(())` if any turn can't exist for a cube with the new cube size.
        //Ok(Move{turns})

        Move{turns: self.turns.into_iter()
            .map(|t| t.change_cube_size_hold_center(new_cube_size))
            .filter(|t| matches!(t, Ok(_))).map(|t| t.unwrap()).collect()}

    }
    
    /// Changes the size of the cube to `new_cube_size` for each [`Turn`]. This is needed because [`Turn`]s hold the size of the cube they are for.
    /// The `index`/`num_in` of the [`Turn`] is re-calculated relative to the center of the cube (so `index` remains the same) for the each turn in the move.
    /// Any turn that can't exist for a cube with the new cube size will be removed from the move.
    /// 
    /// [`Turn`]: enum.Turn.html
    #[allow(dead_code)]
    pub fn change_cube_size_hold_face(self, new_cube_size: usize) -> Self
    {
        // Well return `Err(())` if any turn can't exist for a cube with the new cube size.
        // let mut turns: Vec<Turn> = vec![Turn::default(); self.turns.len()];
        
        // for (i, turn) in self.turns.into_iter().enumerate()
        // {
        //     turns[i] = turn.change_cube_size_hold_face(new_cube_size)?;
        // }

        // Ok(Move{turns})

        Move{turns: self.turns.into_iter()
            .map(|t| t.change_cube_size_hold_face(new_cube_size))
            .filter(|t| matches!(t, Ok(_))).map(|t| t.unwrap()).collect()}
    }

    pub fn empty() -> Self
    {
        Move{turns: vec![]}
    }
}

impl fmt::Display for Move
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(")?;
        if self.turns.len() >= 1
        {
            if let Turn::FaceBased{face, inv, num_in, ..} = self.turns[0].into_face_based()
            {
                write!(f, "{}{}{}", face.as_char(), num_in, if inv {"\'"} else {""})?;
            }
            else
            {
                unreachable!()
            }
            if self.turns.len() > 1
            {
                for turn in &self.turns[1..]
                {
                    if let Turn::FaceBased{face, inv, num_in, ..} = turn.into_face_based()
                    {
                        write!(f, ", {}{}{}", face.as_char(), num_in, if inv {"\'"} else {""})?;
                    }
                    else
                    {
                        // rotate until we find correct orientation
                        unreachable!()
                    }
                }
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl PartialEq for Move
{
    // TODO: add more
    fn eq(&self, other: &Self) -> bool
    {
        // TODO: should I count L' and L^3 and the same move?
        for i in 0..self.turns.len()
        {
            if self.turns[i] != other.turns[i]
            {
                return false;
            }
        }

        return true;
    }
}

impl ops::Mul for Move
{
    type Output = Self;

    fn mul(mut self, mut rhs: Self) -> Self {
        self.append(&mut rhs);
        self
    }
}

impl ops::MulAssign for Move
{
    fn mul_assign(&mut self, mut rhs: Self) {
        self.append(&mut rhs);
    }
}

impl IntoIterator for Move
{
    type Item = Turn;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.turns.into_iter()
    }
}

/// Rubik's Cube State
#[derive(Clone)]
pub struct RubiksCubeState
{
    n: usize,
    data: Vec<Color>
}

impl Hash for RubiksCubeState
{
    /// We dont care about the bottom back right cubie. Only works for 2x2x2 cubes
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        if self.n != 2 { unimplemented!() }

        // we hash such that bottom left cube is b-o-y color
        if self.data[15] == Color::Blue && self.data[18] == Color::Orange && self.data[23] == Color::Yellow
        {
            // oriented correctly
            for c in &self.data
            {
                c.hash(state);
            }
            return;
        }
        else
        {
            let mut new_cube = self.clone();
            // I know this try the same rotation multiple times but I don't care
            for _ in 0..4
            {
                for _ in 0..4
                {
                    for _ in 0..4
                    {
                        if new_cube.data[15] == Color::Blue &&
                           new_cube.data[18] == Color::Orange &&
                           new_cube.data[23] == Color::Yellow
                        {
                            // oriented correctly
                            for c in &new_cube.data
                            {
                                c.hash(state);
                            }
                            return;
                        }
                        new_cube.rotate_cube(Axis::Z);
                    }
                    new_cube.rotate_cube(Axis::Y);
                }
                new_cube.rotate_cube(Axis::X);
            }
        }

        unimplemented!();
    }
}

impl PartialEq for RubiksCubeState
{
    fn eq(&self, other: &Self) -> bool
    {
        if self.n != other.n
        {
            return false;
        }

        for i in 0..self.data.len()
        {
            if self.data[i] != other.data[i]
            {
                return false;
            }
        }

        return true;
    }
}

impl Eq for RubiksCubeState {}

impl fmt::Debug for RubiksCubeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        let mut cube_print_data = vec![];
        // UP
        for i in 0..self.n
        {
            let mut line = (0..self.n).map(|_| ' ').collect::<String>();
            line.push(' ');

            for j in 0..self.n
            {
                line.push(self.data[self.n*i + j].as_char());
            }

            cube_print_data.push(line);
        }

        // LFRB
        for i in 0..self.n
        {
            let mut line = String::from("");

            // Left
            for j in 0..self.n
            {
                line.push(self.data[self.n*self.n + self.n*i + j].as_char());
            }
            line.push(' ');
            
            // Front
            for j in 0..self.n
            {
                line.push(self.data[self.n*self.n*2 + self.n*i + j].as_char());
            }
            line.push(' ');
            
            // Right
            for j in 0..self.n
            {
                line.push(self.data[self.n*self.n*3 + self.n*i + j].as_char());
            }
            line.push(' ');
            
            // Back
            for j in 0..self.n
            {
                line.push(self.data[self.n*self.n*4 + self.n*i + j].as_char());
            }

            cube_print_data.push(line);
        }

        // Down
        for i in 0..self.n
        {
            let mut line = (0..self.n).map(|_| ' ').collect::<String>();
            line.push(' ');

            for j in 0..self.n
            {
                line.push(self.data[self.n*self.n*5 + self.n*i + j].as_char());
            }

            cube_print_data.push(line);
        }

        for line in cube_print_data
        {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl RubiksCubeState
{
    /// String must be of size 6 * n^2. Each char will be a color (W,G,R,B,O,Y).
    /// The face order is ULFRBD. Each face is given left to right top to bottom.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let solved_3x3_state = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    /// let state = RubiksCubeState::from_state_string(&solved_3x3_state);
    /// println!("{:?}", state.unwrap());
    /// ```
    /// Gives
    /// ```
    ///     WWW
    ///     WWW
    ///     WWW
    /// GGG RRR BBB OOO
    /// GGG RRR BBB OOO
    /// GGG RRR BBB OOO
    ///     YYY
    ///     YYY
    ///     YYY
    /// ```
    pub fn from_state_string(s: &String) -> io::Result<Self>
    {
        let len = s.len();
        if len % 6 != 0 || f64::sqrt(len as f64/6.0).floor().powi(2) as usize != len / 6
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "")); // TODO: add message
        }
        // assert_eq!(len % 6, 0);
        // assert_eq!(f64::sqrt(len as f64/6.0).floor().powi(2) as usize, len / 6);
        
        let n = f64::sqrt(len as f64/6.0).floor() as usize;

        let data = s.chars().map(|l| match l.to_ascii_lowercase() 
            {
                'w' => Color::White,
                'g' => Color::Green,
                'r' => Color::Red,
                'b' => Color::Blue,
                'o' => Color::Orange,
                'y' => Color::Yellow,
                _ => unimplemented!()
            }).collect();
        
        Ok(RubiksCubeState{n, data})
    }

    /// Gives a nxnxn cube with where ULFRBD faces have the colors W,G,R,B,O,Y respectively.
    /// And calling [`is_solved`] will return true.
    /// 
    /// [`is_solved`]: struct.RubiksCubeState.html#method.is_solved
    pub fn std_solved_nxnxn(n: usize) -> Self
    {
        let data = vec![Color::White, Color::Green, Color::Red, Color::Blue, Color::Orange, Color::Yellow]
            .into_iter().fold(vec![], |mut v, c| {v.append(&mut vec![c; n*n]); v});
        
        RubiksCubeState {n, data}
    }

    /// Produces a valid cube configuration by starting with [`std_solved_nxnxn`] and then making `num_turns` randoms turns.
    /// 
    /// [`std_solved_nxnxn`]: struct.RubiksCubeState.html#method.std_solved_nxnxn
    pub fn rnd_scramble(n: usize, num_turns: usize) -> (Self, Move)
    {
        let mut state = Self::std_solved_nxnxn(n);

        let rubiks_move = Move::rnd_move(n, num_turns);
        state.do_move(&rubiks_move);

        return (state, rubiks_move);
    }

    /// Creates a 2x2x2 cube from the corners of the `ref_state` cube.
    pub fn from_corners_to_2x2x2(ref_state: &Self) -> Self
    {
        let data = ref_state.data.clone().chunks_exact(ref_state.n).enumerate() // we will get 6n chunks
            .fold(vec![], |mut v, (i, c_row)| 
            {
                if i % ref_state.n == 0 || i % ref_state.n == ref_state.n-1
                { 
                    v.push(c_row[0]); 
                    v.push(*c_row.last().unwrap()); 
                }
                    v
            });
        
        RubiksCubeState {n: 2, data}
    }

    /// internal function used by `turn`
    fn rotate_face(&mut self, face: Face, inv: bool)
    {
        let offset = self.n * self.n * face as usize;
        let mut temp = vec![Color::White; self.n * self.n];
        for i in 0..self.n {
            for j in 0..self.n {
                if inv
                {
                    temp[i * self.n + j] = self.data[offset + j * self.n + (self.n - i - 1)];
                }
                else
                {
                    temp[i * self.n + j] = self.data[offset + (self.n - j - 1) * self.n + i];
                }
            }
        }
        for i in 0..self.n {
            for j in 0..self.n {
                self.data[offset + i * self.n + j] = temp[i * self.n + j];
            }
        }
    }

    /// Will apply a turn
    pub fn turn(&mut self, turn: Turn)
    {
        if let Turn::FaceBased{face, inv, num_in, cube_size} = turn.into_face_based()
        {
            assert_eq!(cube_size, self.n);
            assert!(num_in < self.n/2);

            // We will count 0 and 1 to be the same
            if num_in == 0
            {
                self.rotate_face(face, inv)
            }

            match face
            {
                Face::Up => 
                {
                    let face_offset = self.n * self.n;
                    let row_offset = self.n * num_in;
                    for i in 0..self.n
                    {
                        if inv
                        {
                            let temp = self.data[face_offset + row_offset + i];
                            self.data[face_offset + row_offset + i] = self.data[face_offset*4 + row_offset + i];
                            self.data[face_offset*4 + row_offset + i] = self.data[face_offset*3 + row_offset + i];
                            self.data[face_offset*3 + row_offset + i] = self.data[face_offset*2 + row_offset + i];
                            self.data[face_offset*2 + row_offset + i] = temp;
                        }
                        else
                        {
                            let temp = self.data[face_offset + row_offset + i];
                            self.data[face_offset + row_offset + i] = self.data[face_offset*2 + row_offset + i];
                            self.data[face_offset*2 + row_offset + i] = self.data[face_offset*3 + row_offset + i];
                            self.data[face_offset*3 + row_offset + i] = self.data[face_offset*4 + row_offset + i];
                            self.data[face_offset*4 + row_offset + i] = temp;
                        }
                    }
                },
                Face::Left => 
                {
                    let face_offset = self.n * self.n;
                    let row_offset = num_in;
                    for i in 0..self.n
                    {
                        if inv
                        {
                            let temp = self.data[i*self.n + row_offset];
                            self.data[i*self.n + row_offset] = self.data[face_offset*2 + i*self.n + row_offset];
                            self.data[face_offset*2 + i*self.n + row_offset] = self.data[face_offset*5 + i*self.n + row_offset];
                            self.data[face_offset*5 + i*self.n + row_offset] = self.data[face_offset*4 + (self.n - i - 1)*self.n + (self.n - row_offset - 1)];
                            self.data[face_offset*4 + (self.n - i - 1)*self.n + (self.n - row_offset - 1)] = temp;
                        }
                        else
                        {
                            let temp = self.data[i*self.n + row_offset];
                            self.data[i*self.n + row_offset] = self.data[face_offset*4 + (self.n - i - 1)*self.n + (self.n - row_offset - 1)];
                            self.data[face_offset*4 + (self.n - i - 1)*self.n + (self.n - row_offset - 1)] = self.data[face_offset*5 + i*self.n + row_offset];
                            self.data[face_offset*5 + i*self.n + row_offset] = self.data[face_offset*2 + i*self.n + row_offset];
                            self.data[face_offset*2 + i*self.n + row_offset] = temp;
                        }
                    }
                },
                Face::Front => 
                {
                    let face_offset = self.n * self.n;
                    for i in 0..self.n
                    {
                        if inv
                        {
                            let temp = self.data[(self.n - num_in - 1)*self.n + i];
                            self.data[(self.n - num_in - 1)*self.n + i] = self.data[face_offset*3 + i*self.n + num_in];
                            self.data[face_offset*3 + i*self.n + num_in] = self.data[face_offset*5 + num_in*self.n + (self.n - i - 1)];
                            self.data[face_offset*5 + num_in*self.n + (self.n - i - 1)] = self.data[face_offset*1 + (self.n - i - 1)*self.n + (self.n - num_in - 1)];
                            self.data[face_offset*1 + (self.n - i - 1)*self.n + (self.n - num_in - 1)] = temp;
                        }
                        else
                        {
                            let temp = self.data[(self.n - num_in - 1)*self.n + i];
                            self.data[(self.n - num_in - 1)*self.n + i] = self.data[face_offset*1 + (self.n - i - 1)*self.n + (self.n - num_in - 1)];
                            self.data[face_offset*1 + (self.n - i - 1)*self.n + (self.n - num_in - 1)] = self.data[face_offset*5 + num_in*self.n + (self.n - i - 1)];
                            self.data[face_offset*5 + num_in*self.n + (self.n - i - 1)] = self.data[face_offset*3 + i*self.n + num_in];
                            self.data[face_offset*3 + i*self.n + num_in] = temp;
                        }
                    }
                },
                Face::Right => 
                {
                    
                    let face_offset = self.n * self.n;
                    let row_offset = self.n - num_in - 1;
                    for i in 0..self.n
                    {
                        if inv
                        {
                            let temp = self.data[i*self.n + row_offset];
                            self.data[i*self.n + row_offset] = self.data[face_offset*4 + (self.n - i - 1)*self.n + (self.n - row_offset - 1)];
                            self.data[face_offset*4 + (self.n - i - 1)*self.n + (self.n - row_offset - 1)] = self.data[face_offset*5 + i*self.n + row_offset];
                            self.data[face_offset*5 + i*self.n + row_offset] = self.data[face_offset*2 + i*self.n + row_offset];
                            self.data[face_offset*2 + i*self.n + row_offset] = temp;
                        }
                        else
                        {
                            let temp = self.data[i*self.n + row_offset];
                            self.data[i*self.n + row_offset] = self.data[face_offset*2 + i*self.n + row_offset];
                            self.data[face_offset*2 + i*self.n + row_offset] = self.data[face_offset*5 + i*self.n + row_offset];
                            self.data[face_offset*5 + i*self.n + row_offset] = self.data[face_offset*4 + (self.n - i - 1)*self.n + (self.n - row_offset - 1)];
                            self.data[face_offset*4 + (self.n - i - 1)*self.n + (self.n - row_offset - 1)] = temp;
                        }
                    }
                },
                Face::Back => 
                {
                    let face_offset = self.n * self.n;
                    for i in 0..self.n
                    {
                        if inv
                        {
                            let temp = self.data[self.n * num_in + i];
                            self.data[self.n * num_in + i] = self.data[face_offset*1 + (self.n - i - 1)*self.n + num_in];
                            self.data[face_offset*1 + (self.n - i - 1)*self.n + num_in] = self.data[face_offset*5 + (self.n - num_in - 1)*self.n + (self.n - i - 1)];
                            self.data[face_offset*5 + (self.n - num_in - 1)*self.n + (self.n - i - 1)] = self.data[face_offset*3 + i*self.n + (self.n - num_in - 1)];
                            self.data[face_offset*3 + i*self.n + (self.n - num_in - 1)] = temp;
                        }
                        else
                        {
                            let temp = self.data[self.n * num_in + i];
                            self.data[self.n * num_in + i] = self.data[face_offset*3 + i*self.n + (self.n - num_in - 1)];
                            self.data[face_offset*3 + i*self.n + (self.n - num_in - 1)] = self.data[face_offset*5 + (self.n - num_in - 1)*self.n + (self.n - i - 1)];
                            self.data[face_offset*5 + (self.n - num_in - 1)*self.n + (self.n - i - 1)] = self.data[face_offset*1 + (self.n - i - 1)*self.n + num_in];
                            self.data[face_offset*1 + (self.n - i - 1)*self.n + num_in] = temp;
                        }
                    }
                },
                Face::Down => 
                {
                    let face_offset = self.n * self.n;
                    let row_offset = self.n * (self.n - num_in - 1);
                    for i in 0..self.n
                    {
                        if inv
                        {
                            let temp = self.data[face_offset + row_offset + i];
                            self.data[face_offset + row_offset + i] = self.data[face_offset*2 + row_offset + i];
                            self.data[face_offset*2 + row_offset + i] = self.data[face_offset*3 + row_offset + i];
                            self.data[face_offset*3 + row_offset + i] = self.data[face_offset*4 + row_offset + i];
                            self.data[face_offset*4 + row_offset + i] = temp;
                        }
                        else
                        {
                            let temp = self.data[face_offset + row_offset + i];
                            self.data[face_offset + row_offset + i] = self.data[face_offset*4 + row_offset + i];
                            self.data[face_offset*4 + row_offset + i] = self.data[face_offset*3 + row_offset + i];
                            self.data[face_offset*3 + row_offset + i] = self.data[face_offset*2 + row_offset + i];
                            self.data[face_offset*2 + row_offset + i] = temp;
                        }
                    }
                }
            };
        }
    }

    /// Will apply a move
    pub fn do_move(&mut self, rubiks_move: &Move)
    {
        for turn in &(*rubiks_move).turns
        {
            self.turn(*turn);
        }
    }

    /// Returns a list of all valid turns that can be made
    pub fn all_turns(&self) -> Vec<Turn>
    {
        let mut all_turns = vec![];

        for face_id in 0..6
        {
            let face = match face_id
            {
                0 => Face::Up,
                1 => Face::Left,
                2 => Face::Front,
                3 => Face::Right,
                4 => Face::Back,
                _ => Face::Down
            };

            for i in 0..(self.n/2)
            {
                all_turns.push(Turn::FaceBased{face, inv: true, num_in: i, cube_size: self.n});
                all_turns.push(Turn::FaceBased{face, inv: false, num_in: i, cube_size: self.n});
            }
        }

        return all_turns;
    }

    /// Checks if each face is the same color
    pub fn is_solved(&self) -> bool
    {
        let face_offset = self.n * self.n;
        for face in 0..6
        {
            let first_color = self.data[face_offset * face];
            for i in 1..(self.n*self.n)
            {
                if self.data[face_offset * face + i] != first_color 
                {
                    return false;
                }
            }
        }

        return true;
    }

    /// returns `n` for a `nxnxn` rubik's cube
    pub fn size(&self) -> usize
    {
        self.n
    }

    pub fn data_at(&self, i: usize) -> Color
    {
        self.data[i]
    }

    /// rotates all the faces on the cube, not a slice.
    /// Rotates in teh positive direction.
    pub fn rotate_cube(&mut self, axis: Axis)
    {
        let nn = self.n * self.n;
        match axis 
        {
            Axis::X =>
            {
                self.rotate_face(Face::Back, false);
                self.rotate_face(Face::Back, false);

                self.rotate_face(Face::Right, false);
                self.rotate_face(Face::Left, true);

                for i in 0..nn
                {
                    let temp = self.data[i];
                    self.data[i] = self.data[2*nn + i];
                    self.data[2*nn + i] = self.data[5*nn + i];
                    self.data[5*nn + i] = self.data[4*nn + i];
                    self.data[4*nn + i] = temp;
                }

                self.rotate_face(Face::Back, false);
                self.rotate_face(Face::Back, false);
            },
            Axis::Y =>
            {
                self.rotate_face(Face::Back, false);
                self.rotate_face(Face::Front, true);

                for i in 0..nn
                {
                    let temp = self.data[i];
                    self.data[i] = self.data[3*nn + i];
                    self.data[3*nn + i] = self.data[5*nn + i];
                    self.data[5*nn + i] = self.data[1*nn + i];
                    self.data[1*nn + i] = temp;
                }

                self.rotate_face(Face::Up, true);
                self.rotate_face(Face::Left, true);
                self.rotate_face(Face::Down, true);
                self.rotate_face(Face::Right, true);
            },
            Axis::Z =>
            {
                self.rotate_face(Face::Down, false);
                self.rotate_face(Face::Up, true);

                for i in 0..nn
                {
                    let temp = self.data[1*nn + i];
                    self.data[1*nn + i] = self.data[4*nn + i];
                    self.data[4*nn + i] = self.data[3*nn + i];
                    self.data[3*nn + i] = self.data[2*nn + i];
                    self.data[2*nn + i] = temp;
                }
            },
        }
    }

    /// TODO: i don't want to have this
    pub fn rotate_to_normal_2x2x2(&mut self)
    {
        if self.n != 2 {return};

        // I know this try the same rotation multiple times but I don't care
        for _ in 0..4
        {
            for _ in 0..4
            {
                for _ in 0..4
                {
                    if self.data[15] == Color::Blue &&
                        self.data[18] == Color::Orange &&
                        self.data[23] == Color::Yellow
                    {
                        return;
                    }
                    self.rotate_cube(Axis::Z);
                }
                self.rotate_cube(Axis::Y);
            }
            self.rotate_cube(Axis::X);
        }
    }
}

#[test]
fn test_is_solved()
{
    // TODO: do better
    let solved_3x3_state = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    let solved_3x3_state2 = "WWWWWWWWWOOOOOOOOOGGGGGGGGGRRRRRRRRRBBBBBBBBBYYYYYYYYY".to_owned();
    let solved_4x4_state = "WWWWWWWWWWWWWWWWGGGGGGGGGGGGGGGGRRRRRRRRRRRRRRRRBBBBBBBBBBBBBBBBOOOOOOOOOOOOOOOOYYYYYYYYYYYYYYYY".to_owned();
    let solved_5x5_state = "WWWWWWWWWWWWWWWWWWWWWWWWWGGGGGGGGGGGGGGGGGGGGGGGGGRRRRRRRRRRRRRRRRRRRRRRRRRBBBBBBBBBBBBBBBBBBBBBBBBBOOOOOOOOOOOOOOOOOOOOOOOOOYYYYYYYYYYYYYYYYYYYYYYYYY".to_owned();
    let solved_5x5_state2 = "BBBBBBBBBBBBBBBBBBBBBBBBBOOOOOOOOOOOOOOOOOOOOOOOOOWWWWWWWWWWWWWWWWWWWWWWWWWRRRRRRRRRRRRRRRRRRRRRRRRRYYYYYYYYYYYYYYYYYYYYYYYYYGGGGGGGGGGGGGGGGGGGGGGGGG".to_owned();

    assert_eq!(RubiksCubeState::from_state_string(&solved_3x3_state).unwrap().is_solved(), true);
    assert_eq!(RubiksCubeState::from_state_string(&solved_3x3_state2).unwrap().is_solved(), true);
    assert_eq!(RubiksCubeState::from_state_string(&solved_4x4_state).unwrap().is_solved(), true);
    assert_eq!(RubiksCubeState::from_state_string(&solved_5x5_state).unwrap().is_solved(), true);
    assert_eq!(RubiksCubeState::from_state_string(&solved_5x5_state2).unwrap().is_solved(), true);

    let nsolved_3x3_state = "WWWWWWWWWGGGGGGGGGRRRRRRRRRYBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    let nsolved_3x3_state2 = "WWWWWWWWWOOOOOOOOOGGGGGGGGGRRRRRRRRRBBBBBBBBBBYYYYYYYY".to_owned();
    let nsolved_4x4_state = "WWWWWWWWWWWWWWWWGGGGGGGGGGGGGGGGRRRRRRRRRRRRRRRRBBBBBBBBBBBBWBBBOOOOOOOOOOOOOOOOYYYYYYYYYYYYYYYY".to_owned();
    let nsolved_5x5_state = "WWWWWWWWWWWWWWWWWWWWWWWWWGGGGGGGGGGGGGGGGGGGGGGGGGRRRRRRRRRRRRRRRRRRRRRRRRRBBBBBBBBBBBBBBBBBBBBBBBBBOOOOOOOOOOOOOOOOOOOOOOOOOWYYYYYYYYYYYYYYYYYYYYYYYY".to_owned();
    let nsolved_5x5_state2 = "BBBBBBBBBBBBBBBBBBBBBBBBBOOOOOOOOOOOOOOOOOOOOBOOOOWWWWWWWWWWWWWWWWWWWWWWWWWRRRRRRRRRRRRRRRRRRRRRRRRRYYYYYYYYYYYYYYYYYYYYYYYYYGGGGGGGGGGGGGGGGGGGGGGGGG".to_owned();

    assert_eq!(RubiksCubeState::from_state_string(&nsolved_3x3_state).unwrap().is_solved(), false);
    assert_eq!(RubiksCubeState::from_state_string(&nsolved_3x3_state2).unwrap().is_solved(), false);
    assert_eq!(RubiksCubeState::from_state_string(&nsolved_4x4_state).unwrap().is_solved(), false);
    assert_eq!(RubiksCubeState::from_state_string(&nsolved_5x5_state).unwrap().is_solved(), false);
    assert_eq!(RubiksCubeState::from_state_string(&nsolved_5x5_state2).unwrap().is_solved(), false);

    for n in 2..10
    {
        assert_eq!(RubiksCubeState::std_solved_nxnxn(n).is_solved(), true);
    }
}

#[test]
fn test_turns()
{
    let solved_3x3_state_str = "WWWWWWWWWOOOOOOOOOGGGGGGGGGRRRRRRRRRBBBBBBBBBYYYYYYYYY".to_owned();
    let mut state_3x3 = RubiksCubeState::from_state_string(&solved_3x3_state_str).unwrap();
    let mut state2_3x3 = RubiksCubeState::from_state_string(&solved_3x3_state_str).unwrap();
    state_3x3.turn(Turn::FaceBased{face: Face::Down, inv: true, num_in:0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Back, inv: true, num_in:0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Up, inv: false,num_in: 0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Back, inv: false,num_in: 0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Down, inv: false,num_in: 0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Front, inv: true, num_in:0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Right, inv: true, num_in:0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Front, inv: false,num_in: 0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Left, inv: false,num_in: 0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Right, inv: false,num_in: 0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Up, inv: true, num_in:0, cube_size: 3});
    state_3x3.turn(Turn::FaceBased{face: Face::Left, inv: true, num_in:0, cube_size: 3});
    let solved_3x3_state_with_turns = "OGWWWWWOYYGGBOOOOGRWGGGGROWORRYRRGRRBRBBBWBBWYBOYYYBYY".to_owned();
    assert_eq!(state_3x3, RubiksCubeState::from_state_string(&solved_3x3_state_with_turns).unwrap());

    let rubiks_move = Move{turns: vec![Turn::FaceBased{face: Face::Down, inv: true, num_in:0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Back, inv: true, num_in:0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Up, inv: false,num_in: 0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Back, inv: false,num_in: 0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Down, inv: false,num_in: 0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Front, inv: true, num_in:0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Right, inv: true, num_in:0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Front, inv: false,num_in: 0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Left, inv: false,num_in: 0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Right, inv: false,num_in: 0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Up, inv: true, num_in:0, cube_size: 3},
                                      Turn::FaceBased{face: Face::Left, inv: true, num_in:0, cube_size: 3}]};

    state2_3x3.do_move(&rubiks_move);
    
    assert_eq!(state2_3x3, RubiksCubeState::from_state_string(&solved_3x3_state_with_turns).unwrap());

    // TODO: more and better
}

#[test]
fn test_move_inv()
{
    let move_empty = Move::empty();
    assert_eq!(move_empty, move_empty.clone().invert());

    for _ in 0..10
    {
        let (mut state, rubiks_move) = RubiksCubeState::rnd_scramble(15, 1000);
        state.do_move(&rubiks_move.invert());

        assert!(state.is_solved());
    }
}

#[test]
fn test_move_append()
{
    let move_empty = Move::empty();
    let move_empty2 = Move::empty();

    // mult op does the append (order matters)
    assert_eq!(move_empty, move_empty.clone() * move_empty2);

    for _ in 0..10
    {
        let mut state = RubiksCubeState::std_solved_nxnxn(15);
        let mut state2 = RubiksCubeState::std_solved_nxnxn(15);
        let rubiks_move = Move::rnd_move(15, 1000);
        state.do_move(&(rubiks_move.clone().invert() * rubiks_move.clone()));
        state2.do_move(&(rubiks_move.clone() * rubiks_move.clone().invert()));

        assert!(state.is_solved());
        assert!(state2.is_solved());

        assert_eq!(rubiks_move.clone(), move_empty.clone() * rubiks_move.clone());
        assert_eq!(rubiks_move.clone(), rubiks_move.clone() * move_empty.clone());

        let rubiks_move2 = Move::rnd_move(15, 1000);
        let mut state3 = RubiksCubeState::std_solved_nxnxn(15);
        let mut state4 = RubiksCubeState::std_solved_nxnxn(15);
        state3.do_move(&(rubiks_move.clone() * rubiks_move2.clone()));
        state4.do_move(&(rubiks_move2.clone() * rubiks_move.clone()));

        // This is not always try (but very likely)
        assert_ne!(state3, state4);
    }
}

#[test]
fn test_turn_converts()
{
    for turn in Move::rnd_move(11, 1000).turns
    {
        assert_eq!(turn.into_axis_based(), turn.into_face_based().into_axis_based());
        assert_eq!(turn.into_face_based(), turn.into_axis_based().into_face_based());
        assert_eq!(turn.into_axis_based(), turn.into_face_based());
        assert_eq!(turn.into_face_based(), turn.into_axis_based());
    }
}

#[test]
fn test_change_cube_size()
{
    for n in 2..10
    {
        let (state_rnd, scram_move) = RubiksCubeState::rnd_scramble(n, 100);
        let mut state_rnd_as_2x2x2 = RubiksCubeState::from_corners_to_2x2x2(&state_rnd);

        let soln_move_orig_cube = scram_move.invert();
        let soln_move_2x2x2 = soln_move_orig_cube.change_cube_size_hold_face(2);

        state_rnd_as_2x2x2.do_move(&soln_move_2x2x2);

        assert_eq!(state_rnd_as_2x2x2.is_solved(), true);
        
        let scram_move_2x2x2 = Move::rnd_move(2, 100);
        let solve_move_orig = scram_move_2x2x2.clone().invert();
        let scram_move_nxnxn = scram_move_2x2x2.change_cube_size_hold_center(n);
        let solve_move_nxnxn = solve_move_orig.change_cube_size_hold_center(n);
        let mut state_rnd = RubiksCubeState::std_solved_nxnxn(n);
        state_rnd.do_move(&scram_move_nxnxn);
        state_rnd.do_move(&solve_move_nxnxn);

        assert_eq!(state_rnd.is_solved(), true);
    }
}

#[test]
fn test_rotate_cube()
{
    for n in (1..10).map(|n| n*2)
    {
        let (mut state_rnd, _scram_move) = RubiksCubeState::rnd_scramble(n, 1000);
        let mut state_rnd2 = state_rnd.clone();
        let mut state_rnd3 = state_rnd.clone();
        let mut state_rnd4 = state_rnd.clone();
        let mut state_rnd5 = state_rnd.clone();
        let mut state_rnd6 = state_rnd.clone();

        let turn_move = Move{turns: (-(n as isize)/2..=(n as isize)/2).filter(|i| *i != 0).map(|i| Turn::AxisBased{axis: Axis::X, pos_rot: true, index: i, cube_size: n}).collect()};
        
        state_rnd.do_move(&turn_move);
        state_rnd2.rotate_cube(Axis::X);
        

        let turn_move2 = Move{turns: (-(n as isize)/2..=(n as isize)/2).filter(|i| *i != 0).map(|i| Turn::AxisBased{axis: Axis::Y, pos_rot: true, index: i, cube_size: n}).collect()};
        
        state_rnd3.do_move(&turn_move2);
        state_rnd4.rotate_cube(Axis::Y);
        

        let turn_move3 = Move{turns: (-(n as isize)/2..=(n as isize)/2).filter(|i| *i != 0).map(|i| Turn::AxisBased{axis: Axis::Z, pos_rot: true, index: i, cube_size: n}).collect()};
        
        state_rnd5.do_move(&turn_move3);
        state_rnd6.rotate_cube(Axis::Z);

        assert_eq!(state_rnd, state_rnd2);
        assert_eq!(state_rnd3, state_rnd4);
        assert_eq!(state_rnd5, state_rnd6);
    }

    // TODO: try odd sized cubes
}

#[test]
fn test_hash()
{
    let mut rng = rand::thread_rng();

    for _ in 0..100
    {
        let (state_rnd, _scram_move) = RubiksCubeState::rnd_scramble(2, 1000);
        let mut state_rnd2 = state_rnd.clone();

        let x_rots = rng.gen_range(0, 4);
        let y_rots = rng.gen_range(0, 4);
        let z_rots = rng.gen_range(0, 4);

        for _ in 0..x_rots {state_rnd2.rotate_cube(Axis::X);}
        for _ in 0..y_rots {state_rnd2.rotate_cube(Axis::Y);}
        for _ in 0..z_rots {state_rnd2.rotate_cube(Axis::Z);}

        let mut hasher1 = DefaultHasher::new();
        state_rnd.hash(&mut hasher1);
        let mut hasher2 = DefaultHasher::new();
        state_rnd2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}

#[test]
fn doc_tester()
{
    let mut state = RubiksCubeState::std_solved_nxnxn(3);

    let u_inv_t = Turn::FaceBased{face: Face::Up, inv: true, num_in:0, cube_size: 3};
    let f_inv_t = Turn::FaceBased{face: Face::Front, inv: true, num_in:0, cube_size: 3};
    let l_inv_t = Turn::FaceBased{face: Face::Left, inv: true, num_in:0, cube_size: 3};

    let three_turn_move = u_inv_t.as_move() * f_inv_t.as_move() * l_inv_t.as_move();

    state.do_move(&three_turn_move);

    println!("{:?}", state);
}
