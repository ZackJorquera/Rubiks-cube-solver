use std::fmt;
use std::ops;
use rand;
use rand::prelude::*;

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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
/// Mappings between the to types:
/// - Up = +Z
/// - Left = +X
/// - Front = +Y
/// - Right = -X
/// - Back = -Y
/// - Down = -Z
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
            Turn::AxisBased{axis: Axis::X, pos_rot, index, cube_size} if index > 0 => Turn::FaceBased{face: Face::Left, inv: !pos_rot, num_in: cube_size/2 - index as usize, cube_size},
            Turn::AxisBased{axis: Axis::X, pos_rot, index, cube_size} => Turn::FaceBased{face: Face::Right, inv: pos_rot, num_in: cube_size/2 - (-index) as usize, cube_size},
            Turn::AxisBased{axis: Axis::Y, pos_rot, index, cube_size} if index > 0 => Turn::FaceBased{face: Face::Front, inv: !pos_rot, num_in: cube_size/2 - index as usize, cube_size},
            Turn::AxisBased{axis: Axis::Y, pos_rot, index, cube_size} => Turn::FaceBased{face: Face::Back, inv: pos_rot, num_in: cube_size/2 - (-index) as usize, cube_size},
            Turn::AxisBased{axis: Axis::Z, pos_rot, index, cube_size} if index > 0 => Turn::FaceBased{face: Face::Up, inv: !pos_rot, num_in: cube_size/2 - index as usize, cube_size},
            Turn::AxisBased{axis: Axis::Z, pos_rot, index, cube_size} => Turn::FaceBased{face: Face::Down, inv: pos_rot, num_in: cube_size/2 - ((-index) as usize), cube_size},
            
            t @ Turn::FaceBased{..} => t
        }
    }
    
    /// Converts to `Turn::AxisBased` enum variant.
    pub fn into_axis_based(self) -> Self
    {
        match self
        {
            Turn::FaceBased{face: Face::Up, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::Z, pos_rot: !inv, index: cube_size as isize/2 - num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Left, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::X, pos_rot: !inv, index: cube_size as isize/2 - num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Front, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::Y, pos_rot: !inv, index: cube_size as isize/2 - num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Right, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::X, pos_rot: inv, index: - (cube_size as isize)/2 + num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Back, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::Y, pos_rot: inv, index: - (cube_size as isize)/2 + num_in as isize, cube_size},
            Turn::FaceBased{face: Face::Down, inv, num_in, cube_size} => Turn::AxisBased{axis: Axis::Z, pos_rot: inv, index: - (cube_size as isize)/2 + num_in as isize, cube_size},

            t @ Turn::AxisBased{..} => t
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
}

/// A list of turns
#[derive(Debug, Clone)]
pub struct Move
{
    pub turns: Vec<Turn>
}

impl Move 
{
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

    /// Will create a random move for an nxnxn rubix cube with `num_turns` turns.
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
    /// - The turn commutes with the last move and it is not in the order U->D (higher turns first) L->R F->B.
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

/// Rubix Cube State
#[derive(Clone)]
pub struct RubixCubeState
{
    n: usize,
    data: Vec<Color>
}

impl PartialEq for RubixCubeState
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

impl fmt::Debug for RubixCubeState {
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

impl RubixCubeState
{
    /// String must be of size 6 * n^2. Each char will be a color (W,G,R,B,O,Y).
    /// The face order is ULFRBD. Each face is given left to right top to bottom.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let solved_3x3_state = "WWWWWWWWWGGGGGGGGGRRRRRRRRRBBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    /// let state = RubixCubeState::from_state_string(&solved_3x3_state);
    /// println!("{:?}", state);
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
    pub fn from_state_string(s: &String) -> Self
    {
        let len = s.len();
        assert_eq!(len % 6, 0);
        assert_eq!(f64::sqrt(len as f64/6.0).floor().powi(2) as usize, len / 6);
        
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
        
        Self{n, data}
    }

    /// Gives a nxnxn cube with where ULFRBD faces have the colors W,G,R,B,O,Y respectively.
    /// And calling [`is_solved`] will return true.
    /// 
    /// [`is_solved`]: struct.RubixCubeState.html#method.is_solved
    pub fn std_solved_nxnxn(n: usize) -> Self
    {
        let data = vec![Color::White, Color::Green, Color::Red, Color::Blue, Color::Orange, Color::Yellow]
            .into_iter().fold(vec![], |mut v, c| {v.append(&mut vec![c; n*n]); v});
        
        Self {n, data}
    }

    /// Produces a valid cube configuration by starting with [`std_solved_nxnxn`] and then making `num_turns` randoms turns.
    /// 
    /// [`std_solved_nxnxn`]: struct.RubixCubeState.html#method.std_solved_nxnxn
    pub fn rnd_scramble(n: usize, num_turns: usize) -> (Self, Move)
    {
        let mut state = Self::std_solved_nxnxn(n);

        let rubix_move = Move::rnd_move(n, num_turns);
        state.do_move(&rubix_move);

        return (state, rubix_move);
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
    pub fn do_move(&mut self, rubix_move: &Move)
    {
        for turn in &(*rubix_move).turns
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

    assert_eq!(RubixCubeState::from_state_string(&solved_3x3_state).is_solved(), true);
    assert_eq!(RubixCubeState::from_state_string(&solved_3x3_state2).is_solved(), true);
    assert_eq!(RubixCubeState::from_state_string(&solved_4x4_state).is_solved(), true);
    assert_eq!(RubixCubeState::from_state_string(&solved_5x5_state).is_solved(), true);
    assert_eq!(RubixCubeState::from_state_string(&solved_5x5_state2).is_solved(), true);

    let nsolved_3x3_state = "WWWWWWWWWGGGGGGGGGRRRRRRRRRYBBBBBBBBOOOOOOOOOYYYYYYYYY".to_owned();
    let nsolved_3x3_state2 = "WWWWWWWWWOOOOOOOOOGGGGGGGGGRRRRRRRRRBBBBBBBBBBYYYYYYYY".to_owned();
    let nsolved_4x4_state = "WWWWWWWWWWWWWWWWGGGGGGGGGGGGGGGGRRRRRRRRRRRRRRRRBBBBBBBBBBBBWBBBOOOOOOOOOOOOOOOOYYYYYYYYYYYYYYYY".to_owned();
    let nsolved_5x5_state = "WWWWWWWWWWWWWWWWWWWWWWWWWGGGGGGGGGGGGGGGGGGGGGGGGGRRRRRRRRRRRRRRRRRRRRRRRRRBBBBBBBBBBBBBBBBBBBBBBBBBOOOOOOOOOOOOOOOOOOOOOOOOOWYYYYYYYYYYYYYYYYYYYYYYYY".to_owned();
    let nsolved_5x5_state2 = "BBBBBBBBBBBBBBBBBBBBBBBBBOOOOOOOOOOOOOOOOOOOOBOOOOWWWWWWWWWWWWWWWWWWWWWWWWWRRRRRRRRRRRRRRRRRRRRRRRRRYYYYYYYYYYYYYYYYYYYYYYYYYGGGGGGGGGGGGGGGGGGGGGGGGG".to_owned();

    assert_eq!(RubixCubeState::from_state_string(&nsolved_3x3_state).is_solved(), false);
    assert_eq!(RubixCubeState::from_state_string(&nsolved_3x3_state2).is_solved(), false);
    assert_eq!(RubixCubeState::from_state_string(&nsolved_4x4_state).is_solved(), false);
    assert_eq!(RubixCubeState::from_state_string(&nsolved_5x5_state).is_solved(), false);
    assert_eq!(RubixCubeState::from_state_string(&nsolved_5x5_state2).is_solved(), false);

    for n in 2..10
    {
        assert_eq!(RubixCubeState::std_solved_nxnxn(n).is_solved(), true);
    }
}

#[test]
fn test_turns()
{
    let solved_3x3_state_str = "WWWWWWWWWOOOOOOOOOGGGGGGGGGRRRRRRRRRBBBBBBBBBYYYYYYYYY".to_owned();
    let mut state_3x3 = RubixCubeState::from_state_string(&solved_3x3_state_str);
    let mut state2_3x3 = RubixCubeState::from_state_string(&solved_3x3_state_str);
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
    assert_eq!(state_3x3, RubixCubeState::from_state_string(&solved_3x3_state_with_turns));

    let rubix_move = Move{turns: vec![Turn::FaceBased{face: Face::Down, inv: true, num_in:0, cube_size: 3},
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

    state2_3x3.do_move(&rubix_move);
    
    assert_eq!(state2_3x3, RubixCubeState::from_state_string(&solved_3x3_state_with_turns));

    // TODO: more and better
}

#[test]
fn test_move_inv()
{
    let move_empty = Move{turns: vec![]};
    assert_eq!(move_empty, move_empty.clone().invert());

    for _ in 0..10
    {
        let (mut state, rubix_move) = RubixCubeState::rnd_scramble(15, 1000);
        state.do_move(&rubix_move.invert());

        assert!(state.is_solved());
    }
}

#[test]
fn test_move_append()
{
    let move_empty = Move{turns: vec![]};
    let move_empty2 = Move{turns: vec![]};

    // mult op does the append (order matters)
    assert_eq!(move_empty, move_empty.clone() * move_empty2);

    for _ in 0..10
    {
        let mut state = RubixCubeState::std_solved_nxnxn(15);
        let mut state2 = RubixCubeState::std_solved_nxnxn(15);
        let rubix_move = Move::rnd_move(15, 1000);
        state.do_move(&(rubix_move.clone().invert() * rubix_move.clone()));
        state2.do_move(&(rubix_move.clone() * rubix_move.clone().invert()));

        assert!(state.is_solved());
        assert!(state2.is_solved());

        assert_eq!(rubix_move.clone(), move_empty.clone() * rubix_move.clone());
        assert_eq!(rubix_move.clone(), rubix_move.clone() * move_empty.clone());

        let rubix_move2 = Move::rnd_move(15, 1000);
        let mut state3 = RubixCubeState::std_solved_nxnxn(15);
        let mut state4 = RubixCubeState::std_solved_nxnxn(15);
        state3.do_move(&(rubix_move.clone() * rubix_move2.clone()));
        state4.do_move(&(rubix_move2.clone() * rubix_move.clone()));

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
