use crate::Player;

// sorry
impl crate::Ultimate {
    pub fn is_playable(&self, coord: usize) -> bool {
        let mut is_playable = self.tiles[coord].is_none();
        if let Some(last_move) = self.history.last() {
            let open_minisquare_x = last_move % 3;
            let open_minisquare_y = (last_move / 9) % 3;
            let current_minisquare_x = (coord % 9) / 3;
            let current_minisquare_y = coord / 27;
            is_playable &= (open_minisquare_x == current_minisquare_x
                && open_minisquare_y == current_minisquare_y)
                || self.minisquares[open_minisquare_y * 3 + open_minisquare_x].is_some();
            is_playable &=
                self.minisquares[current_minisquare_y * 3 + current_minisquare_x].is_none();
        };
        is_playable
    }

    pub fn make_move(&mut self, coord: usize) {
        assert!(self.is_playable(coord));
        assert_eq!(self.whose_turn, Some(self.local_player));
        self.tiles[coord] = Some(self.local_player);

        // did we win a minisquare?
        let inner_x = coord % 3;
        let inner_y = (coord / 9) % 3;
        let outer_x = (coord % 9) / 3;
        let outer_y = coord / 27;
        let us = self.local_player;
        let topleft = outer_y * 27 + outer_x * 3;
        let won_axis = (self.tiles[topleft + inner_y * 9] == Some(us)
            && self.tiles[topleft + inner_y * 9 + 1] == Some(us)
            && self.tiles[topleft + inner_y * 9 + 2] == Some(us))
            || (self.tiles[topleft + inner_x] == Some(us)
                && self.tiles[topleft + 1 * 9 + inner_x] == Some(us)
                && self.tiles[topleft + 2 * 9 + inner_x] == Some(us));
        let won_diagonal = ([0, 2].contains(&inner_x) && [0, 2].contains(&inner_y))
            && self.tiles[topleft + inner_y * 9 + inner_x] == Some(us)
            && self.tiles[topleft + 9 + 1] == Some(us)
            && self.tiles[topleft + (inner_y ^ 2) * 9 + (inner_x ^ 2)] == Some(us) /* trust */;
        if won_axis || won_diagonal {
            self.minisquares[outer_y * 3 + outer_x] = Some(us);
            eprintln!("won {}, {}", outer_x, outer_y);
        }

        // did we win the game?
        let mut won_axis = self.minisquares.chunks(3).any(|row| row == [Some(us); 3]);
        for i in 0..3 {
            won_axis |= self.minisquares[i] == Some(us)
                && self.minisquares[i + 3] == Some(us)
                && self.minisquares[i + 6] == Some(us);
        }
        let won_diagonal = self.minisquares[4] == Some(us)
            && (self.minisquares[0] == Some(us) && self.minisquares[8] == Some(us)
                || self.minisquares[2] == Some(us) && self.minisquares[6] == Some(us));

        self.history.push(coord);

        if won_axis || won_diagonal {
            self.whose_turn = None;
            eprintln!("won the game");
            return;
        }
        let whose_turn = match self.whose_turn {
            Some(Player::Nought) => Player::Cross,
            Some(Player::Cross) => Player::Nought,
            None => unreachable!("there's an assert further up"),
        };
        self.whose_turn = Some(whose_turn);
        self.local_player = whose_turn; // local multiplayer
    }
}
