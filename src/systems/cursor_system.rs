use amethyst::{
    ecs::*,
    core::Transform,
    renderer::*,
    input::*
};

use basics::{
    block::Block,
    cursor::Cursor,
    kind_generator::KindGenerator,
    stack::Stack,
};

use data::{
    block_data::*, 
    helpers::tuple2i
};

use std::collections::HashMap;

pub struct CursorSystem {
    key_presses: HashMap<String, i32>
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

        CursorSystem {
            key_presses
        }
    }

    // looks wether an action is held down, good for controller support later
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
        }
        else {
            *self.key_presses.get_mut(name).unwrap() = 0;
        }

        return false;
    }

    // looks wether an action is only pressed once, good for controller support later
    pub fn press(&mut self, input: &mut Read<InputHandler<String, String>>, name: &str) -> bool {
        if input.action_is_down(name).unwrap() {
            if *self.key_presses.get(name).unwrap() == 0 {
                *self.key_presses.get_mut(name).unwrap() = 1;
                return true;
            }
        }
        else {
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
        Write<'a, KindGenerator>,
        WriteStorage<'a, Block>,
        Read<'a, Stack>,
    );

    fn run(&mut self, (
            mut sprites, 
            mut transforms,
            mut cursors, 
            mut input,
            mut kind_gen,
            mut blocks,
            stack,
            ): Self::SystemData) 
    {
        if self.hold(&mut input, "up") {
            for cursor in (&mut cursors).join() {
                if cursor.pos.1 < (ROWS - 1) as f32 {
                    cursor.pos.1 += 1.0;
                }
            }
        }

        if self.hold(&mut input, "down") {
            for cursor in (&mut cursors).join() {
                if cursor.pos.1 > 1.0 {
                    cursor.pos.1 -= 1.0;
                }
            }
        }

        if self.hold(&mut input, "left") {
            for cursor in (&mut cursors).join() {
                if cursor.pos.0 > 0.0 {
                    cursor.pos.0 -= 1.0;
                }
            }
        }

        if self.hold(&mut input, "right") {
            for cursor in (&mut cursors).join() {
                if cursor.pos.0 < (COLS - 2) as f32 {
                    cursor.pos.0 += 1.0;
                }
            }
        }

        // reset all block colors to a random value
        if self.press(&mut input, "space") {
            let kinds = kind_gen.create_stack(5, 8);
            
            for i in 0..BLOCKS {
                blocks.get_mut(stack.entities[i]).unwrap().kind = kinds[i];
            }
        }

        // swaps block kinds around, gets all blocks, searches through creation id,
        // id matches cursor pos conversion, swapping from one block to another block
        if self.press(&mut input, "swap") {
            for cursor in (cursors).join() {
                let pos = tuple2i(cursor.pos);

                let temp_kind: i32 = blocks.get_mut(stack.entities[pos]).unwrap().kind; 
                blocks.get_mut(stack.entities[pos]).unwrap().kind = blocks.get(stack.entities[pos + 1]).unwrap().kind;
                blocks.get_mut(stack.entities[pos + 1]).unwrap().kind = temp_kind;
            }
        }

        for (sprite, transform, cursor) in (&mut sprites, &mut transforms, &mut cursors).join() {
            cursor.set_position(transform);

            sprite.sprite_number = cursor.anim_offset as usize;
            if cursor.anim_offset < 7.0 {
                cursor.anim_offset += 1.0 / 4.0;
            }
            else {
                cursor.anim_offset = 0.0;
            }
        }
    }
}

/*
impl CursorSystem {
    fn swap(b: &mut Block, blocks: &mut WriteStorage<'_, Block>) {
        if let Some(down) = b.down {
            let other = (&mut blocks).get_mut(down).unwrap();

            let temp = b.kind; 
            b.kind = other.kind;
            other.kind = temp;
        }
    }
}*/