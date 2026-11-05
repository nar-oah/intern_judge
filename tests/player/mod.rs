use super::*;
use crate::test_support::{advance, finish_app, headless_app};
use std::time::Duration;

mod movement;
mod physics;
mod stun;
mod support;
use support::{JUMP_SPEED, Movement, WALK_SPEED};
