// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of cc-queue, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2018 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


#![feature(allocator_api)]
#![feature(integer_atomics)]


use self::allocators::*;
use ::std::cell::UnsafeCell;
use ::std::mem::transmute;
use ::std::mem::transmute_copy;
use ::std::ptr::drop_in_place;
use ::std::ptr::NonNull;
use ::std::ptr::null_mut;
use ::std::ptr::write;
use ::std::sync::atomic::AtomicPtr;
use ::std::sync::atomic::AtomicU32;
use ::std::sync::atomic::Ordering::AcqRel;
use ::std::sync::atomic::Ordering::Acquire;
use ::std::sync::atomic::Ordering::Release;
use ::std::sync::atomic::spin_loop_hint as PAUSE;


/// Allocators allow customization of the backing memory used by this queue.
/// One use case is to be able to use persistent memory.
pub mod allocators;


include!("IsNotNull.rs");
include!("Node.rs");
include!("PerQueueThreadHandle.rs");
include!("PerQueueThreadHandleInternal.rs");
include!("Queue.rs");
include!("QueueInternal.rs");
include!("Status.rs");
include!("Synch.rs");
include!("SynchHandle.rs");
include!("SynchNode.rs");
