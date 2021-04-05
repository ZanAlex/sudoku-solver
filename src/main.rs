use std::fmt;

struct Grid {
    cells: [u8; 9 * 9],

    lines_state: [u16; 9],
    columns_state: [u16; 9],

    solved_indices_stack: Vec<usize>,
}

enum PotentialState {
    Several(u8),
    One(u8),
    None,
}

impl Grid {
    fn create(values: &[(u8, (usize, usize))]) -> Grid {
        let mut cells = [0; 9 * 9];

        for entry in values.iter() {
            let (value, (line, column)) = entry;
            cells[Grid::index_of(*line, *column)] = *value;
        }

        Grid {
            cells,
            lines_state: [0; 9],
            columns_state: [0; 9],
            solved_indices_stack: Vec::new(),
        }
    }

    fn index_of(line: usize, column: usize) -> usize {
        return line * 9 + column;
    }

    fn coordinates_of(index: usize) -> (usize, usize) {
        return (index / 9, index % 9);
    }

    fn interpret_potential(potential: u16) -> PotentialState {
        let mut count = 0;
        let mut last_value = 0;
        for i in 1..10 {
            if potential & (1 << i) != 0 {
                count += 1;
                last_value = i;
            }
        }

        match count {
            0 => { PotentialState::None },
            1 => { PotentialState::One(last_value) },
            _ => { PotentialState::Several(count) },
        }
    }

    fn get_potential_value_at(potential: u16, index: u8) -> u8 {
        let mut current_index = 0;
        for i in 1..10 {
            if potential & (1 << i) != 0 {
                if current_index == index {
                    return i;
                }

                current_index += 1;
            }
        }

        return 0;
    }

    fn any_unsolved_cell(&self) -> bool {
        for cell in self.cells.iter() {
            if *cell == 0 {
                return true;
            }
        }

        return false;
    }

    fn try_find_cell_with_low_potential(&self) -> Option<(usize, u16, u8)> {
        let mut result_index = usize::MAX;
        let mut result_potential = 0;
        let mut result_count = u8::MAX;
        for (index, cell) in self.cells.iter().enumerate() {
            if *cell > 0 {
                continue;
            }

            let (line, column) = Grid::coordinates_of(index);
            let potential = self.lines_state[line] & self.columns_state[column];
            match Grid::interpret_potential(potential) {
                PotentialState::Several(count) => {
                    if count < result_count {
                        result_index = index;
                        result_potential = potential;
                        result_count = count;
                    }
                },

                _ => {
                    // Warning? That should not happen...
                }
            }
        }

        if result_potential == 0 {
            return Option::None;
        }

        return Option::Some((result_index, result_potential, result_count));
    }

    fn solve(&mut self) {
        self.compute_potentials();
        let (success, _) = self.try_solve_by_constrains();

        if !success {
            panic!("Base constrains are wrong. Check your definition.");
        }

        self.try_solve_recursive();
    }

    fn compute_potentials(&mut self) {
        for potential in self.lines_state.iter_mut() { *potential = 0x3FE; }
        for potential in self.columns_state.iter_mut() { *potential = 0x3FE; }

        for (index, value) in self.cells.iter().enumerate() {
            if *value > 0 {
                let value_bit = 0x3FE & (1 << value);
                let (line, column) = Grid::coordinates_of(index);

                self.lines_state[line] &= !value_bit;
                self.columns_state[column] &= !value_bit;
            }
        }
    }

    fn try_solve_by_constrains(&mut self) -> (bool, usize) {
        let initial_solved_indices_count = self.solved_indices_stack.len();

        let mut any_solve = true;
        let mut any_error = false;
        while any_solve && !any_error {
            any_solve = false;

            for (index, cell) in self.cells.iter_mut().enumerate() {
                if *cell > 0 {
                    continue;
                }

                let (line, column) = Grid::coordinates_of(index);

                match Grid::interpret_potential(self.lines_state[line] & self.columns_state[column]) {
                    PotentialState::One(value) => {
                        // This value can be solved!
                        *cell = value;
                        self.solved_indices_stack.push(index);
                        self.compute_potentials();
                        any_solve = true;
                        break;
                    },

                    PotentialState::None => {
                        // This cell cannot have any value. Something went wrong!
                        any_error = true;
                        break;
                    },

                    _ => {},
                }
            }
        }

        return (!any_error, self.solved_indices_stack.len() - initial_solved_indices_count);
    }

    fn revert_last_resolved_indices(&mut self, count: usize) {
        for _ in 0..count {
            let reverted_indices = self.solved_indices_stack.pop().unwrap();
            self.cells[reverted_indices] = 0;
        }
    }

    fn try_solve_recursive(&mut self) -> bool {
        if !self.any_unsolved_cell() {
            return true;
        }

        if let Some((cell_index, potential, potential_count)) = self.try_find_cell_with_low_potential() {
            for potential_index in 0..potential_count {
                self.cells[cell_index] = Grid::get_potential_value_at(potential, potential_index);
                self.compute_potentials();

                let (success, number_of_solved_indices) = self.try_solve_by_constrains();

                if success && self.try_solve_recursive() {
                    return true;
                }

                self.revert_last_resolved_indices(number_of_solved_indices);
                self.cells[cell_index] = 0;
                self.compute_potentials();
            }
        }

        return false;
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in 0..9 {
            write!(f, "|")?;
            for column in 0..9 {
                let value = self.cells[Grid::index_of(line, column)];
                match value {
                    0 => write!(f, " |")?,
                    _ => write!(f, "{}|", value)?,
                }
            }

            writeln!(f, " ")?;
        }

        Ok(())
    }
}

fn main() {
    let init : [(u8, (usize, usize)); 32] = [
        (3, (0, 0)),
        (6, (0, 2)),
        (5, (0, 3)),
        (8, (0, 5)),
        (4, (0, 6)),
        (5, (1, 0)),
        (2, (1, 1)),
        (8, (2, 1)),
        (7, (2, 2)),
        (3, (2, 7)),
        (1, (2, 8)),
        (3, (3, 2)),
        (1, (3, 5)),
        (8, (3, 7)),
        (9, (4, 0)),
        (8, (4, 3)),
        (6, (4, 4)),
        (3, (4, 5)),
        (5, (4, 8)),
        (5, (5, 1)),
        (9, (5, 4)),
        (6, (5, 6)),
        (1, (6, 0)),
        (3, (6, 1)),
        (2, (6, 6)),
        (5, (6, 7)),
        (7, (7, 7)),
        (4, (7, 8)),
        (5, (8, 2)),
        (2, (8, 3)),
        (6, (8, 5)),
        (3, (8, 6)),
    ];

    let mut my_grid = Grid::create(&init);
    println!("{}", my_grid);
    my_grid.solve();
    println!("{}", my_grid);
}
