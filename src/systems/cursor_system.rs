use amethyst::{core::Transform, ecs::*, input::*, renderer::*};

use block_states::block_state::change_state;
use block_states::swap::SWAP_TIME;
use components::{
    block::Block,
    cursor::Cursor,
    playfield::{clear::Clear, kind_generator::KindGenerator, push::Push, stack::Stack},
};
use data::block_data::*;

use std::collections::HashMap;

pub struct CursorSystem {
    key_presses: HashMap<String, i32>,
}

// everything the player controls should happen here
// all actions should happen here
impl CursorSystem {
    pub fn new() -> CursorSystem {
        let mut key_presses: HashMap<String, i32> = HashMap::new();
        key_presses.insert(String::from("up"), 0);
        key_presses.insert(String::from("down"), 0);
        key_presses.insert(String::from("right"), 0);
        key_presses.insert(String::from("left"), 0);
        key_presses.insert(String::from("swap"), 0);
        key_presses.insert(String::from("space"), 0);

        CursorSystem { key_presses }
    }

    // looks whether an action is held down, good for controller support later
    pub fn hold(&mut self, input: &mut Read<InputHandler<String, String>>, name: &str) -> bool {
        if input.action_is_down(name).unwrap() {
            let result = *self.key_presses.get(name).unwrap();

            // special, detects at frame 0 and later on returns true all the
            // time like in the real game
            if result == 0 || result > 16 {
                *self.key_presses.get_mut(name).unwrap() += 1;
                return true;
            }

            *self.key_presses.get_mut(name).unwrap() += 1;
        } else {
            *self.key_presses.get_mut(name).unwrap() = 0;
        }

        return false;
    }

    // looks whether an action is only pressed once, good for controller support later
    pub fn press(&mut self, input: &mut Read<InputHandler<String, String>>, name: &str) -> bool {
        if input.action_is_down(name).unwrap() {
            if *self.key_presses.get(name).unwrap() == 0 {
                *self.key_presses.get_mut(name).unwrap() = 1;
                return true;
            }
        } else {
            *self.key_presses.get_mut(name).unwrap() = 0;
        }

        return false;
    }
}

impl<'a> System<'a> for CursorSystem {
    type SystemData = (
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Cursor>,
        Read<'a, InputHandler<String, String>>,
        WriteStorage<'a, KindGenerator>,
        WriteStorage<'a, Block>,
        ReadStorage<'a, Stack>,
        WriteStorage<'a, Push>,
        WriteStorage<'a, Clear>,
    );

    fn run(
        &mut self,
        (
            mut sprites,
            mut transforms,
            mut cursors,
            mut input,
            mut kind_gens,
            mut blocks,
            stacks,
            mut pushes,
            mut clears,
        ): Self::SystemData,
    ) {
        if self.hold(&mut input, "up") {
            for cursor in (&mut cursors).join() {
                if cursor.y < (ROWS - 1) as f32 {
                    cursor.y += 1.0;
                }
            }
        }

        if self.hold(&mut input, "down") {
            for cursor in (&mut cursors).join() {
                if cursor.y > 1.0 {
                    cursor.y -= 1.0;
                }
            }
        }

        if self.hold(&mut input, "left") {
            for cursor in (&mut cursors).join() {
                if cursor.x > 0.0 {
                    cursor.x -= 1.0;
                }
            }
        }

        if self.hold(&mut input, "right") {
            for cursor in (&mut cursors).join() {
                if cursor.x < (COLS - 2) as f32 {
                    cursor.x += 1.0;
                }
            }
        }

        // reset all block colors to a random value
        if self.press(&mut input, "space") {
            for (stack, push, clear, kind_gen) in
                (&stacks, &mut pushes, &mut clears, &mut kind_gens).join()
            {
                let kinds = kind_gen.create_stack(5, 8);

                for i in 0..BLOCKS {
                    let b = blocks.get_mut(stack[i]).unwrap();
                    b.reset();
                    b.kind = kinds[i];
                }

                *push = Default::default();
                *clear = Default::default();
                *cursors.get_mut(stack.cursor_entity).unwrap() = Default::default();
            }
        }

        // swaps block kinds around, gets all blocks, searches through creation id,
        // id matches cursor pos conversion, swapping from one block to another block
        if self.press(&mut input, "swap") {
            for cursor in (cursors).join() {
                for stack in (&stacks).join() {
                    swap(cursor.x, cursor.y, &stack, &mut blocks);
                }
            }
        }

        // TODO: DONT SET THIS FOR EVERY PLAYFIELD, look for cursor specific input, then decide
        // which playfield.signal_raise should be set to true / false
        for playfield_push in (&mut pushes).join() {
            playfield_push.signal_raise = input.action_is_down("raise").unwrap();
        }

        for (sprite, transform, cursor) in (&mut sprites, &mut transforms, &mut cursors).join() {
            cursor.set_position(transform);

            sprite.sprite_number = cursor.anim_offset as usize;
            if cursor.anim_offset < 7.0 {
                cursor.anim_offset += 1.0 / 4.0;
            } else {
                cursor.anim_offset = 0.0;
            }
        }
    }
}

fn swap(x: f32, y: f32, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
    let i = Stack::xy2i(x as usize, y as usize);

    let mut can_swap: bool = false;
    {
        let b1 = blocks.get(stack[i]).unwrap();
        let b2 = blocks.get(stack[i + 1]).unwrap();

        let mut b1_above_block: Option<&Block> = None;
        let mut b2_above_block: Option<&Block> = None;

        if i < BLOCKS - COLS {
            b1_above_block = blocks.get(stack[i + COLS]);
            b2_above_block = blocks.get(stack[i + 1 + COLS]);
        }

        if b1.is_swappable(b2, b1_above_block) && b2.is_swappable(b1, b2_above_block) {
            if b1.is_empty() && b2.is_empty() {
                return;
            }

            can_swap = true;
        }
    }

    if can_swap {
        // set variables
        set_swap_variables(blocks.get_mut(stack[i]).unwrap(), 1.0);
        set_swap_variables(blocks.get_mut(stack[i + 1]).unwrap(), -1.0);

        // set default stack blocks
        let mut left_block = Block::default();
        let mut right_block = Block::default();

        // store data from the left to a temp
        left_block = blocks.get(stack[i]).unwrap().clone();

        // store data from the right to a temp
        right_block = blocks.get(stack[i + 1]).unwrap().clone();

        {
            blocks
                .get_mut(stack[i + 1])
                .unwrap()
                .set_properties(left_block);
        }

        {
            blocks
                .get_mut(stack[i])
                .unwrap()
                .set_properties(right_block);
        }
    }
}

// swap variables that need to be set on a different direction
fn set_swap_variables(b: &mut Block, dir: f32) {
    b.offset.0 = 16.0 * dir;
    b.counter = SWAP_TIME as u32;
    b.move_dir = dir;
    change_state(b, "SWAP");
}
