use amethyst::ecs::*;

use components::{
    block::Block,
    cursor::Cursor,
    playfield::{clear::Clear, stack::Stack},
};

use block_states::block_state::change_state;
use data::block_data::{BLOCKS, COLS, ROWS};
use std::cmp::max;

pub struct ClearSystem;

impl<'a> System<'a> for ClearSystem {
    type SystemData = (
        WriteStorage<'a, Clear>,
        WriteStorage<'a, Block>,
        ReadStorage<'a, Stack>,
    );

    fn run(&mut self, (mut clears, mut blocks, stacks): Self::SystemData) {
        // block clear detection
        // counts the amount of clears each frame, passes them uniquely to an array holding their ids
        // sets a lot of playfield_clear values and then sets the blocks to animate with given times
        for (clear, stack) in (&mut clears, &stacks).join() {
            for x in 0..COLS {
                for y in 0..ROWS {
                    for clear_block_id in check_clear(x, y, &stack, &blocks) {
                        if !clear.clear_queue.contains(&clear_block_id) {
                            clear.clear_queue.push(clear_block_id);
                        }
                    }
                }
            }

            // if no clears were found, don't go through all of them
            let clear_size = clear.clear_queue.len() as u32;
            if clear_size != 0 {
                clear.combo_counter = 0;

                // animation times, TODO: get playfield level dependant times
                let flash: u32 = 44;
                let face: u32 = 10;
                let pop: u32 = 10;

                let all_time: u32 = flash + face + pop * clear_size;

                let had_chainable: bool = any_chainable_exists(&clear.clear_queue, stack, &blocks);

                // max the chain and save data in a last chain
                if had_chainable {
                    clear.chain += 1;
                    clear.last_chain = max(clear.chain, clear.last_chain);
                }
                // otherwise reset the chain
                else {
                    clear.chain = 1;
                }

                // set all animation times and general time it will take all blocks that are
                // comboing to finish their animation
                for id in &clear.clear_queue {
                    let b = blocks.get_mut(stack[*id as usize]).unwrap();
                    let set_time = flash + face + pop * clear.combo_counter;
                    b.clear_time = set_time as i32;
                    clear.combo_counter += 1;

                    b.counter = all_time;
                    b.clear_start_counter = all_time as i32;
                    change_state(b, "CLEAR");
                }

                // clear the clear_queue if its not empty
                clear.blocks_cleared += clear.combo_counter;
                clear.clear_queue.clear();
                println!(
                    "chain: {}, combo: {}, blocks_cleared: {}",
                    clear.chain, clear.combo_counter, clear.blocks_cleared
                );
            }
        }
    }
}

// checks through each block's right, right_right and up, up_up to see if they are performing a combo
// returns an array of block ids to identify them
fn check_clear(x: usize, y: usize, stack: &Stack, blocks: &WriteStorage<'_, Block>) -> Vec<u32> {
    let mut checks: Vec<u32> = Vec::new();

    let r_rr = check_similar_block(x, y, 1, 0, stack, blocks);
    let u_uu = check_similar_block(x, y, 0, 1, stack, blocks);

    if let Some(mut right_vec) = r_rr {
        checks.append(&mut right_vec);
    }

    if let Some(mut up_vec) = u_uu {
        checks.append(&mut up_vec);
    }

    checks
}

// checks for similar blocks from the current block to 2 others
// checks if they all exist, are comboable, and also if their kinds match with the first
// returns an array of u32 ids of the blocks that are comboable or nothing
// to save on cpu -> not creating empty vecs
fn check_similar_block(
    x: usize,
    y: usize,
    x_offset: usize,
    y_offset: usize,
    stack: &Stack,
    blocks: &WriteStorage<'_, Block>,
) -> Option<Vec<u32>> {
    let b1 = blocks.get(stack[(x, y)]).unwrap();

    let check_boundary = |x: usize, y: usize| -> Option<&Block> {
        if x < COLS && y < ROWS {
            blocks.get(stack[(x, y)])
        } else {
            None
        }
    };

    let b2 = check_boundary(x + x_offset, y + y_offset);
    let b3 = check_boundary(x + x_offset * 2, y + y_offset * 2);

    if b1.is_comboable() {
        if let Some(block2) = b2 {
            if let Some(block3) = b3 {
                if block2.is_comboable_with(b1.kind) && block3.is_comboable_with(b1.kind) {
                    return Some(vec![b1.id, block2.id, block3.id]);
                }
            }
        }
    }

    // just return nothing to save up on cpu
    // we could just return an empty vec but since this happens around 72 * 2 times it's expensive to do so
    None
}

fn any_chainable_exists(
    clear_ids: &Vec<u32>,
    stack: &Stack,
    blocks: &WriteStorage<'_, Block>,
) -> bool {
    for id in clear_ids {
        if blocks.get(stack[*id as usize]).unwrap().chainable {
            return true;
        }
    }

    return false;
}
